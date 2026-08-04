# Simulate, Then Size the Transaction

Your deposit works, and it overpays every time. This lesson kills that.

## The cost bug

A priority fee is charged on the compute-unit limit your transaction **requests**, not on what it actually **consumes**. With no `SetComputeUnitLimit`, every non-builtin instruction requests the default **200,000 CU** (clamped at 1,400,000 per transaction). A deposit uses a small fraction of that — so you pay the priority-fee rate on 200,000 CU while consuming maybe a few thousand. Almost every tutorial ships this at the learner's expense. You are going to measure it against your own lesson-7 transaction.

## Simulate, then set the limit

The fix is: simulate the transaction to learn its real cost, add a small margin, and set the limit to that. In Kit the current path is (version stamp: kit 7.0.0, checked 2026-07-27):

- build the message with `fillTransactionMessageProvisoryResourceLimits` at construction,
- estimate with `estimateResourceLimitsFactory({ rpc })`,
- apply with `estimateAndSetResourceLimitsFactory(estimator)`.

Three cautions that cost people hours:

- The **deprecated** line is the compute-unit-named one: `fillTransactionMessageProvisoryComputeUnitLimit`, `estimateComputeUnitLimitFactory`, `estimateAndSetComputeUnitLimitFactory`. The halves are **not** interchangeable with the resource-limit line.
- The resource estimator returns `{ computeUnitLimit, loadedAccountsDataSizeLimit }` — an object, **not** a bare number. Passing it where a number is expected breaks.
- Do **not** reach for `@solana-program/compute-budget`'s estimators. Kit absorbed them.

Add roughly a **10% margin** over the estimate: compute can vary slightly run to run, and a limit set exactly at one observed value will occasionally fail with an exceeded-CU error. Ten percent absorbs the variance without meaningfully raising the fee.

## The worked example

`feeComparison(consumedUnits, microLamportsPerCu)` shows the whole argument in arithmetic: the fee you pay at the 200,000-CU default, the fee you pay at a tightly-sized limit (consumed + 10%), and the difference. It is complete except for one line — the saving — which you wire from the two fees. Run it against your lesson-7 numbers and watch the gap.
