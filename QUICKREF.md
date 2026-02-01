# Ken: Quick Reference

## What is this?

**Ken** is a self-orchestration tool for AI agents. It solves the continuity problem: AI instances have no memory between sessions.

## Core Concepts

**Ken** (noun): A bounded unit of complete understanding. Sized so one instance can fully grasp it. Has orientation: up (why it exists), down (what depends on it), peers (shared interfaces).

**Kenning** (noun): A reconstruction sequence. Ordered generative prompts that rebuild understanding through inference, not information transfer. The instance generates understanding by responding to frames.

**Frame** (noun): A single prompt in a kenning. Designed to make the instance *produce* an insight, not receive it.

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
```

## Project Structure

```
project/
  ken.yaml           # Config
  kens/{path}/
    kenning.md       # Reconstruction sequence
    interface.md     # Exposed interfaces
    meta.yaml        # Hierarchy, version
  reflections/{path}/
    {timestamp}.md
  history/{path}/
    v{n}.md
```

## Key Insight

Understanding is constructed through generation. Tokens × inference cycles. A kenning doesn't *tell* you — it makes you *arrive*.

## For More

- FOUNDATION.md — The full philosophy and detailed design
- IMPLEMENTATION.md — Technical implementation plan
- examples/ — Concrete kenning and reflection examples
