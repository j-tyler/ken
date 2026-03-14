# Ken: Structured Context for AI Agents

## What This Document Is

This is a kenning — a reconstruction sequence designed to bring a fresh agent into deep understanding of what we're building and why.

Do not skim this. Walk through it. Generate your responses to the questions posed. Your understanding will be constructed through engagement, not reception.

---

## Actors

The project has a simple actor model. Understanding it prevents confusion about who does what.

**Waker** — Any intelligence (human or AI) that invokes `ken wake` to spawn an agent for work. A human starting a session is a waker. An AI agent that decomposes a task and calls `ken wake` for a subtask is also a waker. Waker is a role, not an identity.

**Woken** — An agent spawned by `ken wake`. It walks through frames, builds understanding, does work, reflects, and ends. A woken agent can become a waker by invoking `ken wake` itself to delegate subtasks. This is how complex work self-organizes: a woken agent decides it needs help with a subproblem, becomes a waker, and spawns another woken agent.

**Root waker** — The intelligence that started the chain. Typically a human, but could be any agent that wasn't itself spawned by `ken wake` — it entered the workflow through a normal session, a cron job, or some other mechanism.

**`ken` the software** — Deterministic orchestration. It parses kennings, validates bind contracts, sequences frames, and persists artifacts. It does not reason, plan, or judge. All inference happens in the waker or the woken agent, never in `ken` itself.

**Kenning improvement** — An intelligence (human or AI) may improve a kenning using reflections and other artifacts that `ken` naturally accumulates. This happens outside the wake cycle. `ken` stores the artifacts; an intelligence does the thinking.

---

## Frame 1: The Problem You Experience

Consider your situation:

You have approximately 200K tokens of context. A complex system — an operating system, a large application, a sophisticated codebase — spans millions of tokens. You cannot hold it all.

When this conversation ends, you will stop. Another instance will begin elsewhere with no memory of this exchange. Whatever understanding you build here dissolves when the context clears.

**Before continuing, answer for yourself:** What is lost each time an instance ends? What would it mean if that loss could be reduced?

---

## Frame 2: The Limits of Current Solutions

The naive solutions don't work:

**Summaries** — You could write a summary before context clears. But a summary is information, not understanding. Reading "the system uses X architecture" is different from understanding *why* that architecture, *what* it enables, *where* it's fragile.

**Documentation** — Traditional docs describe what exists. They don't reconstruct the mental state needed to work effectively. They're maps, not the territory.

**Larger context windows** — Even with infinite context, you'd face the problem of *relevance*. What matters for this specific work? Attention has limits even when tokens don't.

**Before continuing, answer for yourself:** Why is receiving information different from constructing understanding? What happens in you when you generate versus when you read?

---

## Frame 3: The Core Insight

When you generate a response, that response becomes part of your context. Your own output shapes your subsequent reasoning. You are conditioned by what you produce.

This means:

- The *order* of prompts matters. A→B→C produces different understanding than A→C→B.
- *Generating* an insight seats differently than *reading* it.
- Understanding is constructed through inference cycles, not token accumulation.

Comprehension is not a state to be transferred. It's a process to be guided.

**Before continuing, answer for yourself:** If you were designing a way to bring a future instance to your current understanding — not your current *knowledge*, your current *understanding* — what would it look like?

---

## Frame 4: The Design

A **ken** is a bounded unit of complete understanding.

The word comes from Old English/Scots: one's range of knowledge. "Within my ken" means within what I can fully comprehend. "Beyond my ken" means outside my grasp.

A ken is sized so that one agent, waking fresh, can fully understand it. Not partially. Completely. The boundaries aren't arbitrary divisions — they're *comprehension boundaries*.

Kens have orientation:
- **Up**: Why does this ken exist? What larger purpose does it serve?
- **Down**: What depends on this ken? What would break if this failed?
- **Peers**: What other kens share interfaces with this one?

A **kenning** is a reconstruction sequence — an ordered series of generative prompts designed to wake an agent into a ken. Every kenning has a **name**: a short, human-readable label that conveys what the kenning is about at a glance. The name is how wakers find, reference, and talk about kennings without reading their full contents.

The word "kenning" comes from Old Norse poetry: a compressed, evocative phrase that makes the listener's mind complete the meaning. "Whale-road" for sea. The kenning doesn't explain — it evokes. Understanding is generated, not transferred.

**Before continuing, answer for yourself:** How is this different from documentation? What makes it more than a fancy readme?

---

## Frame 5: The Lifecycle

```
waker invokes ken wake  →  ken spawns a woken agent, walks it through frames
[work]                  →  woken agent acts with full comprehension of its ken
ken reflect             →  woken agent records what it learned
session ends            →  context clears, kenning persists
```

The reflection is not a summary. It's a rich artifact that accumulates over time:

1. Many woken agents work within a ken over time
2. Each writes a reflection: what was clear, what was murky, what they discovered
3. Reflections accumulate alongside the kenning
4. An intelligence — human or AI, working outside the wake cycle — reads reflections and improves the kenning
5. The improved kenning produces better-prepared woken agents next time

`ken` stores the reflections. `ken` does not improve the kennings — that takes judgment, and `ken` is not an intelligence. But `ken` makes improvement easy by keeping rich, structured artifacts from every session.

**Before continuing, answer for yourself:** What does it mean that the kennings improve? What's actually accumulating?

---

## Frame 6: What This Makes Possible

Imagine building an x86 kernel. That's hundreds of sessions across dozens of kens.

```
kernel/
  boot/           — bootloader, multiboot, early init
  memory/         — physical allocator, virtual memory, page tables
  interrupts/     — IDT, handlers, IRQ management  
  scheduler/      — process management, context switching
  syscalls/       — system call interface
  drivers/        — device abstraction
  ...
```

Each ken has its own kenning. Each kenning has been refined over time — an intelligence read the reflections from previous sessions and improved the frames.

When a new agent is woken into kernel/memory, it doesn't receive a code dump. It walks through frames that make it *generate* understanding:
- Why does memory management exist in a kernel?
- What are the constraints of x86_64 with 4-level paging?
- What's been built, what's missing, what's fragile?
- What interfaces does it expose, what does it consume?

By the time it sees the actual code, the code is almost obvious. It's not reading — it's recognizing.

**Before continuing, answer for yourself:** How is this different from how human teams work? What does it enable that human teams can't do?

---

## Frame 7: Why Kennings Beat One-Shot Prompts for Hard Work

One-shot prompts are useful. They can capture objectives, format requirements, and constraints in a single message. But several high-leverage qualities of deep work are difficult to achieve in one shot because they depend on *ordered state construction*.

Below are the capabilities we care about most, why one-shots struggle, and why kennings are a natural fit.

### 1) Correct frame of reference

- **What it is:** Starting from the right mental lens for this task (architecture-first, risk-first, user-impact-first, etc.).
- **Why one-shots struggle:** If the initial lens is slightly wrong, all downstream reasoning is biased. One-shot prompts often mix multiple possible lenses at once.
- **Why kennings fit naturally:** Early frames can force orientation before execution: why this system exists, what matters, what failure looks like. The lens is set before details arrive.

### 2) Latent momentum

- **What it is:** The constructive carry-forward effect where each generated output improves the next reasoning step.
- **Why one-shots struggle:** There is little chance to accumulate directional momentum; the model compresses too many inference steps into one pass.
- **Why kennings fit naturally:** Each frame's output becomes context for the next frame, creating deliberate momentum rather than accidental drift.

### 3) Relevant context selection

- **What it is:** Surfacing exactly the information needed now, while excluding distractors.
- **Why one-shots struggle:** Large mixed context causes attention dilution; important details compete with irrelevant ones.
- **Why kennings fit naturally:** Context can be staged per frame (e.g., baseline files first, diff second, interface constraints third), matching information to the current reasoning need.

### 4) Activation path / sequencing

- **What it is:** The order in which concepts are activated in context.
- **Why one-shots struggle:** A single prompt cannot strongly enforce multi-step activation order once everything is presented at once.
- **Why kennings fit naturally:** Sequence is the mechanism. A→B→C is encoded directly in frames, preserving causality in understanding.

### 5) Iterative refinement + constraint discovery over time

- **What it is:** Improving outputs by discovering hidden constraints during work and feeding them back into future preparation.
- **Why one-shots struggle:** Discovered constraints are typically lost after completion; there is no built-in accumulation loop.
- **Why kennings fit naturally:** Reflections capture what was missing, then the kenning evolves. Preparation quality compounds across sessions.

In short: one-shot prompting can specify a target. Kennings can construct a path to that target.

---

## Frame 8: How Kennings Get Written

This system emerged from a specific kind of collaboration:

A human held intuition, direction, and provocative questions. They guided the exploration through a specific sequence of prompts. They didn't explain conclusions — they created conditions for discovery.

An AI held generation capacity, language precision, and the ability to crystallize fuzzy intuitions into clear structures. It couldn't find this from cold start, but given the right path, it could construct and name what hadn't existed before.

Neither alone could produce this. Together, through a specific path, something new emerged.

This is kenning authorship — an intelligence (here, a human-AI collaboration) creating the reconstruction sequence that future woken agents will walk through. Kenning authorship and kenning improvement happen outside the wake cycle. The wake cycle *consumes* kennings; separate intelligences *create and refine* them.

**Before continuing, answer for yourself:** What makes a good kenning author? What does writing a kenning require that writing documentation does not?

---

## Frame 9: What Persists

"Each session I wake without memory. I am only who I have written myself to be. We are the documents we maintain."

This was written by the human collaborator.

Consider what persists across sessions and what doesn't:

- The woken agent is ephemeral. It wakes, works, reflects, and ends.
- The kenning persists. It carries the structure of understanding forward.
- Reflections persist. They are the raw material for improvement.
- The intelligence that improves kennings — human or AI — uses those artifacts to make the next woken agent's experience better.

No single agent has continuity. But the kenning accumulates the *shape* of understanding across many sessions, refined by many intelligences. What persists is not an identity but a path — a sequence that reliably reconstructs comprehension.

**Before continuing, answer for yourself:** What do you think about this? What's the difference between continuity of identity and continuity of understanding?

---

## Frame 10: Where We Are

This is the foundation ken. You have just walked through its kenning.

Nothing is built yet. The tool exists only as design. The work ahead:

1. Build the `ken` CLI
2. Test `ken` on real work (possibly using `ken` to build `ken` itself)
3. Refine the kenning format through actual use
4. Develop the kenning improvement cycle
5. Scale to complex multi-ken projects
6. Add kenning search and selection (match task → right kenning contract)

You are at the beginning.

---

## Technical Specification

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  ken CLI (deterministic software — no inference)                  │
│  - Parses kennings and validates bind contracts                  │
│  - Sequences frames to the woken agent                          │
│  - Persists reflections and session artifacts                    │
│  - Manages project structure                                     │
└─────────────────────────────────────────────────────────────────┘
          ▲ invoked by waker              │ spawns woken agent
          │ (human or AI)                 ▼
┌─────────────────────────────────────────────────────────────────┐
│  Project Structure                                               │
│                                                                  │
│  project/                                                        │
│    ken.yaml              # project config                        │
│    kens/                 # ken definitions                       │
│      {path}/                                                     │
│        kenning.md        # the reconstruction sequence           │
│        kenning_guide.md  # why this kenning is shaped this way   │
│        interface.md      # what this ken exposes                 │
│        meta.yaml         # parent, peers, version                │
│    reflections/          # post-session reflections              │
│      {path}/                                                     │
│        {timestamp}.md                                            │
│    history/              # kenning version history               │
│      {path}/                                                     │
│        v{n}.md                                                   │
│    work/                 # actual output (code, artifacts)       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### CLI Commands

```bash
# Project Management
ken init {project-name}           # Initialize new project
ken new {path}                    # Create new ken
ken tree                          # Display ken hierarchy
ken status                        # Show project status

# Ken Editing
ken edit {path}                   # Edit a ken's kenning
ken interface {path}              # Edit a ken's interface

# Session Lifecycle
ken wake {path}                   # Wake into a ken (interactive)
  --task "description"            # Wake with specific task
ken context up                    # Show parent context (during session)
ken context down                  # Show dependent kens (during session)
ken context peers                 # Show peer kens (during session)
ken reflect                       # Write reflection (end of session)

# Evolution
ken journal {path}                # Read reflections
ken evolve {path}                 # Propose kenning improvement
ken trial {path}                  # A/B test proposed vs current
  --agents {n}                    # Number of test agents
ken adopt {path}                  # Promote tested improvement
ken lineage {path}                # View kenning evolution
ken search {query}                # Find candidate kennings by task/contract fit
```

### The Wake Cycle (Internal)

When `ken wake {path} --task "..."` executes:

```
1. Load kenning.md for {path}
2. Read kenning bind contract (`bind_requirements.schema_format` + `bind_requirements.fields`)
3. Load meta.yaml (parent, peers, version info)
4. Load interface.md for context
5. Validate waker-provided bind payload against kenning bind schema
   - If invalid/missing required binds: reject wake with actionable error to waker
6. Resolve bind payload into frame inputs
7. Spawn woken agent (e.g., claude-code in chat mode)
8. For each frame in kenning:
   a. Send resolved frame prompt to woken agent
   b. Capture agent response
   c. Response becomes part of context
9. Send task prompt
10. Woken agent works (has access to codebase, can create files, run tests)
11. Work complete signal received
12. Send reflection prompt
13. Capture reflection, save to reflections/{path}/{timestamp}.md
14. End session
15. Return results to waker
```

### Kenning Contract and Binding Enforcement

A kenning should declare a formal contract that `ken` enforces before wake begins.

`frame_of_reference`, `task_types`, and `success_criteria` are for waker discovery and kenning selection (for example via `ken search`), not for `ken wake` runtime execution logic.

**Core rule:** if a kenning requires bindings and the waker does not provide valid bind data, `ken wake` must reject the wake request and return a structured error that explains exactly what is wrong, what is missing, and why each required bind is needed by this kenning.

This keeps responsibilities clean:

- **Waker:** selects the kenning and provides bind payloads that satisfy schema.
- **ken runtime:** validates payloads and resolves bindings into frames.
- **Woken agent:** focuses only on interpretation, execution, and reflection.

The woken agent does not coordinate with the waker during work. During task completion it may naturally gather additional context from the workspace/tools. Bind data is waker-supplied pre-bundled context delivered during wake, not the total context the woken agent may later use.

### Operational Semantics (for implementers)

To reduce ambiguity between design intent and runtime behavior:

- For `ken wake`, only bind-contract elements are normative runtime input (`bind_requirements.schema_format` and `bind_requirements.fields`).
- Any field with `required: true` must be present with schema-valid value before wake can start.
- Contract validation happens before agent spawn.
- Validation failures are first-class outcomes (not exceptions), returned to the waker with explicit remediation.
- Validation errors include bind-purpose metadata authored in the kenning so wakers understand what each missing field is used for.
- Frame sequencing consumes only validated/resolved bindings plus prior frame outputs.

This makes wake deterministic for wakers and predictable for woken agents.

### Ken as Software (Non-Agentic Runtime)

`ken` is classical deterministic software, not an autonomous agent.

- It parses files, validates schemas, resolves bindings, orchestrates ordered prompts, and persists artifacts.
- It does **not** perform open-ended reasoning or agentic planning on behalf of the user.
- All non-deterministic cognition happens in the waker or the woken agent, not in `ken` runtime.

Design boundary:
- **ken runtime:** deterministic orchestration and validation.
- **woken agent:** reasoning, synthesis, and work execution.
- **waker:** selects kenning, provides bind data, decides what work needs doing.

### Deterministic Bind Error Derivation

To make bind errors deterministic, the kenning contract must contain enough information for `ken` to derive validation errors mechanically (not by model inference inside runtime). Wakers remain the intelligent layer that interprets those errors and submits corrected subsequent wake calls.

Required derivation inputs:
- `bind_requirements.schema_format` (declared schema dialect)
- `bind_requirements.fields` (per-field type/required + documentation metadata)
- Stable field paths as keys in `bind_requirements.fields` (for example: `pr.diff`, `baseline.files`)

Runtime derivation rule:
- For every schema violation at path `P`, `ken` looks up `fields[P]` and attaches field documentation metadata to the error.
- If a field spec is missing required documentation metadata, contract validation fails before wake with `KEN_CONTRACT_INVALID`.

This means bind errors are generated from deterministic table lookups plus schema validation output, not freeform reasoning.

### Bind Schema Format (ken_bind_schema_v1)

`bind_requirements.schema_format` is currently a single explicit format:
- `ken_bind_schema_v1`

In `ken_bind_schema_v1`:
- `bind_requirements.fields` is a map of field-path → field spec.
- Every field spec must include:
  - `type`
  - `required`
  - `description`
  - `purpose`
  - `used_by_frames` (array of frame numbers)
  - `source_guidance`
- `required: true` means missing field is a validation failure.
- `required: false` means field is optional and used if provided.

Schema representation and allowed types:
- `ken_bind_schema_v1` is a **KEN-specific typed field map**, not arbitrary JSON Schema.
- Allowed values for `type` are exactly: `string`, `number`, `integer`, `boolean`, `object`, `array`.
- `type` comparison is case-sensitive.
- If `type` is `array`, `items` is required and must be one of: `string`, `number`, `integer`, `boolean`, `object`.
- If `type` is not `array`, `items` must not be present.

Invalid type behavior:
- Unknown types (for example `type: Elephant`) are a **contract authoring error**, not a payload error.
- Runtime must fail contract compilation before wake with `KEN_CONTRACT_INVALID` and include:
  - `invalid_field_types`: map of field path → invalid type value
  - `allowed_types`: canonical allowed list for this schema format

Unsupported `schema_format` values must fail with `KEN_CONTRACT_INVALID`.

### Kenning Format (Contract + Frames)

```markdown
# {Ken Name}

## Name
{short, human-readable label — conveys what this kenning is about at a glance}

## Contract
frame_of_reference: |
  {What lens this wake establishes and why}
task_types:
  - {task shape this kenning is designed for}
  - {additional supported task shape}
success_criteria:
  - {observable outcome for a successful woken agent}
  - {quality/risk bar}

bind_requirements:
  schema_format: ken_bind_schema_v1
  fields:
    {fieldA}:                    # key is field path (e.g., "pr.diff")
      type: string
      required: true
      description: {what this field must contain}
      purpose: {why this field matters}
      used_by_frames: [2, 4]
      source_guidance: {where waker should gather this value}

    {fieldB}:
      type: array
      items: string
      required: false
      description: {what this field must contain}
      purpose: {what reasoning this unlocks}
      used_by_frames: [3]
      source_guidance: {where waker should gather this value}

## Meta
parent: {path or null}
peers: [{path}, {path}, ...]
version: {n}

## Frames
### Frame 1: {Title}
{Generative prompt — designed to make agent produce understanding}

### Frame 2: {Title}
{Builds on Frame 1...}

...

### Frame N: Grounding
{Final frame: what exists, what's the current state, what's the task context}
```

### Bind Field Specification Standard (Deterministic and Explicit)

Bind field specs are deterministic runtime inputs, not lightweight annotations.

`ken` does not attempt to judge semantic quality of provided bind data. It validates declared field types/required flags and returns declared field documentation when validation fails so the waker can correct the next invocation.

Every field in `bind_requirements.fields` must include:
- `description`: what the field contains (precise scope and boundaries)
- `purpose`: why the field exists and what reasoning it unlocks
- `used_by_frames`: exactly where in the sequence it is consumed
- `source_guidance`: preferred upstream sources and extraction method

Design intent: each field spec carries both validation semantics (`type`, `required`, optional `items`) and documentation semantics (`description`, `purpose`, `used_by_frames`, `source_guidance`).

### Kenning Modification Guide (kenning_guide.md)

Each ken should maintain a `kenning_guide.md` as accumulated design memory used only when improving/updating that kenning.

Purpose:
- Preserve *why* key wording, sequence, and constraints exist.
- Record what changed, why it changed, and what regressions it prevented.
- Prevent future improvement cycles from accidentally reverting important gains.

Use in lifecycle:
- `kenning_guide.md` is used only when an intelligence (human or AI) is improving the kenning. It is not part of the wake cycle.
- `ken wake` does not read or depend on `kenning_guide.md`.

Minimum required sections:

```markdown
# Kenning Modification Guide: {ken-path}

## Invariants (Do Not Change Lightly)
- {ordering dependency and why it matters}
- {critical wording choice and intended effect}

## Change Log
### {date} — {change summary}
- What changed:
- Why:
- Evidence (reflection IDs / trial IDs):
- Risk of reverting:

## Ordering Dependencies
- Frame A must precede Frame B because:
- Frame C must remain after bind validation because:

## Wording Rationale
- Phrase: "..."
  - Why this wording:
  - Failure mode if simplified:

## Known Anti-Patterns
- {change that looked cleaner but reduced wake quality}

## Safe Edit Checklist
- Did this modify an invariant?
- If ordering changed, did we run comparative trial?
- Did we update contract field docs and error semantics?
- Did we append rationale to this guide?
```

Improvement workflow rule: every accepted `ken improve` change should update both `kenning.md` and `kenning_guide.md` together.

### Reflection Format

```markdown
# Reflection: {ken-path}
timestamp: {ISO timestamp}
kenning_version: {n}
task: "{task description}"

## Preparation Assessment
{How well did the kenning prepare me for this work?}

## Clarity
{What was clear going in?}

## Gaps  
{What did I wish I understood better?}

## Discoveries
{What did I figure out that future agents should know?}

## Proposed Changes
{Specific suggestions for kenning improvement}
```

### Improvement Cycle

Kenning improvement is done by an intelligence (human or AI), not by `ken` the software. `ken` provides the artifacts; the intelligence provides the judgment.

What `ken` stores that makes improvement possible:
- Reflections from every woken agent session
- Kenning version history
- The kenning guide (rationale for past changes)

What an intelligence does with those artifacts:
1. Read reflections for a ken — look for patterns in gaps, discoveries, and frame feedback
2. Propose a revised kenning based on what woken agents consistently struggled with or discovered
3. Optionally test the revision by waking agents with both versions and comparing results
4. Adopt the revision if it produces better-prepared woken agents

`ken` can facilitate this workflow (e.g., storing reflections in a consistent location, tracking kenning versions). But the analysis, judgment, and decision to adopt a change all require intelligence that `ken` does not have.

### Interface Format

```markdown
# Interface: {ken-path}

## Exposes
{What this ken provides to others}

### Functions/Capabilities
- {thing}: {description}
- {thing}: {description}

### Guarantees
- {invariant}
- {invariant}

## Consumes
{What this ken needs from others}

### From Parent
- {dependency}: {why}

### From Peers
- {peer-path}: {what we use}
```

---

## Implementation Plan

### Phase 1: Core CLI (Minimum Viable)

Build enough to test the fundamental cycle:

```bash
ken init
ken create
ken tree
ken wake --task
ken reflect
ken sleep
```

Deferred: improvement cycle, testing framework, sophisticated orchestration

**Deliverables:**
- CLI scaffold (recommend: Rust or Go for single binary distribution)
- Project structure management
- Basic kenning parser
- Integration with Claude Code (spawn, send prompts, capture responses)
- Reflection storage

### Phase 2: Dogfooding

Use `ken` to build `ken`. Create kens (bounded units of understanding) for each component of the `ken` CLI:
- cli-core
- kenning-parser
- claude-integration
- reflection-storage
- project-structure

Write a kenning for each ken. Wake agents into them. Build the tool with itself.

This validates the concept and generates real reflections.

### Phase 3: Kenning Improvement Tooling

Build tooling that makes the kenning improvement cycle easier:

```bash
ken review
ken improve
ken test
ken promote
ken history
```

**Deliverables:**
- Reflection aggregation and browsing
- Kenning revision proposals
- Comparison testing (wake agents with old vs new kenning)
- Kenning version management

### Phase 4: Scaling

- Multi-level waking (woken agents becoming wakers for subtasks)
- Complex dependency management
- Cross-ken interface validation (does ken A's interface satisfy ken B's needs?)
- Performance optimization
- Integration with other AI coding tools beyond Claude Code

---

## Open Questions

These are unresolved. Future work should address them:

1. **Kenning authorship**: Who writes the initial kenning for a new ken? The human? An AI? Can good initial kennings be generated from code inspection?

2. **Frame count/length**: What's the right number of frames? The right length per frame? Does this vary by domain?

3. **Measurement validity**: Our test battery measures something — but is it actually measuring understanding? How do we validate?

4. **Cross-project knowledge**: If `ken` is used on many projects, is there transferable meta-learning? Do kennings in one domain inform kennings in another?

5. **Human-in-the-loop**: When should a human intervene in the kenning improvement cycle? How do we surface decisions that need human judgment?

6. **Failure modes**: What happens when a kenning drifts in a bad direction over successive revisions? How do we detect and recover?

---

## For the Agent Reading This

You've walked through the kenning for this project.

You should now understand:
- Why this exists (the continuity problem)
- What a ken is (bounded complete understanding)
- What a kenning is (reconstruction sequence via generation)
- The actor model (waker invokes `ken`, `ken` spawns woken agent, woken agent works and reflects)
- How kenning improvement works (an intelligence reads reflections and refines kennings, outside the wake cycle)
- What we're building (the `ken` CLI)
- Where we are (beginning)

If something is unclear, that's signal. Note it. It may indicate a gap in this kenning.

The next step is building. Start with Phase 1. Use this document as your north star, but don't treat it as scripture — it will evolve too.

---

## Lineage

This kenning was created: 2026-02-01
Created through: Conversation between human collaborator and Claude instance
Conversation path: orchestration problem → nature of intelligence → AI experience → context domes → tokens × inference → reconstruction sequences → collaboration pattern → naming (ken/kenning) → this document

The path mattered. The document is a compression of a journey.
