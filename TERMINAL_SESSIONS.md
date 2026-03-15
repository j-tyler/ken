# Ken: Terminal Session Management

## Overview

This document describes how `ken wake` launches and orchestrates AI agent CLI sessions (Claude Code, OpenAI Codex, etc.) through terminal session management.

The design solves five problems:
1. Launching an agent CLI in a controllable, isolated workspace
2. Delivering frames and tasks to the agent
3. Knowing when the agent has finished each step
4. Collecting task output
5. Collecting the reflection

---

## Design Principle: Pull, Not Push

Ken does not screen-scrape terminal output or try to detect prompt states. Instead, the agent **pulls** from ken by running `ken done` — the only command the agent needs to learn.

Ken advances a simple state machine each time `ken done` is called. The terminal is just a launch mechanism. All structured data flows through the filesystem.

---

## The Agent-Side API

One command:

```bash
ken done
```

That's it. The agent runs this when it has finished whatever it was asked to do — whether that's processing a frame, completing a task, or writing a reflection.

`ken done` reads `KEN_SESSION` from the environment (set by ken at launch), advances the session state machine, and delivers the next instruction.

---

## The State Machine

```
┌─────────┐  ken done  ┌─────────┐  ken done       ken done  ┌──────┐  ken done  ┌──────────┐  ken done  ┌──────────┐
│ frame 0 │──────────→│ frame 1 │──────────→ ... ──────────→│ task │──────────→│ reflect  │──────────→│ complete │
└─────────┘           └─────────┘                            └──────┘           └──────────┘           └──────────┘
```

Each state delivers an instruction. The agent reads it, does the work, runs `ken done`. Ken advances and delivers the next instruction. When the machine reaches `complete`, ken tears down the session.

---

## Session Lifecycle

### 1. `ken wake` creates the session

```
ken wake cli/wake --task "Refactor the parser" --platform claude
```

Ken does the following:

1. Generate a session ID
2. Create a git worktree for workspace isolation
3. Create the session directory with all frames, task, and reflection prompt pre-written to disk
4. Write `state.json` with the state machine at frame 0
5. Launch a tmux session with `KEN_SESSION` and `KEN_SESSION_DIR` environment variables
6. Start the platform CLI (claude, codex) inside the tmux session
7. Deliver the first frame (via platform launch argument or send-keys after startup delay)

### 2. Workspace isolation via git worktrees

Each session gets its own git worktree so agents cannot interfere with each other:

```bash
git worktree add .ken/worktrees/{session-id} -b ken/{session-id}
```

The tmux session launches with `cwd` set to the worktree:

```bash
tmux new-session -d -s ken-{session-id} \
  -e KEN_SESSION={session-id} \
  -e KEN_SESSION_DIR={path-to-session-dir} \
  -c {path-to-worktree} \
  '{platform-launch-cmd}'
```

100 agents, 100 worktrees, 100 branches, zero conflicts.

### 3. Frame and task delivery

When `ken done` fires, ken sends a short message through tmux pointing the agent at the next pre-written instruction file:

```python
def deliver(session, step_name):
    content_file = session.dir / f"{step_name}.md"
    msg = (
        f"Your next instruction is in {content_file}. "
        f"Please read it and follow it. "
        f"When you are done, run `ken done`."
    )
    tmux_send(session.tmux_id, msg)
```

All instruction files (frames, task, reflection prompt) are written to the session directory at session creation time. `deliver()` just tells the agent where to look next.

Every delivery is the same shape. The agent never needs to know whether it's processing a frame, a task, or a reflection prompt. It reads, works, runs `ken done`.

### 4. Task output

Task prompts instruct the agent to write output to `result.md` in the working directory (the worktree). Ken reads this file from the worktree after `ken done` advances past the task phase.

There is no terminal output parsing. The agent writes files. Ken reads files.

### 5. Reflection collection

The reflection prompt is delivered like any other step. The agent writes its reflection, runs `ken done`. Ken reads the reflection from the expected file path and saves it to `reflections/{ken-path}/{timestamp}.md`.

### 6. Session completion

When the state machine reaches `complete`:
1. Ken saves the reflection to the project's reflections directory
2. Ken updates session state to `complete`
3. Ken kills the tmux session: `tmux kill-session -t ken-{id}`

The worktree and branch stay. They belong to the waker (see "Worktree Ownership" below).

---

## Worktree Ownership

Worktrees are owned by the waker, not the woken agent. When the woken agent finishes, the worktree persists with all of the agent's work — modified files, commits, everything. The waker reads the worktree directly using normal git commands:

```bash
cd .ken/worktrees/{id}
git log origin/main..HEAD        # see what the agent committed
git diff origin/main             # see all changes
```

The waker merges, cherry-picks, or discards the work at their discretion. Neither the waker nor the woken agent manages worktree lifecycle — ken does.

### Waker identity

Ken identifies the waker by recording the PID and start time of the process that invoked `ken wake`:

```python
def get_waker_identity():
    pid = os.getppid()
    start_time = open(f"/proc/{pid}/stat").read().split()[21]
    return {"pid": pid, "start_time": start_time}

def waker_alive(identity):
    try:
        os.kill(identity["pid"], 0)
        current_start = open(f"/proc/{identity['pid']}/stat").read().split()[21]
        return current_start == identity["start_time"]
    except (ProcessError, FileNotFoundError):
        return False
```

PID alone can be reused by the OS. PID + start time uniquely identifies a process — this is how systemd does it. This works for every waker type:

- **Human in a terminal**: PPID is their shell. Close the terminal, shell dies, PID gone.
- **Agent waker in tmux**: PPID is the shell in the tmux pane. Kill session, shell dies.
- **Agent waker via script**: PPID is whatever process ran `ken wake`.

### Cleanup

Cleanup is a separate, explicit action — not part of the session lifecycle:

```bash
ken clean {id}                # clean a specific session's worktree + branch
ken clean --stale 24h         # clean worktrees from sessions completed/dead > 24h ago
ken clean --all --force       # nuclear option: clean everything finished
```

```python
def clean(session_id):
    session = Session.load(session_id)

    if tmux_session_alive(session.tmux_session):
        die(f"Session {session_id} is still running")

    if waker_alive(session.waker_identity):
        die(f"Waker (pid {session.waker_identity['pid']}) still active")

    run(f"git worktree remove --force {session.worktree_path}")
    run(f"git branch -D ken/{session.id}")
    session.status = "cleaned"
    session.save()
```

Rules:
- Don't clean if the woken agent's tmux session is still alive.
- Don't clean if the waker process is still alive.
- Time-based expiry (`--stale`) handles abandoned worktrees.

---

## Liveness Monitoring

Ken runs a background monitor that checks for stalled woken agent sessions:

```python
def monitor(session, interval_minutes=5, max_timeout_minutes=120):
    deadline = time.time() + max_timeout_minutes * 60
    while session.status != "complete" and time.time() < deadline:
        last_activity = session.last_checkpoint_time()
        if time.time() - last_activity > interval_minutes * 60:
            tmux_send(
                session.tmux_id,
                "Are you still working? If so keep going! "
                "When you are done please notify me by running `ken done`."
            )
        time.sleep(60)

    if session.status != "complete":
        session.status = "timed_out"
        session.save()
        run(f"tmux kill-session -t {session.tmux_session}")
```

Same short, plain-ASCII message every time. The least fragile thing you can send through tmux. If the session exceeds the hard timeout (default 2 hours), ken marks it as timed out and kills the tmux session. The worktree stays for the waker to inspect.

---

## tmux Usage

tmux's role is minimal — just a process container and input channel:

| Operation | When | Fragility |
|---|---|---|
| `new-session -d -s {id} -e KEN_SESSION={id} -c {worktree} '{cmd}'` | Once at start | Rock solid |
| `send-keys -t {id} "{short msg}" Enter` | Per delivery + liveness nudge | Very reliable (short, ASCII, single-line) |
| `kill-session -t {id}` | Once at end | Rock solid |

tmux is never used to read output. All observation happens through the filesystem.

### Initial nudge

The agent needs to know to run `ken done` after its first instruction. Two options:

**Option A: Platform launch argument (preferred)**

Most CLI tools accept an initial prompt:

```bash
claude "Read the file {session_dir}/frame-00.md and follow its instructions. When done, run ken done."
```

Zero `send-keys` calls needed for the initial delivery.

**Option B: Fixed delay + send-keys**

Wait for the CLI to start (2-3 second delay), then send the first message via `send-keys`. Slightly more fragile but works for any platform.

---

## Session Directory Structure

```
.ken/sessions/{session-id}/
├── state.json              # State machine position + metadata
├── frame-00.md             # Frame prompts (written by ken at session creation)
├── frame-01.md
├── frame-02.md
├── task.md                 # Task prompt
├── reflect.md              # Reflection prompt
└── checkpoints/            # Timestamps from each ken done call
    ├── frame-00.ts
    ├── frame-01.ts
    └── ...
```

The worktree is at `.ken/worktrees/{session-id}/` and contains the agent's working copy of the repo. Task output files (like `result.md`) live in the worktree.

### state.json

```json
{
  "id": "a1b2c3d4",
  "ken_path": "cli/wake",
  "platform": "claude",
  "task": "Refactor the parser",
  "current_step": 0,
  "total_frames": 3,
  "task_delivered": false,
  "reflection_delivered": false,
  "status": "walking",
  "created_at": "2026-03-15T10:30:00Z",
  "worktree_path": ".ken/worktrees/a1b2c3d4",
  "tmux_session": "ken-a1b2c3d4"
}
```

Status values: `walking` | `working` | `reflecting` | `complete` | `timed_out`

---

## `ken done` Implementation

```python
def cmd_done():
    session_id = os.environ.get('KEN_SESSION')
    if not session_id:
        print("Error: not inside a ken session", file=sys.stderr)
        sys.exit(1)

    session = Session.load(session_id)
    session.save_checkpoint()
    advance(session)


def advance(session):
    if session.status == "walking":
        session.current_step += 1
        if session.current_step < session.total_frames:
            deliver(session, f"frame-{session.current_step:02d}")
        else:
            session.status = "working"
            deliver(session, "task")

    elif session.status == "working":
        session.status = "reflecting"
        deliver(session, "reflect")

    elif session.status == "reflecting":
        session.status = "complete"
        collect_reflection(session)
        tmux_kill(session.tmux_session)

    session.save()
```

Note: frame 0 is delivered by the initial nudge at session startup (see "Initial nudge" below), not by `advance()`. The first `ken done` call advances past frame 0 and delivers frame 1.

---

## Platform Drivers

The only platform-specific piece is the launch command:

```python
PLATFORMS = {
    "claude": {
        "cmd": "claude",
        "initial_prompt_support": True,
    },
    "codex": {
        "cmd": "codex",
        "initial_prompt_support": True,
    },
}
```

Everything else — frame delivery, state management, output collection — is platform-agnostic.

---

## Multi-Agent Orchestration

When a waker (human or agent) launches multiple sessions:

```bash
ken wake review/security --task "Audit auth module" --platform claude
ken wake review/perf --task "Profile hot paths" --platform claude
ken wake review/tests --task "Increase coverage" --platform codex
```

Each gets its own session ID, worktree, tmux session, and state machine. They run fully independently.

A parent agent or human can query status:

```bash
ken status                    # List all active sessions
ken status {session-id}       # Detail for one session
```

The session directory is the API. Any tool that can read files can observe session state.

---

## What Remains Fragile (and Mitigations)

| Failure mode | Mitigation |
|---|---|
| Agent never runs `ken done` | Liveness nudge every N minutes + hard timeout |
| Agent runs `ken done` prematurely | Frames should be clear about what "done" means |
| tmux `send-keys` garbles message | Messages are short, ASCII, single-line |
| tmux not installed | Check at startup, clear error |
| Agent ignores instruction file | Frame preamble repeats the contract; liveness nudge reminds |
| Two sessions collide | Worktrees + unique session IDs prevent this |
| CLI tool doesn't support initial prompt arg | Fall back to delay + `send-keys` |
| Worktree creation fails (dirty state) | Check git status before, clean error message |

---

## Summary

- **Agent API**: `ken done` (one command)
- **Isolation**: git worktrees (one per session)
- **Communication**: filesystem (ken writes instruction files, agent writes result files)
- **Terminal management**: tmux (launch + short message delivery only)
- **State**: simple linear state machine in `state.json`
- **Output**: agent writes `result.md` in worktree as instructed by task prompts
- **Liveness**: periodic nudge via tmux `send-keys`
- **Platform independence**: only the launch command differs per platform
