# PR Review: claude/codebase-review-dmyKU

## About This Project

**Ken** is a tool for spawning AI agents with structured context. The core idea: instead of dumping documentation on an agent, you walk it through a sequence of "frames" (a "kenning") that build understanding incrementally — like teaching rather than briefing. The agent reconstructs understanding rather than receiving it cold.

The branch refactors ken-bootstrap.py from a direct Anthropic API approach to spawning a Claude Code CLI subprocess in a git worktree, and adds a new `beginning` kenning.

---

## What I Like

- **The core concept is compelling.** Frame-based progressive context loading is a genuinely interesting approach to agent preparation. It's the difference between handing someone a manual vs. walking them through orientation.
- **The rewrite is a clear simplification.** Going from ~310 lines (API client, message management, reflection collection) to ~210 lines (compose prompt, shell out to `claude`) is a good move for an MVP.
- **Worktree isolation is smart.** Agents work in their own worktree, so bad work doesn't pollute the main repo.

---

## Issues

### 1. Reflection capability was dropped without replacement
**Severity: Medium**
**File:** `ken-bootstrap.py`

The old version had `collect_reflection()` and `save_reflection()` — after the agent worked, it would reflect on the kenning quality, and that reflection was persisted. The new version drops this entirely. The README still references reflections (`reflections/` directory, "Reading Reflections" section), and the `cli/wake/kenning.md` Frame 4 is entirely about the reflection system. This is a functional regression with no documented rationale.

### 2. No error handling for `claude` CLI availability
**Severity: Low**
**File:** `ken-bootstrap.py:113-133`

`run_claude()` calls `subprocess.run(["claude", ...])` but if `claude` isn't installed, the user gets a raw `FileNotFoundError`. The old version had clear error messages for missing dependencies (`anthropic` package, API key). A simple check or try/except would improve the UX.

### 3. Prompt is passed as a CLI positional argument
**Severity: Medium**
**File:** `ken-bootstrap.py:116-122`

The composed prompt (which could be very long for multi-frame kennings) is passed as a positional argument to the `claude` command. This can hit OS argument length limits (`ARG_MAX`, typically 128KB-2MB). Should use `--file` or pipe via stdin instead.

### 4. `beginning` kenning is a single open-ended frame
**Severity: Low**
**File:** `kens/beginning/kenning.md`

The frame asks 4 different questions in one prompt ("what is this project about", "your opinion", "what do you like", "what would you try to do"). This goes against the project's own philosophy of progressive frame-based understanding. A single broad frame is essentially a cold-start dump — the thing kennings are supposed to avoid.

### 5. README is out of date
**Severity: Low**
**File:** `README.md`

- Quick Start still says `pip install anthropic` and `export ANTHROPIC_API_KEY` — neither is needed anymore.
- Says "Agent can't actually write files" under Limitations — the whole point of the rewrite is that now it can.
- References `reflections/` workflow that no longer exists in the code.

### 7. `capture_output=True` hides agent progress
**Severity: Low**
**File:** `ken-bootstrap.py:127-131`

Using `capture_output=True` means the user sees nothing while the agent works (which could take minutes). The old API-based approach printed frame-by-frame progress. Consider streaming stdout while still capturing stderr, or at minimum printing a "working..." message.

### 8. `cli/wake/kenning.md` references stale project structure
**Severity: Low**
**File:** `kens/cli/wake/kenning.md:97-110`

Frame 5 references `src/kenning.py`, `src/wake.py`, and a `bootstrap-project/` structure that doesn't match reality. The actual code is `ken-bootstrap.py` at the repo root. An agent woken with this kenning would be confused about where things live.

---

## Summary

The rewrite from API-based to CLI-based is the right direction — it gives agents real tool access instead of just chat. But the branch drops the reflection system (a key differentiator described in the project's own docs) without replacement or acknowledgment, and leaves documentation inconsistent with the new reality. The `beginning` kenning works but is philosophically at odds with the project's frame-by-frame progressive understanding model.

**Recommendation:** Address issues #1, #3, and #5 before merging. The rest are minor improvements.
