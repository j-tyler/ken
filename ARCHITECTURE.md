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

Session state is stored in SQLite for durability — see Storage Model below.

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
│                    │  SQLite   │                                 │
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
1. **State management** — All session state lives in ken's SQLite database
2. **Trigger evaluation** — Continuously checks if sleeping sessions should wake
3. **Agent spawning** — Starts Claude Code instances with proper context
4. **Request handling** — Processes spawn/checkpoint/complete/sleep from agents
5. **Atomicity** — Ensures multi-step operations succeed or fail completely

**Agents communicate only through ken:**
- Agent wants to spawn children → sends request to ken
- Agent completes → tells ken
- Agent needs child results → ken provides them at wake time

### Storage Model: SQLite

All mutable state uses **SQLite** for durability.

**Why SQLite:**
- ACID transactions (spawn_and_sleep is one atomic transaction)
- Built-in indexes (query by status instantly)
- No state replay needed (current state always queryable)
- Concurrent access handled properly
- Crash recovery via WAL mode
- Battle-tested, well-supported in Rust (rusqlite)

**Schema:**

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    ken TEXT NOT NULL,
    task TEXT NOT NULL,
    status TEXT NOT NULL,  -- pending, active, sleeping, complete, failed
    parent_id TEXT,
    trigger TEXT,          -- JSON, null unless sleeping
    checkpoint TEXT,       -- null unless sleeping/resuming
    result TEXT,           -- null unless complete
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES sessions(id)
);

CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_parent ON sessions(parent_id);

CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL,
    session_id TEXT,
    event_type TEXT NOT NULL,
    data TEXT,  -- JSON
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE INDEX idx_events_session ON events(session_id);
CREATE INDEX idx_events_ts ON events(ts);
```

**Atomic spawn_and_sleep:**
```sql
BEGIN;
INSERT INTO sessions (id, ken, task, status, parent_id, ...) VALUES (...);  -- child 1
INSERT INTO sessions (id, ken, task, status, parent_id, ...) VALUES (...);  -- child 2
UPDATE sessions SET status = 'sleeping', trigger = ?, checkpoint = ? WHERE id = ?;  -- parent
INSERT INTO events (ts, session_id, event_type, data) VALUES (...);  -- log event
COMMIT;
```

Either all succeed or nothing changes. No custom transaction logic needed.

### Directory Structure

```
project/
  .ken/
    ken.db                    # SQLite database (WAL mode)
    ken.db-wal                # Write-ahead log (automatic)
    ken.db-shm                # Shared memory (automatic)

  kens/
    {path}/
      kenning.md              # reconstruction sequence (human-authored)

  work/                       # actual code/artifacts
```

Simplified: one database file, kennings in markdown.

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

Checkpoints are stored in the sessions table `checkpoint` column. Updated whenever an agent calls `spawn_and_sleep` or `sleep`.

---

## Ken Interface

### Phase 1 MVP Commands

The MVP has **5 commands** — enough to run the manual workflow loop:

```bash
ken init                          # Create .ken/ken.db, initialize schema
ken wake <ken> --task "..."       # Create session, spawn agent
ken request '<json>'              # Agent → ken communication
ken process                       # Evaluate triggers + spawn one pending
ken status                        # Show what's happening
```

**The manual workflow loop:**
```bash
ken wake project/root --task "build ken"   # Start work

# Keep running until done:
ken process                                 # Check triggers, spawn pending
ken process
ken process
ken status                                  # See what's happening
```

### Agent Request Types

Agents call `ken request '<json>'` with **3 request types**:

```json
// Complete - mark session done
{"type":"complete","session_id":"abc123","result":"...what I produced..."}

// Spawn children + sleep with trigger (atomic)
{"type":"spawn_and_sleep","session_id":"abc123","children":[
  {"ken":"core/cli","task":"build parser"},
  {"ken":"core/state","task":"implement persistence"}
],"trigger":{"all_complete":"__CHILDREN__"},"checkpoint":"...my context..."}

// Sleep without spawning (wait for timeout, etc.)
{"type":"sleep","session_id":"abc123","trigger":{"timeout_seconds":3600},"checkpoint":"..."}

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

**Ken processes atomically (SQLite transaction):**
```sql
BEGIN;
-- Generate IDs and create children
INSERT INTO sessions (id, ken, task, status, parent_id, created_at, updated_at)
  VALUES ('child-1', 'core/cli', 'Build argument parser', 'pending', 'abc123', ...);
INSERT INTO sessions (id, ken, task, status, parent_id, created_at, updated_at)
  VALUES ('child-2', 'core/state', 'Implement persistence', 'pending', 'abc123', ...);
-- Update parent
UPDATE sessions
  SET status = 'sleeping',
      trigger = '{"all_complete":["child-1","child-2"]}',
      checkpoint = '## Context\n...',
      updated_at = ...
  WHERE id = 'abc123';
-- Log event
INSERT INTO events (ts, session_id, event_type, data) VALUES (...);
COMMIT;
```

Either all succeed or nothing changes. SQLite guarantees this.

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
# Via CLI
ken request '{"type":"complete","session_id":"abc123","result":"Implemented storage layer"}'

# Response comes on stdout as JSON
{"ok":true}
```

The agent (Claude) constructs JSON requests and parses responses.

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
- complete: Finish your session with a result
- spawn_and_sleep: Create children, sleep until they complete
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
│  1. Load session from SQLite                                     │
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
│       • spawn_and_sleep (create children, wait)                  │
│       • complete (done, here's result)                           │
│       • sleep (wait for trigger)                                 │
└─────────────────────────────────────────────────────────────────┘
      │
      │ Request received by ken
      ▼
┌─────────────────────────────────────────────────────────────────┐
│   KEN                                                            │
│                                                                  │
│   - Processes request in SQLite transaction                      │
│   - If spawn_and_sleep: creates children, parent sleeps          │
│   - If complete: marks session complete                          │
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

If atomic operation (spawn_and_sleep) fails partway:
1. SQLite transaction is rolled back automatically
2. No partial state exists
3. Parent remains active, can retry

---

## Event Log

Events are stored in the `events` table for debugging and observability.

```sql
SELECT ts, session_id, event_type, data FROM events ORDER BY ts DESC LIMIT 20;
```

Example events:
- `session_created` — new session
- `agent_spawned` — Claude instance started
- `children_spawned` — spawn_and_sleep executed
- `session_sleeping` — waiting for trigger
- `trigger_satisfied` — children completed
- `session_complete` — work done

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
│   ├── state/sqlite (ses-005) [complete] ✓ 12m
│   │   ├── done: "SQLite transactions work atomically"
│   │   └── result: "Implemented with WAL mode"
│   │
│   └── state/queries (ses-006) [active] ⚡ 47m ← LONG
│       ├── done: "Session queries work correctly"
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
          "spawn_and_sleep creates children atomically in SQLite",
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

**Stored in sessions table** with done_when as JSON column.

**Delivered to agent at wake:**
```markdown
## Definition of Done

You are done when:
- CLI handles all request types

Specific criteria:
1. spawn_and_sleep creates children atomically
2. complete marks session done and writes result
3. sleep registers trigger correctly
4. Invalid JSON returns {ok:false, error:...}

To verify: Run test suite: cargo test

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
# Ken: state/queries
# Status: active
# Parent: ses-003
# Started: 2026-02-01T10:00:00Z (47m ago)
#
# Task: Implement session queries
#
# Done When:
#   Session queries work correctly
#   - [?] query_by_status() returns correct sessions
#   - [?] query_children() returns child sessions
#   - [?] Handles empty results gracefully
#
# Last checkpoint (43m ago):
#   "Hit concurrent write issue, investigating"
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
#   ✓ ken.db: 847 sessions, 1203 events
#   ✓ Database integrity OK
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

**Validate integrity:**
```bash
ken validate

# Checks:
# - SQLite database integrity (PRAGMA integrity_check)
# - All referenced sessions exist
# - All triggers reference valid sessions
# - No orphaned sessions (parent doesn't know about them)
```

### Failure Notification

When ken detects problems, it writes to the events table:

```sql
SELECT * FROM events WHERE event_type = 'warning' OR event_type = 'error';
```

A monitoring agent (or the next agent to wake) can check this and take action.

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
