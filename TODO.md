# TODO

## Contract + Validation Follow-ups

- [x] Add deterministic validation fixtures under `tests/validation-fixtures/` for:
  - missing bind object
  - type mismatch
  - field-spec key mismatch (`KEN_CONTRACT_INVALID`)
  - stable multi-violation ordering
- [ ] Harmonize command names across all docs whenever command surface changes.
- [ ] Keep README explicitly marked as bootstrap-only and cross-link to canonical spec docs.
- [ ] Add a contributor note defining the canonical CLI source of truth (`QUICKREF.md` + `DESIGN_REVIEW.md`).

## Bootstrap Rewrite Follow-ups

- [ ] Restore multi-turn frame walking (currently flattened to single prompt due to `claude -p` being single-turn). Investigate `claude --resume` or tmux-based session management per `TERMINAL_SESSIONS.md`.
- [ ] Update `BOOTSTRAP.md` to reflect the shift from Anthropic API to Claude Code CLI.
- [ ] Update `IMPLEMENTATION.md` Claude Code integration section to document the current `claude -p --worktree` approach.
