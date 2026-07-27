# Ship the Finished App

Wire the reducer into the app: every send drives `nextUiState`, and the view renders from the state — a spinner for `pending`, the explorer link and vault balance for `confirmed`/`finalized`, a retry affordance for `retryable`, the decoded reason for `failed`, and nothing alarming for a plain `idle` after a rejection.

Read the live vault with `fetchMaybeVault` so a not-yet-initialized vault renders as empty rather than throwing, and `fetchVault` once you know it exists.

Redeploy to the same Vercel URL from module 2 — the app grows, it does not restart. Record the URL: it is one of the two artifacts you name in the recap, and Course 5 puts a paid tier on this exact app.
