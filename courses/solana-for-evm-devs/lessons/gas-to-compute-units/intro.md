# Gas splits into two separate things

> Version stamp — `@solana/kit` 7.0.0 · `@solana-program/compute-budget` 0.17.0
> · authored 2026-07-30.

On the EVM, one number does two jobs. Gas meters how much work you did *and*, via
gas price, how badly you want to be included. Raise the price and you buy
priority; run an expensive loop and you pay more for the same priority.

Solana separates them.

**Compute units (CU)** meter work. Every instruction has a budget — 200,000 CU by
default per instruction, up to 1.4M for a transaction if you ask. Exceed it and
the transaction fails. Crucially, the CU limit is *not* priced per unit the way
gas is: the base fee is a flat 5,000 lamports per signature, essentially
independent of how much computation you did.

**Priority fees** buy ordering. You bid in micro-lamports per compute unit,
separately, and only that bid competes for block space when there is contention.

Two consequences that will surprise you:

- **Compute is cheap; state is expensive.** The EVM's `SSTORE`-dominated cost
  model taught you to minimize writes above all. On Solana the recurring cost is
  rent for space, and computation inside your budget is close to free. An
  optimization that saves 20,000 CU saves you approximately nothing in fees — it
  buys headroom, not money.
- **Running out of CU is a failure, not a fee increase.** There is no "raise the
  limit and it costs more" gradient. You either fit in the budget or the
  transaction fails, so requesting an appropriate limit is part of building the
  transaction.

## Setting them

Both are instructions from the Compute Budget program, added to your
transaction:

```ts
import {
  getSetComputeUnitLimitInstruction,
  getSetComputeUnitPriceInstruction,
} from "@solana-program/compute-budget";

getSetComputeUnitLimitInstruction({ units: 300_000 });
getSetComputeUnitPriceInstruction({ microLamports: 5_000 });
```

The first says how much room you need. The second is your priority bid. They are
plain instructions like any other — you append them to a transaction message and
they carry no accounts.

Two honest footnotes. Putting them first is a **convention, not a protocol
rule**: the runtime finds them anywhere in the transaction. The real constraints
are at most one of each kind (a duplicate fails with `DuplicateInstruction`) and
the 1.4M-CU clamp. And for *estimating* the limit rather than hardcoding it, the
current API lives in Kit itself — `estimateResourceLimitsFactory({ rpc })` and
`estimateAndSetResourceLimitsFactory(estimator)` — not in this package.

> **In the wild (checked 2026-07-30).** The snippet you will find in nearly
> every tutorial is `ComputeBudgetProgram.setComputeUnitLimit({...})` from
> `@solana/web3.js` v1, a static class method. Same two instructions, superseded
> API. Read it; write the builders above.

## The arithmetic worth internalizing

The priority fee is `computeUnits × microLamportsPerCu`, converted from
micro-lamports to lamports — a division by 1,000,000.

A 200,000 CU transaction bidding 5,000 micro-lamports/CU pays
`200,000 × 5,000 / 1,000,000 = 1,000` lamports of priority, on top of the 5,000
lamport base signature fee.

Notice what that means: **the limit you request affects what you pay.** Request
1.4M CU "just to be safe" while bidding the same price and you have multiplied
your priority fee by seven for a transaction that uses a fraction of it. The
correct move is to simulate the transaction, observe the units actually consumed,
and request that plus a modest margin.

You will implement exactly that conversion next.
