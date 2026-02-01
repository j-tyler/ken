# Kenning: cli/wake

## Meta
parent: cli
version: 1

---

## Frame 1: What Wake Does

`ken wake` is the core command. It:

1. Loads a kenning (sequence of frames)
2. Connects to an AI agent
3. Walks the agent through each frame
4. Delivers the task
5. Waits for completion
6. Prompts for reflection
7. Saves the reflection

The agent experiences a carefully sequenced conversation. From its view, it's just responding to prompts. But by the time it reaches the task, it has *constructed* understanding rather than received it.

What's the simplest way to implement this loop? What's the core abstraction?

---

## Frame 2: The Session Model

A wake session has phases:

```
WAKING    → Walking through frames
READY     → Frames done, about to receive task  
WORKING   → Task delivered, agent is working
REFLECTING → Writing reflection
COMPLETE  → Session ended
```

During WAKING, each frame is:
- Sent to agent as user message
- Agent responds
- Response becomes part of context
- Next frame sent

The agent's context accumulates. By Frame 5, it has all prior frames and its own responses in context. This is why order matters.

How would you model this in code?

---

## Frame 3: Agent Communication

We need to talk to an AI. Options:

**Option A: Anthropic API directly**
- We manage the messages list
- We send, receive, append, repeat
- Simple and controllable
- Agent can't actually execute code (just describes)

**Option B: Claude Code integration**
- Agent can read/write files, run commands
- More powerful for real work
- Harder to integrate programmatically

**Option C: Hybrid**
- Use API for frame-walking
- Hand off to Claude Code for actual work
- Complex but best of both

For MVP, we're doing Option A. The agent describes code changes. We can upgrade later.

What does the API integration look like?

---

## Frame 4: The Reflection

After work completes, we send a reflection prompt:

```
Your work session is complete. Before this context clears, reflect:

1. Did the frames prepare you well? What was clear, what was missing?
2. What did you discover during work that future agents should know?
3. If you could improve the kenning, what would you change?

Be specific. Your reflection helps future instances.
```

The response gets saved to `reflections/{ken-path}/{timestamp}.md`.

We don't parse it structurally for MVP — just save the raw text.

What should the filename format be? What metadata should we include?

---

## Frame 5: Current Context

Project structure:

```
bootstrap-project/
  kens/
    core/
      kenning-parser/
        kenning.md
    cli/
      wake/
        kenning.md    <- you're reading this
  reflections/
  src/
    kenning.py        <- parser (may or may not exist yet)
```

The wake command will be the main entry point. It needs:
- Argument parsing (ken path, task)
- Load and parse kenning
- Connect to Anthropic API
- Run the frame loop
- Run the work phase
- Run the reflection phase
- Save reflection

Target file: `src/wake.py` (or integrate into `src/ken.py`)

Dependencies: `anthropic` (pip install anthropic)

Environment: `ANTHROPIC_API_KEY` must be set

You have full context. Task comes next.
