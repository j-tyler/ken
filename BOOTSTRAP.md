# Ken MVP: Bootstrap Plan

## The Goal

Build just enough of ken that we can use ken to finish building ken.

This is the bootstrapping milestone. After this, every improvement to ken is made using ken.

---

## What "Using Ken to Build Ken" Actually Means

A session looks like this:

```bash
# Human decides what needs to be built next
ken wake cli/wake --task "implement the frame-walking loop"

# Under the hood:
# 1. Ken loads the kenning for cli/wake
# 2. Ken spawns an AI agent (Claude Code or API)
# 3. Ken walks the agent through frames (agent generates responses)
# 4. Ken delivers the task
# 5. Agent works (writes code, runs tests)
# 6. Ken prompts for reflection
# 7. Ken saves reflection to file
# 8. Session ends

# Human reviews the work, decides next task
ken wake cli/reflect --task "implement reflection file writing"
# ... and so on
```

The human is the orchestrator. Ken handles the wake cycle. The agent does the work.

---

## Absolute Minimum for Bootstrap

### Must Work (No Shortcuts)

**1. Project structure exists**

We need somewhere to put kennings and reflections. This can be created manually for bootstrap — `ken init` is nice-to-have, not essential.

```
ken-project/
  kens/
    cli/
      init/
        kenning.md
      new/
        kenning.md
      wake/
        kenning.md
      reflect/
        kenning.md
    core/
      kenning-parser/
        kenning.md
      session/
        kenning.md
  reflections/
    (created as we go)
```

**2. Kennings exist and are parseable**

We write the kennings by hand. The system must be able to read a kenning.md and extract frames.

Minimal parser:
- Read markdown file
- Split on `## Frame N:` headers
- Extract prompt text from each section
- Ignore metadata for now (parent, peers, version)

**3. Agent communication works**

This is the hard unknown. We must be able to:
- Start a conversation with an AI
- Send a prompt
- Receive complete response
- Send another prompt (in same context)
- Repeat

Options (in order of preference):
1. **Claude Code CLI** — if it supports scripted input
2. **Anthropic API directly** — we manage context ourselves
3. **Manual bridge** — script prints prompt, human pastes, human pastes response back

For true bootstrap, option 3 is acceptable. Ugly but functional.

**4. Frame walking loop**

```
for each frame in kenning:
    send frame.prompt to agent
    receive response
    (response is now in agent's context)
send task prompt
agent works
send reflection prompt  
save reflection to file
```

**5. Reflection capture**

At minimum: agent's reflection response gets saved to `reflections/{ken-path}/{timestamp}.md`

Doesn't need structured parsing. Just capture the text.

---

### Can Be Manual/Hacky for Bootstrap

| Feature | Bootstrap Version | Real Version |
|---------|-------------------|--------------|
| `ken init` | Create folders by hand | Automated |
| `ken new` | Copy a template kenning.md | Generates starter |
| `ken tree` | `find kens -name kenning.md` | Pretty printed tree |
| Kenning metadata | Ignore parent/peers/version | Full parsing |
| Dynamic injection (`{{file:...}}`) | Manual: paste file contents into frame | Automated injection |
| Reflection structure | Freeform text | Structured sections |
| Session persistence | None (don't crash) | .ken/session.json |
| `ken context up/down/peers` | Read the files yourself | Automated |

---

## The Bootstrap Implementation Plan

### Phase 0: Manual Proof of Concept (Day 1)

Before writing any code, test if kennings work at all.

1. Write a kenning for one part of ken (e.g., the kenning parser)
2. Open Claude Code manually
3. Copy/paste Frame 1, let it respond
4. Copy/paste Frame 2, let it respond
5. Continue through all frames
6. Paste the task
7. Let it work
8. Paste reflection prompt
9. Save its reflection

**Question we're answering:** Does walking through frames actually produce better work than cold-starting with just a task?

If yes: proceed to automation.
If no: we have a design problem, not an implementation problem.

### Phase 1: Minimal Automation (Days 2-4)

Build just enough to automate the tedious parts.

**Deliverable: `ken wake {path} --task "..."`**

What it does:
1. Reads `kens/{path}/kenning.md`
2. Parses frames (simple regex/split)
3. Connects to AI (API or Claude Code)
4. Walks frames, capturing responses
5. Sends task
6. Waits for completion signal (could be manual: "press enter when done")
7. Sends reflection prompt
8. Saves response to `reflections/{path}/{timestamp}.md`

What it doesn't do yet:
- Create projects (`ken init`)
- Create kens (`ken new`)
- Pretty print (`ken tree`)
- Dynamic injection
- Session recovery
- Anything about evolution

**Language choice for Phase 1: Python**

Reasoning:
- anthropic SDK is Python-native
- Faster iteration while we figure out agent communication
- Can rewrite in Rust once design stabilizes

```python
# Sketch of the core loop
def wake(ken_path: str, task: str):
    kenning = parse_kenning(f"kens/{ken_path}/kenning.md")
    
    client = anthropic.Client()
    messages = []
    
    # Walk frames
    for frame in kenning.frames:
        messages.append({"role": "user", "content": frame.prompt})
        response = client.messages.create(
            model="claude-sonnet-4-20250514",
            messages=messages,
            max_tokens=4096
        )
        assistant_msg = response.content[0].text
        messages.append({"role": "assistant", "content": assistant_msg})
        print(f"[Frame {frame.number} complete]")
    
    # Deliver task
    messages.append({"role": "user", "content": f"## Task\n\n{task}"})
    # ... agent works ...
    
    # Reflection
    messages.append({"role": "user", "content": REFLECTION_PROMPT})
    response = client.messages.create(...)
    save_reflection(ken_path, response.content[0].text)
```

**The Claude Code question:**

If we want the agent to actually write files and run commands (not just chat), we need Claude Code or computer use, not just the messages API.

Options:
1. Use API for frames, then hand off to Claude Code for work
2. Figure out Claude Code's programmatic interface
3. Use computer use API (complex)
4. Accept that bootstrap MVP is chat-only (agent describes what to do, human executes)

**Recommendation:** Start with option 4. Get the frame-walking working. Then upgrade to actual code execution.

### Phase 2: Self-Hosting (Days 5-7)

Once `ken wake` works, use it to build the rest of ken.

```bash
# Create the ken structure for the ken project (manually for now)
mkdir -p kens/{cli,core}/{init,new,tree,wake,reflect,kenning-parser,session}

# Write kennings for each component (manually)
# ... write kens/cli/init/kenning.md etc ...

# Now use ken to build ken
ken wake core/kenning-parser --task "implement markdown parser that extracts frames"
ken wake cli/init --task "implement ken init command"
ken wake cli/new --task "implement ken new command"
ken wake cli/tree --task "implement ken tree command"
# wake is already built, but we can improve it:
ken wake cli/wake --task "add session persistence for crash recovery"
ken wake cli/reflect --task "implement structured reflection parsing"
```

Each session produces a reflection. We read reflections to improve kennings manually.

### Phase 3: Polish (Week 2)

Still using ken, add quality-of-life:
- `ken init` automated
- `ken new` with starter templates
- `ken tree` pretty printing
- Dynamic injection (`{{file:...}}`)
- Better error handling
- Session persistence

### Phase 4: Evolution System (Week 3+)

Only after we have real reflections from real use:
- `ken journal` to review reflections
- `ken evolve` to propose improvements
- `ken trial` to test them
- `ken adopt` to accept them

---

## What We'll Learn By Using It

These are things we explicitly defer deciding until we have experience:

| Question | How We'll Learn |
|----------|-----------------|
| How many frames is right? | Try different counts, see what reflections say |
| What frame types work best? | Experiment, read reflections |
| Should grounding be one frame or woven throughout? | Try both approaches |
| Is the reflection prompt right? | Read reflections, see if they're useful |
| Do we need navigation (up/down/peers)? | Feel the pain of not having it (or not) |
| How should multi-ken tasks work? | Hit the wall, then design |
| Is Python fast enough or do we need Rust? | See if startup time bothers us |
| What should the evolution system actually do? | Manually evolve kennings first, then automate the pattern |

---

## Success Criteria for Bootstrap Milestone

We've achieved bootstrap when:

1. ✓ We can run `ken wake {path} --task "..."` and it works
2. ✓ The agent walks through frames and arrives at understanding
3. ✓ The agent completes the task (even if just describing what to do)
4. ✓ A reflection is saved
5. ✓ We've used ken to implement at least one feature of ken
6. ✓ The reflection from that session is useful (we learned something)

At that point, ken is self-hosting. Every future improvement uses ken.

---

## Immediate Next Steps

1. **Write the first real kenning** — for the kenning parser itself
2. **Manual test** — walk through it by hand in Claude Code
3. **Implement minimal `ken wake`** — Python, API-based, chat-only
4. **Use it** — build the next component using ken
5. **Iterate** — improve based on experience

---

## File: kens/core/kenning-parser/kenning.md (First Real Kenning)

Here's the first kenning we'd use to bootstrap:

```markdown
# core/kenning-parser

## Frame 1: What You're Building

You're building the kenning parser for a tool called ken.

A kenning is a markdown file containing ordered "frames" — prompts designed 
to rebuild understanding in an AI agent. The parser reads this markdown 
and extracts the frames as structured data.

Before I explain the format, answer: if you were designing a simple 
markdown format for ordered prompts, what would it look like? What 
headers or markers would you use?

## Frame 2: The Format

The format we've chosen:

- Frames are marked by `## Frame N: Title` headers
- Everything after the header until the next frame (or end) is the prompt
- There's a metadata section at the top (## Meta) that we ignore for MVP
- There may be a `## Task` section that's not a frame (injected at runtime)
- There may be a `## Reflection` section that's not a frame

Example:

## Meta
parent: core
version: 1

## Frame 1: Orientation
{prompt text here}

## Frame 2: Deep Dive
{prompt text here}

## Task
{injected at runtime}

Given this, what's your parsing strategy? What data structure would 
you return?

## Frame 3: Implementation Constraints

We're building this in Python for fast iteration. Keep it simple:
- Use regex or string splitting (no heavy markdown library needed)
- Return a list of Frame objects with: number, title, prompt
- Handle edge cases: no frames, malformed headers, extra whitespace
- Ignore sections that aren't frames (Meta, Task, Reflection)

What edge cases are you thinking about?

## Frame 4: Current State

The project structure:

ken/
  ken.py          # Main entry point (stub)
  kenning.py      # <- You're implementing this
  session.py      # Session management (stub)
  
No dependencies yet beyond standard library. Add what you need.

Your task will be given next.
```

---

## Risk Mitigation

**Risk: Claude Code integration is harder than expected**

Mitigation: Start with Anthropic API directly. Agent describes code changes, human applies them. Ugly but unblocks bootstrap.

**Risk: Kennings don't actually help**

Mitigation: Manual test on Day 1 before writing any automation. If frames don't produce better understanding, we need to redesign, not implement faster.

**Risk: Scope creep**

Mitigation: This document. Bootstrap is: wake + reflect, nothing else. Everything else waits until we're self-hosting.

**Risk: Premature optimization**

Mitigation: Python first. Rewrite in Rust only if Python is actually too slow for the workflow.

---

## Summary

**What must work:** `ken wake {path} --task "..."` that walks frames and saves a reflection.

**What can be manual:** Everything else. Project creation, ken creation, tree viewing, navigation, evolution.

**What we'll learn by using:** Optimal frame count, frame types, reflection format, whether evolution is even needed.

**How we know we're done:** We used ken to build part of ken, and the reflection was useful.

Let's go.
