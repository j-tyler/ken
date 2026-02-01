# Ken: Design Review and Refinement

## Critical Analysis

Taking a hard look at what we've designed. What's essential? What's premature? What's poorly named?

---

## Command Naming Review

Let me examine each command from the perspective of a fresh instance seeing it for the first time:

### Clear and Good

| Command | Verdict | Notes |
|---------|---------|-------|
| `ken init` | ✓ Keep | Universal convention. Instantly understood. |
| `ken tree` | ✓ Keep | Shows structure. Clear metaphor. |
| `ken reflect` | ✓ Keep | You're writing a reflection. Obvious. |

### Questionable

| Command | Issue | Alternatives |
|---------|-------|--------------|
| `ken create` | Creates what? A ken is understanding, not a folder. | `ken new`, `ken add`, or keep `create` with better docs |
| `ken wake` | Poetic but possibly confusing. Could sound like starting a daemon. | `ken enter`, `ken begin`, `ken start`, or keep `wake` (it's meaningful enough) |
| `ken sleep` | If reflection ends the session, is this needed? | Maybe reflection *is* sleep. Combine them. |
| `ken up/down/peers` | Directional but... subcommands of what? Orphaned feeling. | `ken look up`, `ken context up`, or `ken nav up` |
| `ken review` | Review what? Ambiguous. | `ken reflections`, `ken readings`, `ken journal` |
| `ken improve` | Improve what? The kenning? The ken? | `ken evolve`, `ken refine`, `ken tune` |
| `ken test` | Easily confused with running unit tests. | `ken trial`, `ken compare`, `ken evaluate` |
| `ken promote` | DevOps jargon. Not self-evident. | `ken accept`, `ken adopt`, `ken upgrade` |
| `ken history` | History of what? Versions? Sessions? | `ken versions`, `ken lineage`, `ken ancestry` |

### My Revised Recommendations

**Keep as-is:**
- `ken init`
- `ken tree`
- `ken wake` — the poetry matters, and it pairs with sleep
- `ken reflect`

**Change:**
- `ken create` → `ken new` — feels more natural: "create a new ken"
- `ken sleep` → merge into `ken reflect` — reflection *is* the ending. After you reflect, the session ends. No separate sleep command needed.
- `ken up/down/peers` → `ken context [up|down|peers]` — groups navigation under one command
- `ken review` → `ken journal` — you're reviewing the journal of reflections
- `ken improve` → `ken evolve` — captures the living, iterative nature
- `ken test` → `ken trial` — running a trial of the new kenning
- `ken promote` → `ken adopt` — you're adopting the trialed version
- `ken history` → `ken lineage` — shows the ancestry of a kenning

**Revised command set:**

```bash
# Project setup
ken init {project}
ken new {path}                    # was: create
ken tree

# Session lifecycle  
ken wake {path} --task "..."
ken context [up|down|peers]       # was: up/down/peers as separate commands
ken reflect                       # ends session (was: reflect + sleep)

# Evolution (Phase 2+)
ken journal {path}                # was: review
ken evolve {path}                 # was: improve
ken trial {path}                  # was: test
ken adopt {path}                  # was: promote
ken lineage {path}                # was: history
```

---

## What's Truly Needed for v0.1

The core hypothesis: **Kennings actually work. Walking through generative frames produces better understanding than reading documentation.**

To test this, we need:

### Minimum Viable Ken

```bash
ken init        # Create project structure
ken new         # Create a ken with starter kenning
ken tree        # See what exists
ken wake        # Walk frames, do task
ken reflect     # Write reflection, end session
```

That's it. Five commands.

Everything else is optimization for *after* we prove the core idea works.

### What We're Explicitly Deferring

**Evolution system** (evolve, trial, adopt, lineage)
- Requires: reflection aggregation, mutation generation, A/B orchestration, scoring
- Can be done manually at first: read reflections yourself, edit kennings by hand
- Build this after we have real reflections to learn from

**Navigation** (context up/down/peers)
- Nice to have, not essential
- Instance can just... read the meta.yaml if it needs orientation
- Build this when we feel the pain of not having it

**Multi-agent orchestration**
- Way premature
- We don't even know if single-agent kennings work yet

**Interface validation**
- Checking that kens honor their interface contracts
- Important eventually, not needed to test the core idea

---

## What We Must Be Aware Of (Even If Building Later)

These decisions now affect what we can do later:

### 1. Session State Persistence

If `ken wake` gets interrupted (crash, network, ctrl-c), what happens?

**Decision needed:** Do we persist session state to disk?

Options:
- Yes: Can resume interrupted sessions. More complex.
- No: Lost work. Simpler.

**Recommendation:** Yes, persist. Save a `.ken-session` file. Even for v0.1, losing a session to a crash would be painful. Basic persistence is worth the complexity.

### 2. Kenning Format Stability

The markdown format of kennings will be parsed by code. If we change the format later, old kennings break.

**Decision needed:** How strict is the format?

**Recommendation:** Be loose in what we accept, strict in what we produce. Use clear section headers (`## Frame 1:`) but don't require rigid structure. The parser should be forgiving.

### 3. Claude Code Integration Uncertainty

We don't actually know Claude Code's IPC mechanism. We're designing blind.

**Decision needed:** How do we abstract the agent interface?

**Recommendation:** Define a trait/interface for agents early:

```rust
trait Agent {
    fn send_prompt(&mut self, prompt: &str) -> Result<String>;
    fn is_alive(&self) -> bool;
    fn terminate(&mut self) -> Result<()>;
}
```

Then we can implement `MockAgent` for testing and `ClaudeCodeAgent` when we understand the real interface. If Claude Code doesn't work how we expect, we can implement other backends.

### 4. Reflection Structure

Reflections feed the evolution system. Their structure matters.

**Decision needed:** Freeform or structured?

**Recommendation:** Semi-structured. Prompt with clear sections but don't parse rigidly:

```markdown
## Preparation Assessment
[freeform]

## Gaps
[freeform]

## Discoveries
[freeform]

## Suggestions
[freeform]
```

This gives us enough structure to analyze patterns without being brittle.

### 5. Ken Path Convention

Ken paths like `kernel/memory/paging` need consistent handling.

**Decisions needed:**
- Separator: `/` always, regardless of OS? **Yes.**
- Case sensitivity: `Kernel` vs `kernel`? **Lowercase enforced.**
- Special characters: spaces, dashes, underscores? **Only alphanumeric and dash. No spaces, no underscores.**

Getting this wrong early causes pain forever.

### 6. The Grounding Frame Problem

Frame 5 in our example (Current Implementation State) needs dynamic content — actual file lists, recent changes, etc.

**Decision needed:** How does dynamic injection work?

Options:
- Template syntax in kenning: `{{files:src/}}` gets replaced at wake time
- Grounding frames are always generated, never static
- Hybrid: some static context, some injected

**Recommendation:** Template syntax. Keep it simple:
- `{{tree:path}}` — inject directory tree
- `{{file:path}}` — inject file contents
- `{{recent:n}}` — inject n most recent reflections summary
- `{{interface:path}}` — inject interface.md for a ken

Parser recognizes these, injects at wake time.

---

## Revised Phase 1 Scope

### Week 1-2: Foundation

```
ken init
ken new  
ken tree
```

Plus:
- Project structure (ken.yaml, directories)
- Ken metadata (meta.yaml parsing/writing)
- Kenning parsing (markdown → frames, handle templates)
- Path utilities (ken path ↔ filesystem path)

**Deliverable:** Can create and navigate a ken project structure.

### Week 3: Agent Abstraction

```rust
trait Agent { ... }
struct MockAgent { ... }
```

Plus:
- Session state struct
- Session persistence (.ken-session file)
- Basic frame-walking loop against MockAgent

**Deliverable:** Can simulate a wake cycle without real AI.

### Week 4: Real Integration

```
ken wake {path} --task "..."
```

Plus:
- Claude Code integration (or whatever works)
- Actual prompt/response flow
- Template injection for grounding frames

**Deliverable:** Can actually wake an AI instance into a ken.

### Week 5: Reflection & Polish

```
ken reflect
```

Plus:
- Reflection prompt injection
- Reflection file writing
- Session cleanup
- Error handling
- Help text and documentation

**Deliverable:** Complete v0.1. Can init → new → wake → work → reflect.

---

## Naming: Final Recommendations

```bash
# Core (v0.1)
ken init {project}              # Initialize project
ken new {path}                  # Create new ken
ken tree                        # Show structure
ken wake {path}                 # Wake into ken
ken wake {path} --task "..."    # Wake with task
ken reflect                     # Write reflection, end session

# Navigation (v0.2)
ken context up                  # Why does this ken exist?
ken context down                # What depends on this?
ken context peers               # Related kens

# Evolution (v0.3+)
ken journal {path}              # View reflections
ken evolve {path}               # Propose kenning improvement
ken trial {path}                # A/B test improvement
ken adopt {path}                # Accept improvement
ken lineage {path}              # View kenning history
```

The v0.1 commands should feel complete on their own. A user shouldn't feel like something's missing.

---

## Open Design Questions

### 1. Interactive vs Batch Wake

Current design: `ken wake` is interactive. Agent stays alive, human can interact.

Alternative: `ken wake --task "..." ` is batch. Agent wakes, does task, reflects, exits.

**Question:** Do we need both modes? Or is batch-only sufficient for v0.1?

**Leaning:** Batch-only for v0.1. Simpler. Interactive mode when we understand usage patterns better.

### 2. Where Does Work Output Go?

When an agent writes code, where does it go?
- In the project's `work/` directory?
- In a separate repo that ken knows about?
- Ken doesn't care — agent writes wherever?

**Leaning:** Ken doesn't care. The ken project structure is for kennings and reflections. Actual code lives wherever it lives. The kenning's grounding frame can reference external paths.

### 3. Multi-Ken Tasks

What if a task spans multiple kens?

Example: "Implement feature X" touches kernel/memory and kernel/scheduler.

**Leaning:** Out of scope for v0.1. For now, one task = one ken. Orchestrating multi-ken work is a coordination problem we solve later.

### 4. Kenning Authorship

Who writes the initial kenning for a new ken?

Options:
- Human writes it
- `ken new` generates a starter and human edits
- AI generates from codebase inspection

**Leaning:** `ken new` generates a minimal starter (3 template frames). Human edits. AI generation from codebase is a v2 feature.

---

## Summary of Changes from Original Design

| Original | Revised | Reason |
|----------|---------|--------|
| `ken create` | `ken new` | More natural phrasing |
| `ken sleep` | Merged into `ken reflect` | Redundant. Reflection ends session. |
| `ken up/down/peers` | `ken context [direction]` | Groups related commands |
| `ken review` | `ken journal` | Clearer what's being reviewed |
| `ken improve` | `ken evolve` | Captures living nature |
| `ken test` | `ken trial` | Avoids confusion with unit tests |
| `ken promote` | `ken adopt` | Clearer than DevOps jargon |
| `ken history` | `ken lineage` | More evocative, clearer meaning |
| 5-week timeline | Same, but scope tightened | Focus on proving core hypothesis |

---

## What Success Looks Like

After v0.1, we should be able to:

1. Create a ken project
2. Define several kens with hand-written kennings
3. Wake an AI instance into a ken
4. Watch it walk through frames, generating understanding
5. Give it a task, watch it work
6. Read the reflection it produces
7. *Feel* that the reflection shows deeper understanding than a cold-start would

If #7 is true, the core hypothesis is validated and we proceed.

If #7 is false, we learn why and iterate on the kenning format.

---

## Next Action

Update IMPLEMENTATION.md with these refinements before handing off to a building instance.
