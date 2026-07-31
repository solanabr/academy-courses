# Your First Deposit, From Your Own App

This is where the two artifacts join. You install the package you published — from the **real registry**, not a local path — and use it to send one real devnet deposit to your own vault, from an app at a public URL.

```
npm i @your-scope/vault-client
```

Installing from a `file:../client` path proves nothing: it only shows the code works on your machine, where you wrote it. Installing `@your-scope/vault-client` from npm is the first proof it works for someone who is not you — which is the entire point of publishing.

## The send path

With the wrapper, sending is one line:

```ts
await client.sendTransaction([vaultClient.deposit({ owner, amountLamports })]);
```

Under that line, the client assembles a transaction message, applies your signer, and sends. You will do the assembly by hand once — in the exercise — so the collapsed version is not a mystery. The manual equivalent is a `pipe(...)` that sets the fee payer, sets a lifetime (a recent blockhash), and appends the instruction.

## The four subgoals

You write `assembleDeposit(owner, amountLamports, blockhash)`, building the transaction message in four numbered steps in the starter:

1. **Set the fee payer** to the owner. (done for you)
2. **Set the transaction lifetime** to the given blockhash — a transaction with no recent blockhash is rejected.
3. **Append the deposit instruction** built from your wrapper.
4. **Return the message**, and leave `computeUnitLimitSet` as `false`.

Subgoal 4 is not an oversight. This deposit goes out with **no** `SetComputeUnitLimit`, at the 200,000-CU default — so the next lesson has a real, measurable overpay to fix and you can diff the two fees. Do not add a compute budget here.

## Deploy

Read the deploy note after the exercise: connect the repo to Vercel in the browser, set the devnet RPC as an environment variable, and record your public URL.
