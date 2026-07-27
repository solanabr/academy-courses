# Price It From the Network, Not From a Blog Post

You sized the compute limit. The other half of the fee is the **price** per compute unit — and the right price is not a number you copy from a tutorial. It is derived from what the network actually charged recently for the accounts you are about to write.

## Scope the query to your writable accounts

`getRecentPrioritizationFees` accepts an array of **writable** account addresses and scopes its answer to contention on exactly those accounts. Pass the accounts your transaction writes — the **vault PDA** and the **fee payer** — not an empty array. An empty array is the mistake that makes every estimate identical and useless: you get a network-wide figure instead of one scoped to the contention you will actually hit.

Apply the result with `setTransactionMessageComputeUnitPrice(microLamports, message)` — a `bigint`. (Note the fork: v1 messages use `setTransactionMessagePriorityFeeLamports`, a total in lamports, instead.)

## Honest framing

Priority fees are optional under normal conditions and only matter under congestion. A hardcoded `10,000` µlamports/CU is a silent overpay on a quiet devnet — which, now that you can *measure* the fee, you can see for yourself. So derive it: take a percentile of the recently observed fees, and **floor** it so a quiet slot (where observed fees are all zero) does not hand you a price of 0 when you do want to be prioritized.

## One of each, only

Compute-budget instructions are transaction-wide, and only **one** of each variant is permitted. Add a second `SetComputeUnitPrice` and the transaction fails with `DuplicateInstruction`.

## The exercise

`priceFee(recentFeesPipe, percentile, floorMicroLamports)` turns the observed fees for your writable accounts into a price, in four numbered subgoals:

1. **Parse** the pipe-joined observed fees. (done for you)
2. **Sort and pick** the value at the chosen percentile.
3. **Floor** it against the minimum so a quiet slot does not produce 0.
4. **Return** the price as `{ microLamports }`.
