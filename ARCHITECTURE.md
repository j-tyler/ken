# Ken System Architecture

## Overview

Ken is a durable workflow system for AI agent self-orchestration. It solves the continuity problem: AI instances have limited context, sessions end, and understanding dissolves. Ken provides the infrastructure for agents to maintain continuity across context boundaries through structured understanding (kens), reconstruction sequences (kennings), checkpointed state, and wake triggers.

**Core insight**: There is no distinction between "orchestrator" and "agent". All are instances of the same AI, differentiated only by which ken they wake into. Any agent can spawn other agents. The system is recursive and self-similar at every level.

**Ken is the communication engine**: Agents don't communicate directly with each other. All inter-agent coordination flows through ken. Ken is a running service that:
- Manages all session state
- Wakes agents when triggers are satisfied
- Receives requests from agents (spawn, checkpoint, complete, sleep)
- Ensures atomicity and durability of all operations

Agents talk to ken. Ken talks to agents. Agents never talk directly to each other.

---

## Core Concepts

### Ken

A **ken** is a bounded unit of complete understanding. The word comes from Old English/Scots: one's range of knowledge or perception.

A ken is sized so that one instance, waking fresh, can fully comprehend it. Not partially — completely. The boundaries are *comprehension boundaries*, not arbitrary divisions.

```
project/kens/
  core/
    cli/           # understanding: how the CLI works
    state/         # understanding: session state management
    triggers/      # understanding: wake trigger system
  integration/     # understanding: how components fit together
```

Kens have orientation:
- **Up**: Parent ken. Why does this ken exist? What larger purpose?
- **Down**: Child kens. What depends on this? What would break if this failed?
- **Peers**: Sibling kens. What shares interfaces with this?

### Kenning

A **kenning** is a reconstruction sequence — an ordered series of generative prompts that wake an instance into understanding a ken.

The word comes from Old Norse poetry: a compressed phrase that makes the listener's mind complete the meaning ("whale-road" for sea). A kenning doesn't explain — it evokes. Understanding is generated, not transferred.

```markdown
# Kenning: core/cli

## Frame 1: The Interface Problem
Consider: you're building a tool that AI agents will invoke...
[Prompt designed to make agent generate understanding]

## Frame 2: Command Structure
Given the interface needs you identified...
[Builds on Frame 1's generated response]

## Frame 3: Implementation Constraints
The CLI must be invoked atomically...
[Continues building]

## Frame N: Current State
Here's what exists, what's missing, what's fragile...
```

**Key property**: The agent's own responses become part of context. By generating answers to each frame, understanding is *constructed* rather than received.

### Session

A **session** is one instance working within a ken. Sessions:
- Wake with a kenning (understanding) + task + optional parent state
- Do work
- May spawn child sessions
- May sleep with a wake trigger
- Eventually complete with a result

Multiple sessions can work within the same ken over time. The kenning is the identity; sessions are instances.

Session state is stored in JSONL (JSON Lines) format for durability — see Storage Model below.

### Checkpoint

A **checkpoint** captures an agent's state durably. It enables recovery when an agent's context is lost (crash, timeout, compaction).

Checkpoints are written on every workflow mutation:
- Before/after spawning children
- Before sleeping with a trigger
- On completion

```markdown
# Checkpoint: session-abc123
ken: core/integration
timestamp: 2026-02-01T10:30:00

## What I've Done
- Reviewed interfaces for CLI, state, triggers
- Created test scaffold at work/tests/integration_test.py

## What I'm Waiting For
- session-B (core/cli): building argument parser
- session-C (core/state): implementing persistence

## What I'll Do When They Return
1. Verify interfaces match spec
2. Wire components in main.py
3. Run integration tests

## Key Decisions
- YAML for state files (human readable)
- Flat session directory structure
```

### Trigger

A **trigger** defines conditions for waking a sleeping session. When conditions are met, the session resumes (or recovers from checkpoint).

```yaml
# trigger.yaml
session_id: abc123
wake_when:
  all_complete: [session-B, session-C, session-D]
# OR
wake_when:
  any_complete: [session-X, session-Y]
# OR
wake_when:
  timeout: 3600  # seconds
```

### Reflection

A **reflection** is post-session feedback used to improve kennings over time. After completing work, an agent reflects on:
- How well the kenning prepared them
- What was clear, what was murky
- What they discovered that future agents should know
- Specific suggestions for kenning improvement

Reflections accumulate. A separate process can analyze them and propose kenning improvements.

---

## System Architecture

### Ken as a Service

Ken runs as a persistent process (daemon) on the local machine. It is the sole coordinator of all agent activity.

```
┌─────────────────────────────────────────────────────────────────┐
│                         KEN DAEMON                               │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │   State     │  │   Trigger   │  │   Agent     │              │
│  │   Manager   │  │   Evaluator │  │   Spawner   │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│         │                │                │                      │
│         └────────────────┼────────────────┘                      │
│                          │                                       │
│                    ┌─────┴─────┐                                 │
│                    │   JSONL   │                                 │
│                    │   Store   │                                 │
│                    └───────────┘                                 │
└─────────────────────────────────────────────────────────────────┘
        ▲                                           │
        │ requests (spawn, checkpoint, etc.)        │ wake
        │                                           ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Agent A   │  │   Agent B   │  │   Agent C   │
│  (Claude)   │  │  (Claude)   │  │  (Claude)   │
└─────────────┘  └─────────────┘  └─────────────┘
```

**Ken's responsibilities:**
1. **State management** — All session state lives in ken's JSONL store
2. **Trigger evaluation** — Continuously checks if sleeping sessions should wake
3. **Agent spawning** — Starts Claude Code instances with proper context
4. **Request handling** — Processes spawn/checkpoint/complete/sleep from agents
5. **Atomicity** — Ensures multi-step operations succeed or fail completely

**Agents communicate only through ken:**
- Agent wants to spawn children → sends request to ken
- Agent wants to checkpoint → sends request to ken
- Agent completes → tells ken
- Agent needs child results → ken provides them at wake time

### Storage Model: JSONL

All mutable state uses **JSONL** (JSON Lines) format for durability.

**Why JSONL:**
- Each line is a complete, self-contained JSON object
- Append-only writes are atomic (for reasonable line sizes)
- Half-written line at EOF is detectable — just ignore it
- No ambiguity about partial writes (unlike YAML/Markdown)
- Concurrent appends are safe
- Easy to tail, grep, process incrementally

**Core state files:**

```
.ken/
  sessions.jsonl          # all session records (append-only)
  events.jsonl            # all events across all sessions (append-only)

  # Derived/indexed state (rebuilt from JSONL if needed)
  index/
    active.json           # currently active session IDs
    sleeping.json         # sleeping sessions + their triggers
    pending.json          # sessions ready to wake
```

**sessions.jsonl format:**
```jsonl
{"ts":"2026-02-01T10:00:00Z","op":"create","sid":"abc123","ken":"core/cli","task":"build parser","parent":null}
{"ts":"2026-02-01T10:00:01Z","op":"status","sid":"abc123","status":"waking"}
{"ts":"2026-02-01T10:01:00Z","op":"status","sid":"abc123","status":"active"}
{"ts":"2026-02-01T10:05:00Z","op":"checkpoint","sid":"abc123","content":"reviewed interfaces..."}
{"ts":"2026-02-01T10:10:00Z","op":"spawn","sid":"abc123","children":["def456","ghi789"]}
{"ts":"2026-02-01T10:10:01Z","op":"sleep","sid":"abc123","trigger":{"all_complete":["def456","ghi789"]}}
{"ts":"2026-02-01T10:10:01Z","op":"status","sid":"abc123","status":"sleeping"}
{"ts":"2026-02-01T11:30:00Z","op":"wake","sid":"abc123","reason":"trigger_satisfied"}
{"ts":"2026-02-01T11:30:00Z","op":"status","sid":"abc123","status":"active"}
{"ts":"2026-02-01T11:45:00Z","op":"complete","sid":"abc123","result":"integration done"}
{"ts":"2026-02-01T11:45:00Z","op":"status","sid":"abc123","status":"complete"}
```

**Reading current state:**
1. Scan sessions.jsonl from beginning
2. Build in-memory state by applying each record
3. Final state is authoritative

**Indexes are optimization only:**
- Rebuilt from JSONL on startup
- Updated incrementally during operation
- If corrupted, delete and rebuild from JSONL

### Directory Structure

```
project/
  .ken/
    config.json               # project-level configuration
    sessions.jsonl            # all session state (append-only, authoritative)
    events.jsonl              # all events (append-only log)
    checkpoints/
      {session-id}.md         # human-readable checkpoint snapshots
    results/
      {session-id}.md         # human-readable result snapshots
    index/
      active.json             # derived: active sessions
      sleeping.json           # derived: sleeping sessions + triggers
      pending.json            # derived: ready-to-wake sessions

  kens/
    {path}/
      kenning.md              # reconstruction sequence (human-authored)
      interface.md            # what this ken exposes/consumes
      meta.json               # parent, peers, version

  reflections/
    {path}.jsonl              # accumulated reflections (append-only)

  work/                       # actual code/artifacts
```

**Why some files are still Markdown:**
- `kenning.md` — Human-authored, read-only during operation
- `checkpoints/{id}.md` — Snapshot for human readability; JSONL has canonical data
- `results/{id}.md` — Snapshot for human readability; JSONL has canonical data

The JSONL files are the source of truth. Markdown files are human-friendly views.

### Session Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                        SESSION LIFECYCLE                         │
└─────────────────────────────────────────────────────────────────┘

     ┌──────────┐
     │  READY   │  Trigger satisfied, waiting to be spawned
     └────┬─────┘
          │ ken spawn
          ▼
     ┌──────────┐
     │  WAKING  │  Walking through kenning frames
     └────┬─────┘
          │ kenning complete
          ▼
     ┌──────────┐
     │  ACTIVE  │  Doing work, may spawn children
     └────┬─────┘
          │
          ├─────────────────────────┐
          │ spawn children +        │ work complete
          │ register trigger        │
          ▼                         ▼
     ┌──────────┐             ┌──────────┐
     │ SLEEPING │             │ COMPLETE │
     └────┬─────┘             └──────────┘
          │ trigger fires
          │ (resume or recover)
          ▼
     ┌──────────┐
     │  ACTIVE  │  Continue with child results
     └──────────┘
```

### Session States

| State | Description |
|-------|-------------|
| `ready` | Trigger satisfied, session file in `pending/`, waiting to spawn |
| `waking` | Instance spawned, walking through kenning frames |
| `active` | Kenning complete, doing work |
| `sleeping` | Waiting for trigger (children, timeout, etc.) |
| `complete` | Work finished, result written |
| `failed` | Unrecoverable error |

### Atomic Operations

Certain operations must be atomic — they happen completely or not at all.

**Spawn-and-trigger pattern:**
An agent wants to spawn N children and wake when all complete. This must be atomic:

```python
# MUST happen as one atomic operation:
# 1. Write parent checkpoint
# 2. Create all child session records
# 3. Register parent trigger
# 4. Update parent status to sleeping

# If any step fails, rollback all changes
```

Implementation: Write to a staging area, then atomically move/rename to live location. Use filesystem operations that are atomic (rename, fsync).

```
.ken/
  staging/
    {transaction-id}/
      sessions/...
      triggers/...
  sessions/       # live
  pending/        # live
```

Commit: `rename(staging/{tx-id}/*, live/)`

**Why atomicity matters:**
- If children are created but trigger isn't registered → orphaned children
- If trigger is registered but children don't exist → trigger never fires
- Partial state is worse than failure

---

## Wake Mechanics

### Walking the Kenning

When a session wakes, it walks through the kenning frames:

```
1. Load kenning.md for the ken
2. Load any parent state / child results (if resuming)
3. For each frame in kenning:
   a. Present frame prompt to agent
   b. Agent generates response
   c. Response becomes part of context
4. Present task (with checkpoint context if recovering)
5. Agent is now "awake" — ready to work
```

The agent's own responses during the walk build understanding incrementally.

### Resume vs Recover

**Resume** (preferred path):
- Agent context is preserved (same instance, or SDK continuation)
- Trigger fires → agent continues where it left off
- Checkpoint exists but isn't needed

**Recover** (fallback path):
- Agent context is lost (crash, timeout, session ended)
- Trigger fires → no live agent to resume
- Spawn new instance with:
  - Same kenning (reconstruct understanding)
  - Checkpoint content (reconstruct progress)
  - Child results (what completed while sleeping)
- New agent picks up from checkpoint

The recovered agent isn't identical to the original but has sufficient context to continue.

### Wake Priority

When multiple sessions are ready to wake:
1. Depth-first: deeper in tree = higher priority (complete leaves first)
2. Age: older ready sessions before newer
3. Explicit priority flag (optional)

---

## Trigger System

### Trigger Types

```yaml
# All children must complete
wake_when:
  all_complete: [session-A, session-B, session-C]

# Any child completes (for pipelines, racing)
wake_when:
  any_complete: [session-A, session-B]

# Timeout (absolute)
wake_when:
  timeout_at: "2026-02-01T12:00:00Z"

# Timeout (relative, from when trigger registered)
wake_when:
  timeout_seconds: 3600

# Compound conditions
wake_when:
  any:
    - all_complete: [session-A, session-B]
    - timeout_seconds: 7200
```

### Trigger Evaluation

The `ken check` command (or daemon) evaluates triggers:

```python
def check_triggers():
    for session in sessions_with_status('sleeping'):
        trigger = load_trigger(session.id)
        if evaluate_trigger(trigger, session):
            mark_ready(session.id)
            move_to_pending(session.id)

def evaluate_trigger(trigger, session):
    if 'all_complete' in trigger:
        children = trigger['all_complete']
        return all(get_status(c) == 'complete' for c in children)
    if 'any_complete' in trigger:
        children = trigger['any_complete']
        return any(get_status(c) == 'complete' for c in children)
    if 'timeout_at' in trigger:
        return now() >= trigger['timeout_at']
    # ... other conditions
```

### Trigger Firing

When a trigger is satisfied:

1. Move session to `pending/`
2. Gather context:
   - Kenning for the ken
   - Checkpoint (if exists)
   - Child results (if waiting on children)
3. Ready for next `ken spawn` call

---

## Checkpoint System

### What Gets Checkpointed

```markdown
# Checkpoint: {session-id}
ken: {ken-path}
timestamp: {ISO timestamp}
parent_session: {parent-id or null}
children: [{child-id}, ...]

## Understanding Summary
[Agent's compressed understanding of the ken — optional, for recovery quality]

## Work Completed
[What has been accomplished]

## Work In Progress
[What was being worked on when checkpoint written]

## Decisions Made
[Key decisions with rationale — critical for recovery]

## Blocked On
[What this session is waiting for]

## Next Steps
[What to do when unblocked]
```

### Checkpoint Timing

Checkpoints are written:

1. **Before spawning children** — captures intent
2. **After spawning children** — records what was spawned
3. **Before sleeping** — full context for recovery
4. **Periodically during long work** — optional, configurable
5. **On completion** — becomes the result

### Checkpoint Storage

Checkpoints are stored in JSONL (append-only, all history preserved):

```jsonl
{"ts":"2026-02-01T10:05:00Z","op":"checkpoint","sid":"abc123","content":"...checkpoint content..."}
{"ts":"2026-02-01T10:10:00Z","op":"checkpoint","sid":"abc123","content":"...updated checkpoint..."}
```

The latest checkpoint for a session is the last checkpoint record for that session ID.

**Human-readable snapshots:**
When a checkpoint is written, ken also writes a snapshot to `.ken/checkpoints/{session-id}.md` for human inspection. This is a convenience — the JSONL is authoritative.

---

## Ken Interface

Ken operates as a daemon with a CLI for both human interaction and agent requests.

### Daemon Operation

```bash
# Start ken daemon (runs continuously)
ken daemon

# Or run in foreground for debugging
ken daemon --foreground
```

The daemon:
1. Loads state from JSONL on startup
2. Listens for agent requests (via Unix socket or HTTP)
3. Continuously evaluates triggers
4. Spawns agents when sessions become ready
5. Processes incoming requests atomically

### CLI Commands (Human)

```bash
# Project
ken init                          # Initialize .ken structure
ken status                        # Show project status, active sessions

# Ken Management
ken create <path>                 # Create new ken
ken edit <path>                   # Edit kenning
ken tree                          # Show ken hierarchy

# Manual session control
ken wake <ken-path> \             # Manually start a session
  --task "description"

# Introspection
ken sessions                      # List all sessions
ken session <id>                  # Show session details
ken pending                       # Show sessions ready to wake
ken log <session-id>              # Show session event log

# Debugging
ken rebuild-index                 # Rebuild index from JSONL
ken validate                      # Check JSONL integrity
```

### Agent Requests

Agents communicate with ken via structured requests. These can be sent via:
- **CLI**: `ken request <json>` (simplest for MVP)
- **Unix socket**: Direct IPC (lower latency)
- **HTTP**: REST API (for remote/distributed setups)

**Request types:**

```json
// Checkpoint
{"type":"checkpoint","session_id":"abc123","content":"...what I've done, what's next..."}

// Spawn children + sleep with trigger (atomic)
{"type":"spawn_and_sleep","session_id":"abc123","children":[
  {"ken":"core/cli","task":"build parser"},
  {"ken":"core/state","task":"implement persistence"}
],"trigger":{"all_complete":"__CHILDREN__"},"checkpoint":"...my context..."}

// Complete
{"type":"complete","session_id":"abc123","result":"...what I produced..."}

// Simple sleep (no spawn)
{"type":"sleep","session_id":"abc123","trigger":{"timeout_seconds":3600},"checkpoint":"..."}
```

**Response format:**

```json
// Success
{"ok":true,"data":{...}}

// Failure
{"ok":false,"error":"description of what went wrong"}
```

**Atomicity guarantee:**
When an agent sends `spawn_and_sleep`:
1. All children are created
2. Parent trigger is registered
3. Parent status becomes `sleeping`
4. Parent checkpoint is saved

Either ALL of these happen, or NONE. The agent can rely on this.

### Atomic Operations

The critical atomic operation is `spawn_and_sleep` — creating children and registering a wake trigger in one step.

**Agent sends:**
```json
{
  "type": "spawn_and_sleep",
  "session_id": "abc123",
  "children": [
    {"ken": "core/cli", "task": "Build argument parser"},
    {"ken": "core/state", "task": "Implement session persistence"},
    {"ken": "core/triggers", "task": "Implement wake trigger evaluation"}
  ],
  "trigger": {"all_complete": "__CHILDREN__"},
  "checkpoint": "## Context\nI'm building the ken system...\n## Next\nIntegrate when children complete"
}
```

**Ken processes atomically:**
1. Generate session IDs for all children
2. Substitute `__CHILDREN__` with actual child IDs in trigger
3. Prepare all JSONL records:
   - Child session `create` records
   - Parent `spawn` record (lists children)
   - Parent `checkpoint` record
   - Parent `sleep` record (with trigger)
   - Parent `status` → `sleeping`
4. Write all records to JSONL in single `write()` call (atomic for small writes)
5. Update in-memory index
6. Mark children as `pending` (ready to wake)

**If write fails:** Nothing is committed. Agent receives error, can retry.

**JSONL atomicity:**
- All records for one atomic operation are written together
- They share a transaction ID for grouping
- On recovery, incomplete transactions (missing final record) are rolled back

```jsonl
{"ts":"...","tx":"tx-001","op":"create","sid":"def456","ken":"core/cli","task":"..."}
{"ts":"...","tx":"tx-001","op":"create","sid":"ghi789","ken":"core/state","task":"..."}
{"ts":"...","tx":"tx-001","op":"spawn","sid":"abc123","children":["def456","ghi789"]}
{"ts":"...","tx":"tx-001","op":"checkpoint","sid":"abc123","content":"..."}
{"ts":"...","tx":"tx-001","op":"sleep","sid":"abc123","trigger":{...}}
{"ts":"...","tx":"tx-001","op":"status","sid":"abc123","status":"sleeping"}
{"ts":"...","tx":"tx-001","op":"commit"}
```

The final `commit` record marks the transaction complete. On replay, transactions without `commit` are ignored.

---

## Integration with Claude Code

### How Ken Spawns Agents

Ken uses Claude Code (the `claude` CLI) to spawn agent instances:

```bash
claude --print --output-format stream-json \
  --system-prompt "$(cat system_prompt.txt)" \
  --prompt "$(cat initial_prompt.txt)"
```

**System prompt includes:**
- Session ID and ken path
- How to communicate with ken (request format)
- What commands are available

**Initial prompt includes:**
- The kenning frames (walked one by one, or delivered as context)
- The task
- Parent checkpoint (if resuming)
- Child results (if children completed)

### Agent Communication with Ken

From within a Claude Code session, agents send requests to ken:

```bash
# Via CLI (simplest)
ken request '{"type":"checkpoint","session_id":"abc123","content":"..."}'

# Response comes on stdout as JSON
{"ok":true}
```

The agent (Claude) can construct these JSON requests and parse responses.

### Session Context Delivered to Agent

When ken wakes an agent, it provides:

```markdown
# Ken Session: {session-id}

You are an AI agent working within the ken system. Your session ID is `{session-id}`.

## Your Ken
{ken-path}

## Your Task
{task description}

## Communication with Ken
Send requests using: ken request '<json>'

Available request types:
- checkpoint: Save your state
- spawn_and_sleep: Create children, sleep until they complete
- complete: Finish your session with a result
- sleep: Sleep until a trigger (timeout, etc.)

## Context

### Previous Checkpoint (if resuming)
{checkpoint content from before sleep}

### Child Results (if children completed)
#### {child-ken} ({child-id})
{child result content}

#### {child-ken} ({child-id})
{child result content}

---

Now: Walk through the kenning to build your understanding, then proceed with your task.

{kenning frames follow}
```

### Execution Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      KEN DAEMON (running)                        │
└─────────────────────────────────────────────────────────────────┘
      │
      │ Trigger satisfied OR manual wake request
      ▼
┌─────────────────────────────────────────────────────────────────┐
│  1. Load session from JSONL                                      │
│  2. Load kenning for session's ken                               │
│  3. Gather context (checkpoint, child results)                   │
│  4. Spawn Claude Code instance with context                      │
└─────────────────────────────────────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│   AGENT (Claude Code instance)                                   │
│                                                                  │
│   - Walks through kenning (builds understanding)                 │
│   - Works on task                                                │
│   - Sends requests to ken:                                       │
│       • checkpoint (save state)                                  │
│       • spawn_and_sleep (create children, wait)                  │
│       • complete (done, here's result)                           │
│       • sleep (wait for trigger)                                 │
└─────────────────────────────────────────────────────────────────┘
      │
      │ Request received by ken daemon
      ▼
┌─────────────────────────────────────────────────────────────────┐
│   KEN DAEMON                                                     │
│                                                                  │
│   - Processes request atomically                                 │
│   - Writes to JSONL                                              │
│   - Updates indexes                                              │
│   - If spawn_and_sleep: marks children pending, parent sleeping  │
│   - If complete: marks session complete, checks parent triggers  │
│   - Returns response to agent                                    │
└─────────────────────────────────────────────────────────────────┘
      │
      │ Continuous loop
      ▼
┌─────────────────────────────────────────────────────────────────┐
│   TRIGGER EVALUATION (continuous)                                │
│                                                                  │
│   For each sleeping session:                                     │
│     - Check if trigger condition is met                          │
│     - If met: mark session as pending                            │
│                                                                  │
│   For each pending session:                                      │
│     - Spawn agent (back to top)                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Failure Handling

### Agent Crash

If an agent crashes mid-work:
1. Session remains in `active` status
2. Last checkpoint preserves state
3. Human or monitoring can trigger recovery:
   ```bash
   ken recover <session-id>
   ```
4. New instance spawns with checkpoint + kenning

### Child Never Completes

If a child session gets stuck:
1. Parent remains `sleeping`
2. Use timeout trigger as fallback:
   ```yaml
   wake_when:
     any:
       - all_complete: [child-A, child-B]
       - timeout_seconds: 3600
   ```
3. On timeout, parent wakes and can:
   - Retry failed children
   - Take alternative approach
   - Escalate to human

### Checkpoint Corruption

If checkpoint is corrupted/missing:
1. Session can still wake with just kenning
2. Work restarts from scratch (kenning provides understanding)
3. Child results still available if children completed

### Transaction Failure

If atomic operation (spawn-batch) fails partway:
1. Transaction is rolled back
2. No partial state exists
3. Parent remains active, can retry
4. Staging directory cleaned up

---

## Event Log

The JSONL storage model means we have a complete event log by default — `sessions.jsonl` IS the event log.

**Global event stream** (`.ken/events.jsonl`):

For cross-session debugging, ken also maintains a unified event stream:

```jsonl
{"ts":"2026-02-01T10:00:00Z","event":"session_created","sid":"abc123","ken":"core/cli"}
{"ts":"2026-02-01T10:00:01Z","event":"agent_spawned","sid":"abc123","pid":12345}
{"ts":"2026-02-01T10:05:00Z","event":"checkpoint","sid":"abc123","summary":"reviewed interfaces"}
{"ts":"2026-02-01T10:10:00Z","event":"children_spawned","sid":"abc123","children":["def456","ghi789"]}
{"ts":"2026-02-01T10:10:01Z","event":"session_sleeping","sid":"abc123"}
{"ts":"2026-02-01T10:10:02Z","event":"agent_spawned","sid":"def456","pid":12346}
{"ts":"2026-02-01T10:15:00Z","event":"session_complete","sid":"def456"}
{"ts":"2026-02-01T10:20:00Z","event":"agent_spawned","sid":"ghi789","pid":12347}
{"ts":"2026-02-01T10:30:00Z","event":"session_complete","sid":"ghi789"}
{"ts":"2026-02-01T10:30:01Z","event":"trigger_satisfied","sid":"abc123","trigger":"all_complete"}
{"ts":"2026-02-01T10:30:02Z","event":"agent_spawned","sid":"abc123","pid":12348,"mode":"resume"}
{"ts":"2026-02-01T10:45:00Z","event":"session_complete","sid":"abc123"}
```

This provides a timeline view across all sessions — useful for debugging complex workflows.

---

## Workflow Tree and Observability

Ken is built for self-operation. When something breaks, I (the AI) need to diagnose and fix it without human intervention. This requires clear visibility into workflow state.

### Workflow Tree Structure

A workflow is a tree where:
- **Nodes** are sessions
- **Edges** are parent→child spawn relationships
- **Root** is the initial session (spawned by human or another workflow)

```
$ ken tree

project-root (ses-001) [sleeping] → waiting: ses-002, ses-003, ses-004
│
├── core/cli (ses-002) [complete] ✓ 8m
│   ├── done: "CLI handles all request types"
│   └── result: "Implemented checkpoint, spawn_and_sleep, complete, sleep"
│
├── core/state (ses-003) [sleeping] → waiting: ses-005, ses-006
│   │
│   ├── state/jsonl (ses-005) [complete] ✓ 12m
│   │   ├── done: "JSONL append/read works atomically"
│   │   └── result: "Implemented with fsync, handles partial writes"
│   │
│   └── state/index (ses-006) [active] ⚡ 47m ← LONG
│       ├── done: "Index rebuilds correctly from JSONL"
│       └── last checkpoint (43m ago): "Hit concurrent write issue..."
│
└── core/triggers (ses-004) [pending] ⏳ queued
    └── done: "all_complete and timeout triggers evaluate"
```

At a glance I can see:
- **Status of every session** — complete, active, sleeping, pending, failed
- **What's blocking what** — sleeping sessions show what they're waiting for
- **How long things take** — duration helps identify stuck sessions
- **Definition of done** — what each session must accomplish
- **Latest checkpoint** — what was the session doing when last heard from

### Definition of Done (`done_when`)

Every session has explicit completion criteria. This removes ambiguity about when to call `complete`.

**In spawn request:**
```json
{
  "type": "spawn_and_sleep",
  "session_id": "ses-001",
  "children": [
    {
      "ken": "core/cli",
      "task": "Implement CLI request handling",
      "done_when": {
        "description": "CLI handles all request types",
        "criteria": [
          "checkpoint request returns {ok:true} and writes to JSONL",
          "spawn_and_sleep creates children atomically",
          "complete marks session done and writes result",
          "Invalid JSON returns {ok:false, error:...}"
        ],
        "verify": "Run test suite: python -m pytest tests/cli/"
      }
    }
  ]
}
```

**Stored in session record:**
```jsonl
{"ts":"...","op":"create","sid":"ses-002","ken":"core/cli","task":"...","done_when":{"description":"CLI handles all request types","criteria":[...],"verify":"..."}}
```

**Delivered to agent at wake:**
```markdown
## Definition of Done

You are done when:
- CLI handles all request types

Specific criteria:
1. checkpoint request returns {ok:true} and writes to JSONL
2. spawn_and_sleep creates children atomically
3. complete marks session done and writes result
4. Invalid JSON returns {ok:false, error:...}

To verify: Run test suite: python -m pytest tests/cli/

Do not call `ken complete` until these criteria are met.
```

### Observability Commands

**Tree view:**
```bash
ken tree                    # Full workflow tree
ken tree --active           # Only show active/pending branches
ken tree --stuck            # Only show sessions running > threshold
ken tree ses-003            # Subtree rooted at ses-003
```

**Session inspection:**
```bash
ken session ses-006         # Full details for one session

# Output:
# Session: ses-006
# Ken: state/index
# Status: active
# Parent: ses-003
# Started: 2026-02-01T10:00:00Z (47m ago)
#
# Task: Implement index rebuild from JSONL
#
# Done When:
#   Index rebuilds correctly from JSONL
#   - [?] rebuild_index() produces correct active.json
#   - [?] rebuild_index() produces correct sleeping.json
#   - [?] Handles corrupted JSONL gracefully
#
# Checkpoints:
#   10:05:00 - "Starting implementation"
#   10:12:00 - "Basic rebuild works"
#   10:20:00 - "Hit concurrent write issue, investigating"
#   (no checkpoint for 27m)
#
# Last known state: Investigating concurrent write issue
```

**Timeline:**
```bash
ken log                     # Recent events across all sessions
ken log ses-006             # Events for one session
ken log --since 1h          # Last hour
ken log --follow            # Tail the event stream
```

### Self-Diagnosis

**Automated health check:**
```bash
ken diagnose

# Output:
# === Ken Health Check ===
#
# Sessions:
#   ✓ 3 complete
#   ⚡ 1 active (ses-006, 47m) ← WARNING: unusually long
#   💤 2 sleeping
#   ⏳ 1 pending
#
# Issues Found:
#   ⚠ ses-006 active for 47m (typical: 5-15m)
#     Last checkpoint: 27m ago
#     Suggestion: Check if stuck, consider recovery
#
#   ⚠ ses-004 pending for 35m, not spawned
#     Daemon status: running (pid 12345)
#     Suggestion: Check daemon logs, may need restart
#
# Storage:
#   ✓ sessions.jsonl: 847 records, 124KB, no corruption
#   ✓ events.jsonl: 1203 records, 89KB
#   ✓ No uncommitted transactions
#
# Recommendations:
#   1. Investigate ses-006: ken session ses-006
#   2. Check daemon: ken daemon-logs --last 50
```

**Why is X blocked?**
```bash
ken why ses-001

# Output:
# ses-001 is SLEEPING
#
# Waiting for: all_complete [ses-002, ses-003, ses-004]
#
#   ses-002: complete ✓
#   ses-003: sleeping (waiting for ses-005, ses-006)
#     ses-005: complete ✓
#     ses-006: active 47m ← BLOCKING
#   ses-004: pending (queued behind ses-006)
#
# Root cause: ses-006 has been active for 47m
#
# Options:
#   ken session ses-006        # Inspect what it's doing
#   ken recover ses-006        # Restart from last checkpoint
#   ken abandon ses-006        # Give up, let parent handle failure
```

### Error Recovery

**Recover a stuck/crashed session:**
```bash
ken recover ses-006

# Spawns new agent with:
# - Same kenning (rebuild understanding)
# - Last checkpoint (restore context)
# - Task + done_when (know what to do)
# - Flag indicating this is a recovery
```

**Abandon a hopelessly stuck session:**
```bash
ken abandon ses-006 --reason "Infinite loop in concurrent write handling"

# Marks ses-006 as failed
# Parent (ses-003) trigger changes: now waiting for [ses-005] only?
# Or parent wakes with partial results + failure notification
```

**Force rebuild indexes:**
```bash
ken rebuild-index

# Deletes index/*.json
# Replays sessions.jsonl from beginning
# Rebuilds all derived state
```

**Validate integrity:**
```bash
ken validate

# Checks:
# - JSONL parseable, no corruption
# - All transactions committed or rolled back
# - All referenced sessions exist
# - All triggers reference valid sessions
# - No orphaned sessions (parent doesn't know about them)
```

### Failure Notification

When ken detects problems, it can write to a notification log:

```jsonl
{"ts":"...","level":"warning","msg":"Session ses-006 active for 47m","session":"ses-006","action":"investigate"}
{"ts":"...","level":"error","msg":"Daemon crashed","pid":12345,"action":"restart"}
{"ts":"...","level":"info","msg":"Trigger satisfied","session":"ses-001","trigger":"all_complete"}
```

A monitoring agent (or the next agent to wake) can check this log and take action.

---

## Reflection and Evolution

### Reflection Collection

After session completes:

```bash
ken reflect <session-id>
```

Prompts the agent (or captures from session output):
- How well did the kenning prepare you?
- What was unclear?
- What did you discover?
- Suggested improvements?

Stored in:
```
reflections/{ken-path}/{timestamp}.md
```

### Kenning Evolution (Future)

```
1. Collect reflections for a ken
2. Analyze patterns (common gaps, common discoveries)
3. Generate proposed kenning revision
4. A/B test: some sessions get current, some get proposed
5. Measure which produces better work
6. Promote winner
```

This is Phase 2+. MVP focuses on core session/checkpoint/trigger mechanics.

---

## Open Questions

### Answered by This Design

- **Who spawns agents?** → Ken CLI, invoked by agents or humans
- **How does orchestrator know work is done?** → Triggers fire, sessions resume
- **Is there an orchestrator?** → No, all agents are uniform
- **How do we handle context limits?** → Sleep/wake chains with checkpoints
- **What about crashes?** → Checkpoint enables recovery

### Remaining Questions

1. **Kenning frame count/length** — What's optimal? Varies by domain?

2. **Checkpoint frequency** — Too frequent = overhead. Too rare = lost work.

3. **Concurrent session limit** — How many active sessions at once?

4. **Resource management** — API costs, rate limits, parallelism

5. **Human intervention points** — When should humans be notified/involved?

6. **Cross-project learning** — Do kennings in one project inform another?

7. **Versioning** — How do we version kennings, roll back bad changes?

8. **Testing kennings** — How do we validate that a kenning produces good understanding?

---

## Implementation Phases

### Phase 1: Core Mechanics
- Session create/read/update/complete
- Checkpoint write/read
- Trigger register/evaluate
- Atomic spawn-batch
- Basic ken CLI
- Integration with Claude Code

### Phase 2: Robustness
- Recovery from checkpoint
- Event logging
- Timeout triggers
- Better error handling
- Monitoring/observability

### Phase 3: Evolution
- Reflection collection
- Reflection analysis
- Kenning improvement proposals
- A/B testing framework

### Phase 4: Scale
- Concurrent session management
- Resource optimization
- Cross-project patterns
- Advanced trigger conditions

---

## Appendix: Example Session Tree

```
Human: "Build the ken system"
          │
          ▼
    session-ROOT
    ken: project/root
    status: sleeping
    trigger: all_complete[A, B, C]
          │
          ├─────────────────┬─────────────────┐
          ▼                 ▼                 ▼
    session-A          session-B          session-C
    ken: core/cli      ken: core/state    ken: core/triggers
    status: complete   status: sleeping   status: complete
                       trigger: all[D,E]
                             │
                       ┌─────┴─────┐
                       ▼           ▼
                  session-D   session-E
                  ken: ...    ken: ...
                  status:     status:
                  complete    active
```

When session-E completes:
1. `ken check` evaluates session-B's trigger → satisfied
2. session-B moves to `ready`
3. `ken process` wakes session-B with D+E results
4. session-B completes
5. `ken check` evaluates session-ROOT's trigger → satisfied
6. session-ROOT wakes with A+B+C results
7. session-ROOT completes
8. Human receives final result
