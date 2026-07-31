## What this changes

<!-- One or two lines. Which course/lesson/path, and what a learner sees differently. -->

## Salvage ledger

Every PR that retires or replaces prior content records where each piece went. Delete the table and
write **`Salvage: none — all-new content`** if this PR reuses nothing.

| Prior lesson / asset | CARRY · PROSE-ONLY · DELETE | Destination (or why not ported) |
| --- | --- | --- |
|  |  |  |

- [ ] Nothing on the **DELETE — DO NOT PORT** list is ported, cited as prior art, or "fixed in place"
      (`docs/superpowers/specs/2026-07-26-course-salvage-ledger.md` §7, in the app repo).
- [ ] Carried prose was re-authored against the current stack — a `versionStamp` is **never** inherited.

## Checklist

- [ ] **Originality.** Every word and every line of code here is original, or is adapted from a
      corpus that permits it (LiteSVM, Mollusk, Surfpool — Apache-2.0; Trident — MIT) **with
      attribution in the lesson**. Nothing is lifted from the Solana docs/cookbook (GPL-3.0) or from
      any all-rights-reserved corpus.
- [ ] `validate-content` is green — 0 errors (see [CONTRIBUTING.md](../CONTRIBUTING.md) to run it locally).
- [ ] Schema-valid: ids unchanged, `slots.lock.json` not hand-edited, `xpPerLesson × lessonCount ≤ 10000`.
- [ ] Code-bearing lessons carry a `versionStamp` with the versions they actually teach and today's `checkedAt`.
- [ ] No `content.lock` bump here — that lives in the app repo. **Merging this stages the content; it
      does not ship it.**
- [ ] Every quoted statistic, price or market claim is **dated** in the prose.

Closes #
