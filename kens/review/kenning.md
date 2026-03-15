# Kenning: review

## Name
Code Review

## Meta
version: 1

---

## Frame 1: Paranoid Review

Run `git diff main` to see every change on this branch compared to main.

Now give a paranoid, pedantic code review. Nitpick everything:

- Naming: Are names precise? Misleading? Too vague?
- Correctness: Are there off-by-one errors, race conditions, unhandled edge cases?
- Security: Any injection vectors, unchecked inputs, leaked secrets?
- Style: Inconsistencies with surrounding code? Unnecessary complexity?
- Missing tests: What isn't tested that should be?
- Unclear intent: Would a reader six months from now understand why this code exists?
- Silent failures: Are errors swallowed? Are return values ignored?

Assume every line is guilty until proven innocent. If something looks fine, explain exactly why it's fine — don't just skip it.

---

## Frame 2: Double Down

Now re-examine your review from Frame 1, point by point.

For each issue you raised, commit to a verdict:

- **AGREE** — the concern is real, stands as written, and should be addressed.
- **REFUTE** — the concern was wrong, overly cautious, or misguided. Explain why you were wrong.

No hedging. No "partially agree." Pick one. If you agree, sharpen the critique. If you refute, own the reversal and say what you missed the first time.
