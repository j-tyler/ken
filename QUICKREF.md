# Ken: Quick Reference

## What is this?

**Ken** is a self-orchestration tool for AI agents. It solves the continuity problem: AI instances have no memory between sessions.

## Core Concepts

**Ken** (noun): A bounded unit of complete understanding. Sized so one instance can fully grasp it. Has orientation: up (why it exists), down (what depends on it), peers (shared interfaces).

**Kenning** (noun): A reconstruction sequence. Ordered generative prompts that rebuild understanding through inference, not information transfer. The instance generates understanding by responding to frames.

**Frame** (noun): A single prompt in a kenning. Designed to make the instance *produce* an insight, not receive it.

**Kenning contract**: Required metadata for wake selection and validation: frame of reference, task types, success criteria, and bind requirements/schema.

## The Lifecycle

```
ken wake {path} --task "..."   # Walk frames, then receive task
[instance works]                # Full understanding, focused action
ken reflect                     # Record what was learned
ken sleep                       # Context clears, kenning persists
```

## The Evolution Loop

1. Instances work in kens, write reflections
2. Reflections accumulate
3. Improvement process proposes kenning updates
4. A/B test: new kenning vs current
5. Winner becomes canonical
6. Kennings get better over time

## Key Commands

```bash
# v0.1 Core
ken init {project}           # Create project structure
ken new {path}               # Create a new ken
ken tree                     # View ken hierarchy
ken wake {path} --task "..." # Wake into ken with task
ken reflect                  # Write reflection AND end session

# v0.2 Navigation
ken context up               # Why does this ken exist?
ken context down             # What depends on this?
ken context peers            # Related kens

# v0.3+ Evolution
ken journal {path}           # Read reflections
ken evolve {path}            # Propose kenning improvement
ken trial {path}             # A/B test improvement
ken adopt {path}             # Accept improvement
ken lineage {path}           # View kenning history
ken search {query}           # Find a kenning by task + contract fit
```

## Project Structure

```
project/
  ken.yaml           # Config
  kens/{path}/
    kenning.md       # Reconstruction sequence
    kenning_guide.md # Why this sequence/wording/order exists
    interface.md     # Exposed interfaces
    meta.yaml        # Hierarchy, version
  reflections/{path}/
    {timestamp}.md
  history/{path}/
    v{n}.md
```


## Why Kennings for These Hard Cases?

These qualities are possible to mention in one-shot prompts, but hard to *reliably realize* without sequencing:

- **Correct frame of reference**: one-shots can mix lenses; kennings orient early.
- **Latent momentum**: one-shots have little stepwise carry-forward; kennings accumulate intermediate reasoning.
- **Relevant context selection**: one-shots overload attention; kennings stage context by phase.
- **Activation path / sequencing**: one-shots collapse order; kennings encode A→B→C explicitly.
- **Iterative refinement + constraint discovery**: one-shots end at output; kennings add reflection and evolution loops.


## Binding Contract (Enforced by `ken`)

A kenning can declare required bindings with an explicit schema.

- If required bind data is missing/invalid, `ken wake` must reject and report exactly what is wrong, plus what each field is used for in the kenning.
- Deterministic bind errors: schema violations + field_docs lookup + stable sorting (no LLM generation).
- `field_docs` should be rich enough to optimize input quality, not just satisfy required keys.
- Orchestrator/caller provides bind payloads; `ken` validates and resolves them before frames run.
- Awakened agent should not choose binding inputs; it may request context, but runtime/orchestrator decides.

## Build Rules for Next Implementing Agent

1. Validate kenning contract before agent spawn.
2. If required bind fields are missing/invalid, reject wake with machine-readable errors.
3. Require field-level bind metadata (`purpose`, `used_by_frames`, `source_guidance`) in kenning contracts.
4. Include quality bars and examples in field docs (`quality_bar`, `failure_modes`, `example_good`, `example_bad`).
5. Do not auto-fill required binds from implicit context.
6. Add `--dry-run` validation mode for orchestration pipelines.
7. For `ken search`, rank by contract fit + bind satisfiability, then show missing binds.

## Kenning Guide (Why Changes Persist)

Every kenning should carry `kenning_guide.md` documenting:
- important ordering dependencies,
- wording choices that are intentional,
- what changed over time and why,
- evidence for accepted edits,
- regressions to avoid reintroducing.

Rule: if `kenning.md` changes meaningfully, `kenning_guide.md` should be updated in the same improvement cycle.

## Key Insight

Understanding is constructed through generation. Tokens × inference cycles. A kenning doesn't *tell* you — it makes you *arrive*.

## For More

- FOUNDATION.md — The full philosophy and detailed design
- IMPLEMENTATION.md — Technical implementation plan
- examples/ — Concrete kenning and reflection examples
