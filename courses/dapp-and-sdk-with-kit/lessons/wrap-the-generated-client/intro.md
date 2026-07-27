# Wrap It: the API You'd Actually Want to Import

The generated client works, but nobody wants to import it raw. To make a deposit with the bare generated code a consumer has to know the vault's seeds, derive the PDA themselves, assemble the account list in the right order, and remember that the amount is a `bigint`. That is exactly the knowledge your package exists to hide.

So you write a wrapper: a small, hand-authored surface — `deposit`, `withdraw`, `getVault` — that takes the two or three things a caller actually has (an owner, an amount) and returns something they can send. The wrapper calls the generated code; it never reimplements it.

## The rule to hammer

**Generated code is regenerated output. Never edit it — always wrap it.** The moment your IDL changes and you run codegen again, every hand-edit inside `src/generated` is gone. The wrapper is the part that survives, the part you version, and the part a bounty reviewer reads. This is exactly where this course diverges from a pure codegen tutorial: the codegen is three commands; the wrapper is the product.

## What the generated surface gives you

From your vault IDL, Codama generates (names are stable in Anchor 1.x IDLs):

- `getDepositInstruction`, `getWithdrawInstruction` — instruction builders
- `fetchVault(rpc, address)`, `fetchMaybeVault(...)` — account readers
- `getVaultDecoder()` — the raw decoder
- `findVaultPda(...)` — present because a 1.x IDL declares the seeds

Their types are Kit types: an `Address` (a branded string), not a `PublicKey`; lamports as `bigint`, not `number`.

## The four subgoals

In the exercise you write `deposit(owner, amountLamports)`. The generated surface — `deriveVaultPda` and `getDepositInstruction` — is already in the file and must not be edited. Your wrapper does four things, marked as numbered subgoals in the starter:

1. **Derive the PDA** from the owner, using the generated helper. (pre-filled)
2. **Narrow the input** — a deposit amount must be a positive `bigint`, so reject anything else before building. (pre-filled)
3. **Build from the generated builder** — call `getDepositInstruction` with the owner, the derived PDA and the amount.
4. **Return the ergonomic shape** — `{ ok: true, vaultPda, instruction }`, so the caller gets back exactly what they need and never touches a discriminator or a seed.

Subgoals 1 and 2 are written for you. Complete 3 and 4.
