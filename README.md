# Ken Bootstrap Project

This is the minimal viable implementation to test and validate the ken concept.

## Quick Start

```bash
# Install dependency
pip install anthropic

# Set API key
export ANTHROPIC_API_KEY=your-key-here

# Run your first ken session
python ken-bootstrap.py core/kenning-parser --task "implement the frame parser in src/kenning.py"
```

## What Happens

1. The script loads `kens/core/kenning-parser/kenning.md`
2. It walks Claude through each frame (you'll see the conversation)
3. It delivers your task
4. Claude works on the task
5. It prompts Claude for a reflection
6. The reflection is saved to `reflections/core/kenning-parser/TIMESTAMP.md`

## Project Structure

```
bootstrap-project/
├── ken-bootstrap.py      # The bootstrap script
├── kens/                  # Kenning definitions
│   ├── core/
│   │   └── kenning-parser/
│   │       └── kenning.md
│   └── cli/
│       └── wake/
│           └── kenning.md
├── reflections/           # Saved reflections (created as you use it)
└── src/                   # Your code output (you create this)
```

## The Bootstrap Loop

1. Use `ken-bootstrap.py` to build `src/kenning.py` (the parser)
2. Use `ken-bootstrap.py` to build `src/wake.py` (the wake command)
3. Now you have a better ken, use it to build more of ken
4. Repeat until ken is complete

## Writing Kennings

Kenning format:

```markdown
# Kenning: path/to/ken

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

- Agent can't actually write files (just describes code)
- No session recovery (don't interrupt)
- No dynamic injection (no `{{file:...}}` templates)
- No navigation (`ken context up/down`)
- No evolution system

These come later, once we validate the core concept works.

## Success Criteria

The bootstrap is successful when:
- [x] We can run a ken session
- [ ] The agent produces useful work
- [ ] The reflection contains actionable insights
- [ ] We've used ken to build part of ken itself
