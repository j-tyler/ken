# Ken Bootstrap Project

This is the minimal viable implementation to test and validate the concept behind `ken` — that kennings (reconstruction sequences) produce deeper understanding than cold-start documentation.

## Quick Start

```bash
# Requires: claude CLI installed and authenticated
# See: https://docs.anthropic.com/en/docs/claude-code

# Run your first ken session
python ken-bootstrap.py beginning --task "review the codebase and suggest improvements"
```

## Specification Status

This README documents the **bootstrap MVP** workflow and simplified kenning format.

For the current canonical contract/runtime design (kenning contract, deterministic bind validation, wake semantics), use:
- `FOUNDATION.md`
- `IMPLEMENTATION.md`
- `TERMINAL_SESSIONS.md` — how `ken wake` orchestrates CLI agent sessions via tmux
- `QUICKREF.md`

## What Happens

1. The script loads `kens/<path>/kenning.md`
2. It composes all frames + task into a structured prompt
3. It spawns a Claude Code agent in an isolated git worktree
4. The agent does real work (edits files, runs commands)
5. The agent writes a reflection to `reflection.md` in the worktree
6. The reflection is saved to `reflections/<path>/TIMESTAMP.md`

## Project Structure

```
ken/
├── ken-bootstrap.py      # The bootstrap script (parser + prompt + agent spawn)
├── kens/                  # Kenning definitions
│   ├── beginning/
│   │   └── kenning.md
│   ├── cli/
│   │   └── wake/
│   │       └── kenning.md
│   └── test/
│       └── kenning.md
└── reflections/           # Saved reflections (created as agents run)
```

## The Bootstrap Loop

1. Use `ken-bootstrap.py` to wake agents with structured context
2. Agents do real work in isolated worktrees
3. Read reflections, improve kennings based on what agents report
4. Repeat until `ken` is self-hosting

## Writing Kennings

Kenning format:

```markdown
# Kenning: path/to/ken

## Name
Short Human-Readable Label

## Meta
parent: parent-path
version: 1

---

## Frame 1: Title
Your prompt here. Ask generative questions.
Make the agent produce understanding, not just receive it.

## Frame 2: Title
Build on Frame 1. Go deeper.

## Frame 3: Title
Current context. What exists. What's the state.
```

## Reading Reflections

After each session, read the reflection in `reflections/`. 

Key questions:
- Did the frames prepare the agent well?
- What was missing?
- What should future agents know?

Use this to improve your kennings.

## Limitations of Bootstrap

- Frame walking is single-prompt (not multi-turn conversation)
- No session recovery (don't interrupt)
- No dynamic injection (no `{{file:...}}` templates)
- No navigation (`ken context up/down`)
- No kenning improvement tooling

These come later, once we validate the core concept works.

## Success Criteria

The bootstrap is successful when:
- [x] We can run a `ken` session
- [ ] The agent produces useful work
- [ ] The reflection contains actionable insights
- [ ] We've used `ken` to build part of `ken` itself
