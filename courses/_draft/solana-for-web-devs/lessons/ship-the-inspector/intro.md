# Ship the Inspector

> Version stamp — `@solana/kit` 7.0.0 · devnet · authored 2026-07-28.

No new API in this lesson. You have every part: the RPC read from lesson 2, the byte decode from lesson 2, the derivation from lesson 3, rent-exemption from lesson 1. Today you assemble them into the module's deliverable — an **account inspector** — and the assembly is where the engineering lives.

## The spec

`inspect(address)` answers, for **any** devnet address:

- **type** — one of `missing`, `system-owned`, `program`, `vault`, `other`;
- **lamports** and **sol** — the balance, exact (bigint) and human-readable;
- **ownerProgram** — who may modify it;
- **rentExempt** — does the balance clear `getMinimumBalanceForRentExemption(dataLength)`;
- **vault** — the decoded `{ owner, balance, bump }`, **but only when it really is a vault**: owned by the vault program *and* carrying the `VaultState` discriminator with the exact 49-byte layout. Otherwise `null`.

## Acceptance criteria: the failure paths

Paste-any-address means the input is hostile by default. The inspector's grade lives in what it does when the account is *not* the happy path:

1. **The address does not exist** — RPC returns `{ value: null }`. Report `missing`; never touch fields that are not there.
2. **The account exists but has no data** — a plain wallet: zero-length data, system-owned. There is nothing to decode, and trying is a bug.
3. **The account is a program** — `executable` is set; its bytes are code, not state.
4. **The account is owned by some other program** — a sysvar, a token account, anything. Not yours to decode: `other`.
5. **The account is owned by the vault program but is not a VaultState** — wrong length or wrong discriminator. This is the trap lesson 2's quiz warned about: a blind decode *succeeds mechanically* and returns confident garbage. The discriminator check is what stands between your inspector and lying.

Order matters: check existence before anything, executability and ownership before shape, shape before decode. A branch order that "mostly works" is how inspectors end up reporting a sysvar as a vault.

## Two rungs

**Rung 1 — the branch table.** `classifyAccount(acc)` returns just the type. The lines are in a parts bin, scrambled, with two decoys: one classifies any vault-program-owned account as a vault without checking its shape (criterion 5's bug, made flesh), and one declares zero-length accounts `missing` (an existing empty account is not missing — criterion 2 vs 1).

**Rung 2 — the whole inspector.** `inspect(acc, minRentFor)` — written by you from the spec, no pattern shown, no scaffold. Your rung-1 logic is the spine; add the balance report, rent-exemption, and the guarded decode (your lesson-2 decoder, now with the discriminator check in front). Rent minimums are injected as a recorded-real lookup, and the fee fixtures include a vault-program-owned account that is **not** rent-exempt — the check must use the real minimum for *that* account's data length, not a constant.

Both rungs grade over **recorded real devnet accounts** — the reference vault, a second vault instance, a wallet, the frozen program itself, and a sysvar — plus one synthetic layout probe (a truncated vault-owned account; details in the fixture comments). The set is exactly the taxonomy above; when it passes, your inspector has handled every failure path this course knows how to produce.

Then the quiz asks you to predict failure paths, and the reflection asks which one taught you something.
