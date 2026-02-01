# Ken: Implementation Plan

## Overview

This document covers the concrete technical implementation of ken. For the philosophical foundation and deep intent, see FOUNDATION.md — read that first.

---

## Critical Pre-Work: Claude Code Integration Research

**BEFORE WRITING CODE**, investigate:

1. How does Claude Code accept input in non-interactive mode?
2. Can we pipe prompts to it? Or does it need a different IPC mechanism?
3. How do we detect when a response is complete?
4. Can we inject prompts mid-session (for frames, then task, then reflection)?
5. Does Claude Code have an API or SDK we should use instead of CLI wrapping?

The entire architecture depends on being able to:
- Send a prompt
- Receive a complete response  
- Send another prompt (in same session/context)
- Repeat

If Claude Code doesn't support this, we need a different approach (perhaps direct API calls to Claude, managing our own context).

**Fallback option:** If Claude Code integration is complex, MVP could use direct Anthropic API calls. We manage context ourselves. Less elegant but more controllable.

---

## Technology Choice

**Recommended: Rust**

Reasons:
- Single binary distribution (no runtime dependencies)
- Strong CLI ecosystem (clap for args, indicatif for progress)
- Good process spawning and management
- Fast startup time (matters for frequent CLI invocations)
- Error handling model fits the domain

Alternative: Go (similar benefits, slightly easier learning curve)

**Also acceptable for MVP: Python**

If agent integration proves complex, Python offers:
- Faster iteration during research phase
- anthropic SDK is Python-native
- Can rewrite in Rust once design stabilizes

Avoid: Node (async complexity not worth it here)

---

## Project Structure (MVP)

Simplified for MVP. Evolution features added later.

```
project/
├── ken.yaml              # Project config
├── kens/                 # Ken definitions
│   └── {path}/
│       ├── kenning.md    # The reconstruction sequence
│       └── meta.yaml     # Parent, version, stats
├── reflections/          # Post-session reflections  
│   └── {path}/
│       └── {timestamp}.md
└── .ken/                 # Internal state (gitignore this)
    └── session.json      # Active session state (crash recovery)
```

**Intentionally omitted from MVP:**
- `interface.md` — Adds complexity without core value yet
- `history/` — Version history can wait; just increment version in meta.yaml
- `work/` — Code output goes wherever the project keeps code, not our concern

---

## Project Structure (Full Vision)

For reference, the complete structure we're building toward:

```
project/
├── ken.yaml              # Project config
├── kens/                 # Ken definitions
│   └── {path}/
│       ├── kenning.md    # The reconstruction sequence
│       ├── interface.md  # What this ken exposes (Phase 2)
│       └── meta.yaml     # Parent, peers, version, stats
├── reflections/          # Post-session reflections
│   └── {path}/
│       └── {timestamp}.md
├── history/              # Kenning version history (Phase 2)
│   └── {path}/
│       └── v{n}.md
└── .ken/                 # Internal state
    ├── session.json      # Active session state
    └── evolution/        # A/B test state (Phase 2)
```

---

## Data Structures

### Project Configuration (ken.yaml)

```yaml
name: "x86-kernel"
version: "0.1.0"
created: "2026-02-01T14:30:00Z"

# Default AI agent configuration
agent:
  type: "claude-code"
  model: "claude-sonnet-4-20250514"
  
# Project-level settings
settings:
  reflection_required: true
  min_frames: 3
  max_frames: 12
```

### Ken Metadata (kens/{path}/meta.yaml)

```yaml
name: "memory-management"
created: "2026-02-01T14:30:00Z"
updated: "2026-02-01T16:45:00Z"

# Hierarchy
parent: "kernel/core"
children:
  - "kernel/memory/physical"
  - "kernel/memory/virtual"
  - "kernel/memory/paging"
peers:
  - "kernel/interrupts"
  - "kernel/scheduler"

# Kenning version
kenning_version: 3

# Statistics
sessions: 47
last_session: "2026-02-01T16:45:00Z"
```

### Kenning Structure (Internal Representation)

```rust
pub struct Kenning {
    pub ken_path: String,
    pub version: u32,
    pub frames: Vec<Frame>,
    pub metadata: KenningMetadata,
}

pub struct Frame {
    pub number: u32,
    pub title: String,
    pub prompt: String,
    pub frame_type: FrameType,
}

pub enum FrameType {
    Orientation,    // Sets initial context
    Generative,     // Prompts for insight generation
    Structural,     // Asks for diagrams/structures
    Critical,       // Challenges understanding
    Integrative,    // Connects to other kens
    Grounding,      // Current state, specific context
}

pub struct KenningMetadata {
    pub parent: Option<String>,
    pub peers: Vec<String>,
}
```

### Session State

```rust
pub struct Session {
    pub id: String,
    pub ken_path: String,
    pub kenning_version: u32,
    pub task: Option<String>,
    pub started: DateTime<Utc>,
    pub frames_completed: Vec<CompletedFrame>,
    pub status: SessionStatus,
}

pub struct CompletedFrame {
    pub frame_number: u32,
    pub prompt_sent: String,
    pub response_received: String,
    pub completed_at: DateTime<Utc>,
}

pub enum SessionStatus {
    WakingUp,       // Walking through frames
    Ready,          // Frames complete, ready for work
    Working,        // Task in progress
    Reflecting,     // Writing reflection
    Complete,       // Session ended
}
```

### Reflection

```rust
pub struct Reflection {
    pub ken_path: String,
    pub timestamp: DateTime<Utc>,
    pub kenning_version: u32,
    pub task: String,
    pub preparation_assessment: String,
    pub clarity: String,
    pub gaps: String,
    pub discoveries: String,
    pub proposed_changes: String,
}
```

---

## Command Implementation Details

### `ken init {project-name}`

```
1. Create directory: ./{project-name}/
2. Create subdirectories: kens/, reflections/, history/, work/
3. Generate ken.yaml with defaults
4. Create root ken (kens/root/):
   - kenning.md (minimal starter)
   - meta.yaml
   - interface.md
5. Print success message with next steps
```

### `ken new {path}`

```
Arguments:
  path         Ken path (e.g., "kernel/memory")
  --parent     Parent ken path (default: inferred from path)
  --peers      Comma-separated peer paths

Process:
1. Validate path doesn't exist
2. Resolve parent (explicit or from path hierarchy)
3. Create directory: kens/{path}/
4. Generate meta.yaml
5. Generate starter kenning.md:
   - Frame 1: Orientation (templated)
   - Frame 2: Empty generative frame
   - Frame 3: Grounding (templated)
6. Update parent's children list (if parent exists)
7. Print success, remind to edit kenning
```

### `ken wake {path} --task "..."`

This is the core command. Detailed flow:

```
1. Load project config (ken.yaml)
2. Load ken metadata (kens/{path}/meta.yaml)
3. Parse kenning (kens/{path}/kenning.md)
4. Load interface (kens/{path}/interface.md)
5. Initialize session state

6. Start AI agent:
   - Spawn claude-code process in chat mode
   - Establish communication channel (stdin/stdout)

7. Walk through frames:
   for each frame in kenning.frames:
     a. Prepare prompt:
        - Frame prompt text
        - If grounding frame: inject current file state, interface info
     b. Send to agent
     c. Wait for response
     d. Store in session.frames_completed
     e. Update session status

8. If --task provided:
   a. Send task prompt
   b. Agent enters working mode
   c. Agent can: read/write files, run commands, access codebase
   d. Wait for work completion signal

9. Reflection phase:
   a. Send reflection prompt
   b. Capture reflection response
   c. Parse into Reflection struct
   d. Save to reflections/{path}/{timestamp}.md

10. End session:
    a. Update ken metadata (session count, last session)
    b. Close agent process
    c. Return results
```

### `ken up` / `ken down` / `ken peers`

These work within an active session:

```
ken up:
1. Check session is active
2. Load parent ken's interface.md
3. Load parent ken's kenning.md (just Frame 1 for orientation)
4. Display summary: why this ken exists in context of parent

ken down:
1. Check session is active
2. Load all children kens' interface.md files
3. Display summary: what depends on this ken

ken peers:
1. Check session is active
2. Load peer kens' interface.md files
3. Display summary: who shares interfaces with this ken
```

### `ken reflect`

Can be called explicitly during session:

```
1. Check session is active and in Working status
2. Send reflection prompt to agent
3. Capture response
4. Parse and validate reflection format
5. Save to reflections/{path}/{timestamp}.md
6. Update session status
```

### `ken sleep`

```
1. Check session is active
2. If reflection not yet written, prompt for it
3. Gracefully terminate agent process
4. Update session status to Complete
5. Update ken metadata
6. Clear session state
```

---

## Claude Code Integration

### Spawning

```rust
use std::process::{Command, Stdio};

pub fn spawn_claude_code(working_dir: &Path) -> Result<AgentProcess> {
    let child = Command::new("claude")
        .arg("--chat")           // Interactive mode
        .arg("--no-permissions") // We handle permissions
        .current_dir(working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    Ok(AgentProcess::new(child))
}
```

### Communication Protocol

```rust
pub struct AgentProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl AgentProcess {
    pub async fn send_prompt(&mut self, prompt: &str) -> Result<String> {
        // Send prompt
        writeln!(self.stdin, "{}", prompt)?;
        writeln!(self.stdin, "---END_PROMPT---")?;
        self.stdin.flush()?;
        
        // Read response until delimiter
        let mut response = String::new();
        loop {
            let mut line = String::new();
            self.stdout.read_line(&mut line)?;
            if line.trim() == "---END_RESPONSE---" {
                break;
            }
            response.push_str(&line);
        }
        
        Ok(response)
    }
    
    pub fn terminate(&mut self) -> Result<()> {
        self.child.kill()?;
        Ok(())
    }
}
```

Note: Actual Claude Code integration may require different IPC mechanism. This is a starting sketch — will need adjustment based on Claude Code's actual API.

---

## Phase 1 Milestone Checklist

### Week 1: Scaffold
- [ ] Initialize Rust project with dependencies
- [ ] Implement CLI argument parsing (clap)
- [ ] Implement `ken init`
- [ ] Implement `ken create`
- [ ] Implement `ken tree`
- [ ] Write unit tests for project structure

### Week 2: Kenning System
- [ ] Implement kenning parser (markdown → structs)
- [ ] Implement kenning writer (structs → markdown)
- [ ] Implement meta.yaml handling
- [ ] Implement interface.md handling
- [ ] Write unit tests for parsing

### Week 3: Agent Integration
- [ ] Research Claude Code's actual IPC mechanism
- [ ] Implement agent spawning
- [ ] Implement prompt/response protocol
- [ ] Implement session state management
- [ ] Test basic communication

### Week 4: Wake Cycle
- [ ] Implement frame walking logic
- [ ] Implement dynamic content injection (grounding frames)
- [ ] Implement task injection
- [ ] Implement `ken wake` end-to-end
- [ ] Implement `ken reflect`
- [ ] Implement `ken sleep`
- [ ] Integration testing

### Week 5: Navigation & Polish
- [ ] Implement `ken up`
- [ ] Implement `ken down`
- [ ] Implement `ken peers`
- [ ] Error handling cleanup
- [ ] User-facing messages and help text
- [ ] Documentation

---

## Testing Strategy

### Unit Tests
- Project structure creation/validation
- Kenning parsing (valid and invalid inputs)
- Reflection parsing
- Path manipulation utilities

### Integration Tests
- Full `init` → `create` → `wake` → `reflect` → `sleep` cycle
- Multi-ken projects with hierarchy
- Error recovery (agent crash, malformed input)

### Dogfood Tests
- Use ken to build ken (Phase 2)
- Track what works and what doesn't
- Generate real reflections

---

## Future Considerations (Not Phase 1)

### Evolution System
- Reflection aggregation across sessions
- Pattern detection in gaps/discoveries
- Kenning mutation generation
- A/B test orchestration
- Statistical comparison of test results
- Version promotion logic

### Scaling
- Parallel session orchestration
- Cross-ken dependency validation
- Resource management (how many agents at once?)
- Caching and performance

### Alternative Agents
- Support for other AI coding tools
- Agent-agnostic protocol definition
- Adapter pattern for different backends

---

## Getting Started

```bash
# Clone and build
git clone <repo>
cd ken
cargo build --release

# Initialize a test project
./target/release/ken init test-project
cd test-project

# Create some kens
ken new core
ken new core/utils --parent core
ken new core/config --parent core --peers core/utils

# View structure
ken tree

# Edit a kenning
ken edit core/utils

# Wake into a ken with a task (task is required)
ken wake core/utils --task "implement a logging utility"

# When done, reflect ends the session
ken reflect
```

**Note:** See DESIGN_REVIEW.md for command naming rationale. Key changes from early design:
- `ken new` (not `create`) — more natural phrasing
- `ken reflect` ends the session — no separate `sleep` command
- Navigation is `ken context [up|down|peers]` (Phase 2)

---

## Notes for Implementing Instance

You're about to build this. Some guidance:

1. **Start with the data structures.** Get the types right. The rest flows from there.

2. **Mock the agent first.** Before integrating Claude Code, build against a mock agent that just echoes prompts. Get the flow right.

3. **The kenning parser is fiddly.** Markdown is ambiguous. Be defensive. Handle edge cases. Consider using a proper markdown parsing library.

4. **Session state must be persistent.** If ken crashes mid-session, we need to recover. Consider using a state file.

5. **Logging is your friend.** When debugging agent communication, you'll want to see every prompt sent and response received.

6. **Error messages matter.** A confused user is a lost user. Every error should say what went wrong and what to do about it.

This is the foundation of something important. Build it carefully.
