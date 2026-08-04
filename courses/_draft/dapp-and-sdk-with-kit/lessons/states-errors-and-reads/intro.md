# Pending, Confirmed, Failed — and What the Chain Actually Says

The app sends and pays correctly. Now make it honest: drive the UI from real confirmation state and decoded program errors, not from a spinner that guesses.

This is the course's last technical rung, and it is **independent-write**. Two courses ago you wrote `vault_core.rs` and a hardened Anchor program unaided; you get a spec here, not a pattern.

## The commitment ladder

A transaction advances `processed → confirmed → finalized`, and each rung is honest to display differently. `processed` means a validator saw it; `confirmed` means a supermajority voted on its block; `finalized` means it cannot be rolled back. Show what is true — do not paint "confirmed" the moment you hit send.

In Kit the confirmation path is `sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })` plus `getSignatureFromTransaction`. **There is no `connection.confirmTransaction` in Kit** — the old catalog's confirmation code cannot be ported line for line. Reads come from your generated client: `fetchVault` and `fetchMaybeVault`.

## The error taxonomy, each mapped to a UI state

Not every failure is a failed transaction. Map each to what is honest:

- **User rejected** — the wallet said no. This never reached the network, so **do not show a failed transaction.** Return the UI to its resting state.
- **Blockhash expired** — the transaction aged out. This is **retryable**: rebuild with a fresh blockhash and resend.
- **Preflight failure** — simulation rejected it before it went out. This is **your bug** — surface the logs.
- **Custom program error** — your program returned a `VaultError`. Decode it with the **generated error map** from your module-1 Codama client — the concrete payoff of generating a client instead of hand-writing one. It is a failure, with a real reason.
- **Insufficient lamports** — the fee payer cannot cover the transaction. A failure.

## Explorer links

When you link a transaction to the explorer, drive the cluster from the app's environment — never hardcode `cluster=mainnet-beta` on what are devnet links. That single hardcode is why so many devnet apps link to a transaction that "does not exist".

## The exercise

Write `nextUiState(current, event)` — a pure reducer that takes the current UI state and one lifecycle-or-error event and returns the next state. The spec is in the starter; there is no pattern shown. It is graded on hidden fixtures. After it passes, ship the app and record the URL.
