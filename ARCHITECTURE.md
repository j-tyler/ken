# Ken System Architecture

## Overview

Ken is a durable workflow system for AI agent self-orchestration. It solves the continuity problem: AI instances have limited context, sessions end, and understanding dissolves. Ken provides the infrastructure for agents to maintain continuity across context boundaries through structured understanding (kens), reconstruction sequences (kennings), checkpointed state, and wake triggers.

**Core insight**: There is no distinction between "orchestrator" and "agent". All are instances of the same AI, differentiated only by which ken they wake into. Any agent can spawn other agents. The system is recursive and self-similar at every level.

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

```
.ken/sessions/
  {session-id}/
    meta.yaml        # ken, status, parent, children
    checkpoint.md    # current state (for recovery)
    trigger.yaml     # wake condition (if sleeping)
    result.md        # output (when complete)
```

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

### Directory Structure

```
project/
  .ken/
    config.yaml              # project-level configuration
    sessions/
      {session-id}/
        meta.yaml            # ken, status, parent_session, children
        checkpoint.md        # durable state for recovery
        trigger.yaml         # wake conditions (if sleeping)
        result.md            # output (when complete)
        events.yaml          # append-only event log (optional)
    pending/
      {session-id}.yaml      # sessions ready to wake (trigger satisfied)

  kens/
    {path}/
      kenning.md             # reconstruction sequence
      interface.md           # what this ken exposes/consumes
      meta.yaml              # parent, peers, version

  reflections/
    {path}/
      {timestamp}.md         # accumulated feedback

  work/                      # actual code/artifacts
```

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

```
.ken/sessions/{session-id}/
  checkpoint.md           # latest checkpoint (overwritten)
  checkpoint-history/     # optional: historical checkpoints
    {timestamp}.md
```

For MVP: single checkpoint file, overwritten.
Later: checkpoint history for debugging/replay.

---

## CLI Interface

### Commands

```bash
# Project
ken init                          # Initialize .ken structure
ken status                        # Show project status, active sessions

# Ken Management
ken create <path>                 # Create new ken
ken edit <path>                   # Edit kenning
ken tree                          # Show ken hierarchy
ken interface <path>              # Edit interface definition

# Session Operations (typically called by agents, not humans)
ken wake <ken-path> \             # Spawn a new session
  --task "description" \
  --parent <session-id> \
  --checkpoint-file <path>

ken spawn-batch \                 # Atomic: spawn multiple + trigger
  --children <child-specs.yaml> \
  --trigger <trigger-spec.yaml> \
  --checkpoint-file <path>

ken checkpoint \                  # Write checkpoint for current session
  --session <session-id> \
  --content-file <path>

ken complete \                    # Mark session complete
  --session <session-id> \
  --result-file <path>

ken sleep \                       # Register trigger, go dormant
  --session <session-id> \
  --trigger <trigger-spec.yaml> \
  --checkpoint-file <path>

# Trigger Processing
ken check                         # Evaluate triggers, move ready sessions
ken process                       # Wake one ready session
ken daemon                        # Continuous check + process loop

# Introspection
ken sessions                      # List all sessions
ken session <id>                  # Show session details
ken pending                       # Show sessions ready to wake
ken tree --sessions               # Show session hierarchy
ken log <session-id>              # Show session event log

# Reflection
ken reflect <session-id>          # Record reflection for completed session
ken reflections <ken-path>        # Show reflections for a ken
ken improve <ken-path>            # Propose kenning improvements from reflections
```

### Atomic Spawn-Batch

The critical atomic operation:

```bash
ken spawn-batch \
  --parent-session abc123 \
  --children children.yaml \
  --parent-trigger trigger.yaml \
  --parent-checkpoint checkpoint.md
```

**children.yaml:**
```yaml
- ken: core/cli
  task: "Build argument parser"

- ken: core/state
  task: "Implement session persistence"

- ken: core/triggers
  task: "Implement wake trigger evaluation"
```

**trigger.yaml:**
```yaml
wake_when:
  all_complete: [__CHILD_0__, __CHILD_1__, __CHILD_2__]
```

**Behavior:**
1. Begin transaction
2. Create child session records (get IDs)
3. Substitute `__CHILD_N__` with actual IDs in trigger
4. Write parent checkpoint with children listed
5. Write parent trigger
6. Update parent status to `sleeping`
7. Commit transaction (atomic rename)
8. Add children to `pending/` for spawning

If any step fails: rollback, parent continues as active (can retry).

---

## Integration with Claude Code

### Spawning Sessions

From within a Claude Code session (agent), spawn children using:

```python
# Option A: Shell out to ken CLI
subprocess.run([
    'ken', 'spawn-batch',
    '--parent-session', my_session_id,
    '--children', 'children.yaml',
    '--parent-trigger', 'trigger.yaml',
    '--parent-checkpoint', 'checkpoint.md'
])

# Option B: Task tool (if ken provides MCP integration)
# The agent uses Task tool, ken intercepts/wraps
```

### Session Context

When an agent wakes, it receives:

```markdown
# Session Context

## Session ID
{session-id}

## Ken
{ken-path}

## Kenning
[The kenning frames were already walked — understanding is constructed]

## Task
{task description}

## Parent Context (if resuming/recovering)
{checkpoint content from before sleep}

## Child Results (if children completed)
### Child: {child-id} ({child-ken})
{child result content}

### Child: {child-id} ({child-ken})
{child result content}

## Available Commands
- ken checkpoint --session {session-id} --content-file <path>
- ken spawn-batch --parent-session {session-id} ...
- ken complete --session {session-id} --result-file <path>
- ken sleep --session {session-id} --trigger <spec> --checkpoint-file <path>
```

### Process Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      EXECUTION FLOW                              │
└─────────────────────────────────────────────────────────────────┘

Human or Agent
      │
      │ ken wake core/foo --task "do something"
      ▼
┌─────────────┐
│  ken CLI    │
│             │
│ 1. Create session record
│ 2. Load kenning
│ 3. Spawn Claude Code instance
│ 4. Feed kenning frames
│ 5. Deliver task + context
└─────┬───────┘
      │
      ▼
┌─────────────┐
│   Agent     │
│             │
│ - Works on task
│ - May call ken checkpoint
│ - May call ken spawn-batch
│ - Calls ken complete OR ken sleep
└─────┬───────┘
      │
      ▼
┌─────────────┐         ┌─────────────┐
│  Complete   │   OR    │  Sleeping   │
│             │         │             │
│ Result      │         │ Waiting for │
│ stored      │         │ trigger     │
└─────────────┘         └──────┬──────┘
                               │
                               │ ken check (periodic)
                               │ trigger satisfied
                               ▼
                        ┌─────────────┐
                        │   Ready     │
                        │             │
                        │ In pending/ │
                        │ queue       │
                        └──────┬──────┘
                               │
                               │ ken process
                               ▼
                        ┌─────────────┐
                        │   Agent     │
                        │  (resumed)  │
                        │             │
                        │ Continues   │
                        │ with child  │
                        │ results     │
                        └─────────────┘
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

## Event Log (Optional)

For debugging and replay, sessions can maintain an event log:

```yaml
# events.yaml
- ts: "2026-02-01T10:00:00Z"
  type: wake
  kenning_version: 3

- ts: "2026-02-01T10:05:00Z"
  type: checkpoint
  summary: "Reviewed interfaces"

- ts: "2026-02-01T10:10:00Z"
  type: spawn
  children:
    - id: session-B
      ken: core/cli
    - id: session-C
      ken: core/state

- ts: "2026-02-01T10:10:01Z"
  type: sleep
  trigger: {all_complete: [session-B, session-C]}

- ts: "2026-02-01T11:30:00Z"
  type: wake
  reason: trigger_satisfied
  mode: resume  # or 'recover'

- ts: "2026-02-01T11:45:00Z"
  type: complete
  result_summary: "Integration complete, tests passing"
```

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
