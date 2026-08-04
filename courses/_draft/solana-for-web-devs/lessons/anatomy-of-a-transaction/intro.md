# Anatomy of a Transaction

> Version stamp — `@solana/kit` 7.0.0 · devnet · authored 2026-07-28.

You have only read the chain. Writing to it takes a **transaction** — and before you send your first one in lesson 6, you are going to take a complete one apart and predict, to the lamport, what it costs.

## A complete Kit transaction, end to end

This is the whole pipeline for a v0 transaction — a SOL transfer, built with `pipe` (Kit's left-to-right function composition):

```ts
import {
  createSolanaRpc,
  generateKeyPairSigner,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
  signTransactionMessageWithSigners,
  getBase64EncodedWireTransaction,
  pipe,
} from "@solana/kit";
import { getTransferSolInstruction } from "@solana-program/system";

const rpc = createSolanaRpc("https://api.devnet.solana.com");
const signer = await generateKeyPairSigner();
const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const message = pipe(
  createTransactionMessage({ version: 0 }),
  (m) => setTransactionMessageFeePayerSigner(signer, m),
  (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
  (m) =>
    appendTransactionMessageInstructions(
      [getTransferSolInstruction({ source: signer, destination, amount: 1_000_000n })],
      m
    )
);

const signed = await signTransactionMessageWithSigners(message);
const wire = getBase64EncodedWireTransaction(signed);
const sim = await rpc.simulateTransaction(wire, { encoding: "base64" }).send();
```

Read it as four declarations, not four steps of ceremony:

1. **Version** — `{ version: 0 }`. Say "**v0 transactions**", not "the only version": Kit also has v1 messages, with a different fee surface (`setTransactionMessagePriorityFeeLamports`, a total in lamports). Everything below — the `price × CU` arithmetic included — is **v0-specific**, and v0 is what this course and the current ecosystem run on.
2. **Fee payer** — the signer whose lamports pay for this transaction.
3. **Lifetime** — a recent blockhash. The network accepts the transaction only while that blockhash is recent (about a minute); after that it can never land, which is what makes retries safe.
4. **Instructions** — the actual work, appended last.

Then sign, encode, and — before ever sending — **simulate**. Simulation runs the transaction against current state without landing it and reports `unitsConsumed`: how much compute it actually used. We ran exactly this simulation against the reference vault's `deposit` on 2026-07-28 and recorded **6,697 compute units** (a bare SOL transfer: 150).

## What it costs — the exact formula

A v0 transaction's fee has two parts:

```
fee = 5000 lamports × signatures                    ← base fee (50% burned)
    + priorityFeeMicroLamportsPerCU × requestedCUlimit ÷ 1,000,000   ← priority fee
```

The word that costs people real money is **requested**. The priority fee is charged on the compute-unit limit your transaction *asks for* — not on what it consumes. And if you don't ask, each non-builtin instruction requests the **default 200,000 CU** (clamped at 1,400,000 per transaction). A deposit that consumes 6,697 CU but requests 200,000 pays the priority rate on all 200,000.

That is why the professional habit is **simulate, then tighten**: request `unitsConsumed` plus ~10% margin (compute varies slightly run to run; a limit set exactly at one observed value occasionally fails with an exceeded-CU error).

You request a limit (and a price) with two extra instructions from `@solana-program/compute-budget`: `getSetComputeUnitLimitInstruction({ units })` and `getSetComputeUnitPriceInstruction({ microLamports })`. Two honest footnotes on those:

- Putting the compute-budget instructions **first is a convention, not a protocol rule**. The runtime finds them anywhere in the transaction. The real constraints are: at most one of each kind (a duplicate fails with `DuplicateInstruction`), and the 1,400,000-CU clamp.
- For production estimation, Kit's current API family is `fillTransactionMessageProvisoryResourceLimits` at message construction, `estimateResourceLimitsFactory({ rpc })` to estimate, and `estimateAndSetResourceLimitsFactory(estimator)` to apply. The older compute-unit-named trio (`fillTransactionMessageProvisoryComputeUnitLimit`, `estimateComputeUnitLimitFactory`, `estimateAndSetComputeUnitLimitFactory`) is **deprecated** — you will see it in tutorials; don't build on it. Note the resource estimator returns `{ computeUnitLimit, loadedAccountsDataSizeLimit }` — an object, not a bare number.

## The exercise

`planBudget(sim, price)` is a **worked example with exactly one blank**. It takes a recorded simulation result and a priority-fee price and returns the full cost plan: the requested limit, the 5,000-lamport signature fee (one signer), the priority fee (rounded **up** — the network never rounds in your favor), and the total. Everything is written except one line: the requested limit, sized from `sim.unitsConsumed` plus a 10% margin, in integer bigint arithmetic (`consumed + consumed / 10n`).

Fill the blank, then run the quiz and predict the costs the way you now can: on paper, before anything runs.
