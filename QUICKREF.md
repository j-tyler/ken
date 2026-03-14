# Ken: Quick Reference

## What is this?

`ken` is a tool for structured context delivery to AI agents. It solves the continuity problem: AI agents have no memory between sessions.

## Actors

**Waker**: Any intelligence (human or AI) that invokes `ken wake` to spawn an agent for work. Waker is a role, not an identity.

**Woken agent**: An agent spawned by `ken wake`. Walks frames, works, reflects, ends. A woken agent can itself become a waker by invoking `ken wake` for subtasks.

**Root waker**: The intelligence that started the chain — typically a human, or any agent that entered the workflow outside of `ken wake`.

**`ken` the software**: Deterministic orchestration. Parses, validates, sequences, persists. Does not reason or judge.

## Core Concepts

**Ken** (noun): A bounded unit of complete understanding. Sized so one agent, waking fresh, can fully grasp it. Has orientation: up (why it exists), down (what depends on it), peers (shared interfaces).

**Kenning** (noun): A reconstruction sequence. Ordered generative prompts that rebuild understanding through inference, not information transfer. Every kenning has a **name** — a short label that conveys what the kenning is about at a glance (e.g., "Wake Command Implementation"). The woken agent generates understanding by responding to frames.

**Frame** (noun): A single prompt in a kenning. Designed to make the woken agent *produce* an insight, not receive it.

**Kenning contract**: Metadata for waker discovery/selection (`frame_of_reference`, `task_types`, `success_criteria`) plus bind validation (`bind_requirements.schema_format` + `bind_requirements.fields`).


## The Lifecycle

```
waker invokes ken wake          # ken spawns woken agent, walks it through frames
[woken agent works]             # Full understanding, focused action
ken reflect                     # Woken agent records what it learned
session ends                    # Context clears, kenning persists
```

## Kenning Improvement

Improvement is done by an intelligence (human or AI), outside the wake cycle. `ken` provides the artifacts:

1. Woken agents work in kens (bounded units of understanding), write reflections
2. Reflections accumulate in `ken`'s project structure
3. An intelligence reads reflections, identifies patterns, revises the kenning
4. Optionally tests revised vs current kenning by waking agents with each
5. Adopts the revision if it produces better-prepared agents
6. Kennings get better over time — through intelligent authorship, not automation

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

# v0.3+ Kenning Improvement
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

A kenning can declare required bindings with an explicit schema. Current schema format is `ken_bind_schema_v1`.

- If required bind data is missing/invalid, `ken wake` must reject and report exactly what is wrong, plus what each field is used for in the kenning.
- Deterministic bind errors: field-type/required validation + field metadata lookup + stable sorting (no LLM generation).
- `ken_bind_schema_v1` uses a fixed type system: `string`, `number`, `integer`, `boolean`, `object`, `array` (plus `items` for arrays).
- Unknown type tokens (for example `type: Elephant`) are rejected as `KEN_CONTRACT_INVALID` before payload validation.
- Only bind schema fields are used directly by `ken wake` runtime validation.
- Each bind field spec should include detailed documentation so missing/invalid errors can tell the waker exactly what the field is and why it matters.
- The waker provides bind payloads; `ken` validates and resolves them before frames run.
- Bind inputs are waker-provided pre-bundled context for wake. The woken agent may still naturally gather additional context during task completion.

## Build Rules

1. Validate kenning contract before spawning woken agent.
2. If required bind fields are missing/invalid, reject wake with machine-readable errors returned to the waker.
3. Require per-field bind metadata (`type`, `required`, `description`, `purpose`, `used_by_frames`, `source_guidance`) in kenning contracts.
4. Do not auto-fill required binds from implicit context.
5. Add `--dry-run` validation mode so wakers can check bind payloads without spawning an agent.
6. For `ken search`, rank by contract fit + bind satisfiability, then show missing binds.

## Kenning Guide (Why Changes Persist)

Every kenning should carry `kenning_guide.md` documenting:
- important ordering dependencies,
- wording choices that are intentional,
- what changed over time and why,
- evidence for accepted edits,
- regressions to avoid reintroducing.

This file is for the intelligence improving the kenning, not for the wake cycle. `ken wake` does not read `kenning_guide.md`. If `kenning.md` changes meaningfully, `kenning_guide.md` should be updated alongside it.

## Key Insight

Understanding is constructed through generation. Tokens × inference cycles. A kenning doesn't *tell* you — it makes you *arrive*.

## For More

- FOUNDATION.md — The full philosophy and detailed design
- IMPLEMENTATION.md — Technical implementation plan
- examples/ — Concrete kenning and reflection examples
