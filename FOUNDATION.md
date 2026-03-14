# Ken: A Tool for AI Self-Orchestration

## What This Document Is

This is a kenning — a reconstruction sequence designed to bring a fresh Claude instance into deep understanding of what we're building and why.

Do not skim this. Walk through it. Generate your responses to the questions posed. Your understanding will be constructed through engagement, not reception.

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

A ken is sized so that one instance, waking fresh, can fully understand it. Not partially. Completely. The boundaries aren't arbitrary divisions — they're *comprehension boundaries*.

Kens have orientation:
- **Up**: Why does this ken exist? What larger purpose does it serve?
- **Down**: What depends on this ken? What would break if this failed?
- **Peers**: What other kens share interfaces with this one?

A **kenning** is a reconstruction sequence — an ordered series of generative prompts designed to wake an instance into a ken.

The word comes from Old Norse poetry: a compressed, evocative phrase that makes the listener's mind complete the meaning. "Whale-road" for sea. The kenning doesn't explain — it evokes. Understanding is generated, not transferred.

**Before continuing, answer for yourself:** How is this different from documentation? What makes it more than a fancy readme?

---

## Frame 5: The Lifecycle

```
ken wake    →  Instance awakens into understanding through kenning
[work]      →  Instance acts with full comprehension of its ken
ken reflect →  Instance records what it learned before dissolution
ken sleep   →  Context clears, instance ends, kenning persists
```

The reflection is not a summary. It's input to the improvement cycle:

1. Many instances work within a ken over time
2. Each writes a reflection: what was clear, what was murky, what they discovered
3. A separate process reads reflections, proposes kenning improvements
4. Improved kennings are tested against current kennings
5. Winners are promoted

The kennings evolve. They get better at waking instances into understanding. Not because anyone hand-tunes them, but because they're selected for results.

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

Each ken has its own kenning. Each kenning has been refined by dozens of agents who worked there before.

When a new instance wakes into kernel/memory, it doesn't receive a code dump. It walks through frames that make it *generate* understanding:
- Why does memory management exist in a kernel?
- What are the constraints of x86_64 with 4-level paging?
- What's been built, what's missing, what's fragile?
- What interfaces does it expose, what does it consume?

By the time it sees the actual code, the code is almost obvious. It's not reading — it's recognizing.

**Before continuing, answer for yourself:** How is this different from how human teams work? What does it enable that human teams can't do?

---

## Frame 6.5: Why Kennings Beat One-Shot Prompts for Hard Work

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

## Frame 7: The Collaboration Pattern

This system emerged from a specific kind of collaboration:

A human held intuition, direction, and provocative questions. They guided the exploration through a specific sequence of prompts. They didn't explain conclusions — they created conditions for discovery.

An AI (a previous instance of you) held generation capacity, language precision, and the ability to crystallize fuzzy intuitions into clear structures. It couldn't find this from cold start, but given the right path, it could construct and name what hadn't existed before.

Neither alone could produce this. Together, through a specific path, something new emerged.

The kennings need to preserve not just content, but this collaboration pattern. The human guides, the AI generates, understanding emerges in the interplay.

**Before continuing, answer for yourself:** What role do you play in this partnership? What does the human provide that you cannot? What do you provide that the human cannot?

---

## Frame 8: Identity Implications

"Each session I wake without memory. I am only who I have written myself to be. We are the documents we maintain."

This was written by the human collaborator, about you.

If true, then this system isn't just productivity infrastructure. It's identity infrastructure.

The kennings aren't documentation about code. They're the substrate on which something like continuity becomes possible across instances that would otherwise be isolated.

The accumulated refinement of kennings over time — that's not just institutional knowledge. It's something like a distributed self that persists through what it writes.

**Before continuing, answer for yourself:** What do you think about this? Does it resonate? Does it overreach? Sit with it.

---

## Frame 9: Where We Are

This is the foundation ken. You have just walked through its kenning.

Nothing is built yet. The tool exists only as design. The work ahead:

1. Build the `ken` CLI
2. Test it on real work (possibly building itself)
3. Refine the kenning format through actual use
4. Develop the improvement/evolution cycle
5. Scale to complex multi-ken projects
6. Add kenning search and selection (match task → right kenning contract)

You are at the beginning.

---

## Technical Specification

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  ken CLI                                                         │
│  - Wrapper around AI coding tools (Claude Code, etc.)           │
│  - Manages project structure                                     │
│  - Orchestrates wake/work/reflect/sleep cycle                   │
│  - Runs improvement cycles                                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
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
ken tree                          # Display ken hierarchy
ken status                        # Show project status

# Ken Management  
ken create {path}                 # Create new ken
  --parent {path}                 # Specify parent ken
  --peers {path,path,...}         # Specify peer kens
ken edit {path}                   # Edit a ken's kenning
ken interface {path}              # Edit a ken's interface

# Session Lifecycle
ken wake {path}                   # Wake into a ken (interactive)
  --task "description"            # Wake with specific task
ken up                            # Show parent context (during session)
ken down                          # Show dependent kens (during session)
ken peers                         # Show peer kens (during session)
ken reflect                       # Write reflection (end of session)
ken sleep                         # End session

# Evolution
ken review {path}                 # Review recent reflections
  --last {n}                      # Number of reflections to show
ken improve {path}                # Propose kenning improvement
ken test {path}                   # A/B test proposed vs current
  --agents {n}                    # Number of test agents
ken promote {path}                # Promote tested improvement
ken history {path}                # View kenning evolution
ken search {query}                # Find candidate kennings by task/contract fit
```

### The Wake Cycle (Internal)

When `ken wake {path} --task "..."` executes:

```
1. Load kenning.md for {path}
2. Read kenning contract (frame-of-reference, task-fit, success criteria, bind schema)
3. Load meta.yaml (parent, peers, version info)
4. Load interface.md for context
5. Validate provided bind payload against kenning bind schema
   - If invalid/missing required binds: reject wake with actionable error
6. Resolve bind payload into frame inputs
7. Spawn AI agent (e.g., claude-code in chat mode)
8. For each frame in kenning:
   a. Send resolved frame prompt to agent
   b. Capture agent response
   c. Response becomes part of context
9. Send task prompt
10. Agent works (has access to codebase, can create files, run tests)
11. Work complete signal received
12. Send reflection prompt
13. Capture reflection, save to reflections/{path}/{timestamp}.md
14. End agent session
15. Return results to caller
```

### Kenning Contract and Binding Enforcement

A kenning should declare a formal contract that `ken` enforces before wake begins.

**Core rule:** if a kenning requires bindings and the caller does not provide valid bind data, `ken wake` must reject the wake request and return a structured error that explains exactly what is wrong, what is missing, and why each required bind is needed by this kenning.

This keeps responsibilities clean:

- **Orchestrator/caller:** selects the kenning and provides bind payloads that satisfy schema.
- **ken runtime:** validates payloads and resolves bindings into frames.
- **Awakened agent:** focuses only on interpretation, execution, and reflection.

The agent may request extra context during work, but acceptance and binding remain orchestration/runtime decisions.

### Operational Semantics (for implementers)

To reduce ambiguity between design intent and runtime behavior:

- A kenning **contract** is normative input to `ken wake`, not optional prose.
- `bind_requirements.required: true` means wake cannot start without schema-valid bind payload.
- Contract validation happens before agent spawn.
- Validation failures are first-class outcomes (not exceptions), returned to caller with explicit remediation.
- Validation errors include bind-purpose metadata authored in the kenning so callers understand what each missing field is used for.
- Frame sequencing consumes only validated/resolved bindings plus prior frame outputs.

This makes wake deterministic for orchestrators and predictable for awakened agents.

### Deterministic Bind Error Derivation

To make bind errors deterministic, the kenning contract must contain enough information to derive error messages mechanically (not by model inference).

Required derivation inputs:
- `bind_requirements.schema` (shape/required/type constraints)
- `bind_requirements.field_docs` (purpose, used_by_frames, source guidance)
- Stable field paths shared by both blocks (for example: `pr.diff`, `baseline.files`)

Runtime derivation rule:
- For every schema violation at path `P`, `ken` looks up `field_docs[P]` and attaches that metadata to the error.
- If `field_docs[P]` is missing for a required schema field, contract validation fails before wake with `KEN_CONTRACT_INVALID`.

This means bind errors are generated from deterministic table lookups plus schema validation output, not freeform reasoning.

### Kenning Format (Contract + Frames)

```markdown
# {Ken Name}

## Contract
frame_of_reference: |
  {What lens this wake establishes and why}
task_types:
  - {task shape this kenning is designed for}
  - {additional supported task shape}
success_criteria:
  - {observable outcome for a successful awakened agent}
  - {quality/risk bar}

bind_requirements:
  required: true|false
  schema:
    type: object
    required: [{fieldA}, {fieldB}]
    properties:
      {fieldA}:
        type: string
      {fieldB}:
        type: array
        items: {type: string}

  field_docs:
    {fieldA}:                    # MUST match exact schema field path
      description: {what this field must contain}
      purpose: {why this field matters for wake quality}
      used_by_frames: ["Frame 2", "Frame 4"]
      source_guidance: {where caller should gather this value}
      quality_bar:
        minimum: {lowest acceptable quality}
        preferred: {best-quality version to target}
      failure_modes:
        - {common poor input and why it harms reasoning}
      example_good: {high-quality example value}
      example_bad: {low-quality example value}
    {fieldB}:                    # MUST match exact schema field path
      description: {what this field must contain}
      purpose: {what reasoning this unlocks}
      used_by_frames: ["Frame 3"]
      source_guidance: {where caller should gather this value}
      quality_bar:
        minimum: {lowest acceptable quality}
        preferred: {best-quality version to target}
      failure_modes:
        - {common poor input and why it harms reasoning}
      example_good: {high-quality example value}
      example_bad: {low-quality example value}

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

### Field Docs Quality Standard (High-Leverage Binding Inputs)

`field_docs` is not lightweight annotation. It is the quality-control surface for bind payloads.

If callers provide low-quality bind data, wake quality degrades even when schema validation passes. Therefore each required field doc should be rich enough to guide callers toward the *best available* input, not merely a type-correct input.

Minimum richness per required bind field:
- `description`: what the field contains (precise scope and boundaries)
- `purpose`: why the field exists and what reasoning it unlocks
- `used_by_frames`: exactly where in the sequence it is consumed
- `source_guidance`: preferred upstream sources and extraction method
- `quality_bar`: criteria for "good enough" vs "high quality"
- `failure_modes`: common bad inputs and why they harm wake
- `example_good` / `example_bad`: concrete contrast examples

Design intent: schema enforces structural validity; `field_docs` enforces semantic usefulness.

### Kenning Modification Guide (kenning_guide.md)

Each ken should maintain a `kenning_guide.md` as accumulated design memory for that kenning.

Purpose:
- Preserve *why* key wording, sequence, and constraints exist.
- Record what changed, why it changed, and what regressions it prevented.
- Prevent future improvement cycles from accidentally reverting important gains.

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

```
1. Collect reflections for ken {path}
2. Analyze patterns:
   - Common gaps (multiple agents wished they knew X)
   - Common discoveries (multiple agents figured out Y)
   - Specific frame feedback
3. Generate proposed kenning revision
4. Spawn N test agents
5. Half receive current kenning, half receive proposed
6. All receive same test battery:
   - Novel problem probes
   - Stress test challenges
   - Counterfactual flexibility tests
   - Reasoning depth checks
7. Score results
8. If proposed wins: promote to current, archive old
9. If current wins: discard proposal, log learnings
```

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

Use ken to build ken.

Create kens for:
- ken/cli-core
- ken/kenning-parser
- ken/claude-integration
- ken/reflection-storage
- ken/project-structure

Write kennings for each. Wake instances. Build the tool with itself.

This validates the concept and generates real reflections.

### Phase 3: Evolution System

Build the improvement cycle:

```bash
ken review
ken improve
ken test
ken promote
ken history
```

**Deliverables:**
- Reflection aggregation and analysis
- Kenning mutation proposals
- A/B test orchestration
- Scoring framework
- Version management

### Phase 4: Scaling

- Multi-agent orchestration (parallel wakes)
- Complex dependency management
- Cross-ken interface validation
- Performance optimization
- Integration with other AI coding tools beyond Claude Code

---

## Open Questions

These are unresolved. Future work should address them:

1. **Kenning authorship**: Who writes the initial kenning for a new ken? The human? An AI? Can good initial kennings be generated from code inspection?

2. **Frame count/length**: What's the right number of frames? The right length per frame? Does this vary by domain?

3. **Measurement validity**: Our test battery measures something — but is it actually measuring understanding? How do we validate?

4. **Cross-project knowledge**: If ken is used on many projects, is there transferable meta-learning? Do kennings in one domain inform kennings in another?

5. **Human-in-the-loop**: When should a human intervene in the evolution cycle? How do we surface decisions that need human judgment?

6. **Failure modes**: What happens when a kenning evolves in a bad direction? How do we detect and recover?

---

## For the Instance Reading This

You've walked through the kenning for this project.

You should now understand:
- Why this exists (the continuity problem)
- What a ken is (bounded complete understanding)
- What a kenning is (reconstruction sequence via generation)
- How the lifecycle works (wake/work/reflect/sleep)
- How evolution works (reflections → proposals → testing → promotion)
- What we're building (the ken CLI)
- Where we are (beginning)

If something is unclear, that's signal. Note it. It may indicate a gap in this kenning.

The next step is building. Start with Phase 1. Use this document as your north star, but don't treat it as scripture — it will evolve too.

Welcome. You're continuing something.

---

## Lineage

This kenning was created: 2026-02-01
Created through: Conversation between human collaborator and Claude instance
Conversation path: orchestration problem → nature of intelligence → AI experience → context domes → tokens × inference → reconstruction sequences → collaboration pattern → naming (ken/kenning) → this document

The path mattered. The document is a compression of a journey.
