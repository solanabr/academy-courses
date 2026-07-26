# Token or Token-2022: The Decision Rule

> **Version stamp — checked 2026-07-25.** `@solana/kit@7.0.0` · `@solana/subscriptions@0.4.0` (exact) · `@x402/svm@2.19.0` · `@solana-program/token` latest `0.15.0`, `@solana-program/token-2022` latest `0.13.0`. Every dependency range quoted below was read off the published package manifests on that date.

This is a checkpoint, not new material. You are about to open a delegation against a specific mint, and the program will either take that mint or refuse it. Before you write the call, get the rule straight.

## Two token programs, not two eras

| | Program id |
| --- | --- |
| **Token** (the original) | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| **Token-2022** (token extensions) | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` |

Older material — including an earlier version of this very curriculum — frames these as "the mature one" and "the new one with limited support". That framing is dead. Weekly npm downloads for the two client libraries sit within a couple of percent of each other, DEX and wallet support is at parity, and Token-2022 has been carrying production stablecoin supply for years. Neither program is a legacy of the other. They are **two mints with different capability surfaces**, and you choose per mint.

The one thing you cannot do is choose late. A mint's owner program is fixed at creation and every downstream address depends on it.

## Why the choice changes your addresses

The associated token account is a PDA whose seeds are `[owner, tokenProgram, mint]`. The token program id is **in the seeds**. So the same wallet holding the same nominal asset has a different ATA depending on which token program minted it.

```ts
import { findAssociatedTokenPda } from "@solana-program/token";

const [ata] = await findAssociatedTokenPda({
  mint,
  owner,
  tokenProgram, // Token or Token-2022 — this changes the resulting address
});
```

Two notes on that snippet, both of which are corrections to material you may have read here before:

1. **`getAssociatedTokenAddress` from `@solana/spl-token` is not the current API.** The Kit-native derivation is `findAssociatedTokenPda`, it is synchronous-looking but returns a promise, and it returns a `[address, bump]` tuple — not a bare address. Destructure it.
2. **`tokenProgram` is a required input, not an optional flourish.** Passing the wrong one silently derives a real, valid, empty account that will never receive the transfer you are expecting.

### The version seam you will actually trip over

`@solana-program/token` is at `0.15.0`. Under semver, a caret range on a `0.x` package pins the **minor**, not the major — `^0.9.0` means `>=0.9.0 <0.10.0`. Read off the published manifests on 2026-07-25:

| Package | Range it declares on `@solana-program/token` |
| --- | --- |
| `@x402/svm@2.19.0` | `^0.9.0` |
| `@solana/subscriptions@0.4.0` | `^0.13.0` |
| `@solana/pay@1.0.23` | `^0.12.0` (a **peer**) |

Those three ranges are pairwise disjoint. For `@x402/svm` and `@solana/subscriptions` this is harmless — they are ordinary dependencies, so your package manager installs a copy each and nobody argues. For `@solana/pay` it is a **peer** dependency, which is your problem to satisfy, and it is one of the reasons Solana Pay is a decision in this course rather than an install.

Pin `@solana-program/token` yourself, at whatever version your own code imports, and do not assume the one your dependency resolved is the one you got.

## The rule

> **Legacy Token by default. Token-2022 only when a named extension is required. Then verify your rail accepts it.**

Three clauses, and the third is the one everybody skips.

There are around 28 Token-2022 extensions. This course is not going to tour them — Blueshift already owns that material and does it well. What matters here is that **your payment rail gets a veto**, and the Subscriptions Delegation Program uses it.

## What the rail refuses

Read the error constants shipped in `@solana/subscriptions@0.4.0` and a whole family jumps out:

| Error constant (suffix) | Message |
| --- | --- |
| `MINT_HAS_CONFIDENTIAL_TRANSFER` | Mint has ConfidentialTransfer extension |
| `MINT_HAS_MINT_CLOSE_AUTHORITY` | Mint has MintCloseAuthority extension |
| `MINT_HAS_NON_TRANSFERABLE` | Mint has NonTransferable extension |
| `MINT_HAS_PAUSABLE` | Mint has Pausable extension |
| `MINT_HAS_PERMANENT_DELEGATE` | Mint has PermanentDelegate extension |
| `MINT_HAS_TRANSFER_FEE` | Mint has TransferFee extension |
| `MINT_HAS_TRANSFER_HOOK` | Mint has TransferHook extension |

Seven extensions that can disqualify a mint from a delegation. Look at *why* each one is there and the list stops being arbitrary:

- **NonTransferable** — a delegation is a promise that somebody else can move your tokens. A mint that forbids transfers cannot honour it.
- **PermanentDelegate** — the mint authority already holds an irrevocable delegate over every account. A user-scoped, user-revocable allowance is a fiction on top of it.
- **Pausable / MintCloseAuthority** — a third party can invalidate the arrangement out from under both parties.
- **TransferFee** — the amount that lands is not the amount authorised, so a cap stops meaning what it says.
- **ConfidentialTransfer** — the balances the program must check are encrypted.
- **TransferHook** — an arbitrary program runs inside the transfer, so the extra accounts it needs must be discovered and appended before the instruction is built. The SDK ships `resolveTransferHookAccounts` for exactly that; hook support is therefore **path-dependent**, which is the strongest possible argument for the rule's third clause. Do not assume. Test on devnet against the rail you are actually going to use.

The two errors you will hit most in module 1 have nothing to do with extensions, and are worth memorising now: `AMOUNT_EXCEEDS_LIMIT` ("Transfer amount exceeds delegation limit") for a fixed delegation, and `AMOUNT_EXCEEDS_PERIOD_LIMIT` ("Transfer amount exceeds period limit") for a recurring one. Lesson 4 is built on the second.

## The exercise

Write the decision rule as a function. It takes what a decoded mint tells you — which program owns it, which extensions are configured on it — plus the one extension your product actually needs, and it returns a verdict.

Check the disqualifying extensions in this fixed order, so the verdict is deterministic when a mint carries more than one:

```
confidential-transfer, mint-close-authority, non-transferable,
pausable, permanent-delegate, transfer-fee, transfer-hook
```

Verdict codes:

| Code | Meaning |
| --- | --- |
| `INVALID_TOKEN_PROGRAM` | Not owned by either token program. Not a payment mint. |
| `MINT_HAS_*` | The rail refuses this mint. Names the first disqualifying extension in the order above. |
| `design-error-extension-requires-token-2022` | You asked for an extension on a legacy Token mint. That is not a thing. |
| `design-error-required-extension-missing` | Token-2022 mint, but the extension you need is not configured on it. |
| `ok-legacy-token` | Correct default. |
| `ok-token-2022-extension-required` | Token-2022, justified by a named extension the rail accepts. |
| `ok-token-2022-not-justified` | The rail will take it, but you had no reason to leave legacy Token. |

Note that the last one is `accepted: true`. The rail is not your code reviewer. A verdict of "this works and you still chose wrong" is exactly the kind of thing a decision rule should be able to say.
