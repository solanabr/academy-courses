# One token program, and your balance is an account

> Version stamp — `@solana/kit` 7.0.0 · `@solana-program/token` 0.15.0 ·
> authored 2026-07-30.

Every ERC-20 is a separate contract. Deploying a token means deploying code, and
"balance of X" is a lookup inside that contract's own mapping. Two tokens are two
codebases, and each may behave differently — fee-on-transfer, rebasing, a
non-standard `approve` — which is why integrating an arbitrary ERC-20 is a small
audit.

Solana inverts this. There is **one** SPL Token program, deployed once, shared by
every token on the chain. Creating a token deploys nothing; it creates a **mint
account** that the token program owns.

```text
Mint account          supply, decimals, mint authority, freeze authority
Token account (ATA)   owner, mint, amount, delegate
```

A balance is not a row in a mapping. It is a **token account** — its own account,
owned by the SPL Token program, holding one balance of one mint for one owner. In
practice its address is an **associated token account**: exactly the derivation
from lesson 3, applied to the wallet and the mint.

Get the details right, because this is the one PDA everybody derives and it does
not derive the way you would guess. The seeds are **three**, in this order —
`[owner, token_program, mint]` — and the derivation runs under the **Associated
Token Account program**, not under the token program whose address sits in the
middle of the seed list:

```ts
import { findAssociatedTokenPda, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";

const [ata] = await findAssociatedTokenPda({
  owner,
  tokenProgram: TOKEN_PROGRAM_ADDRESS,
  mint,
});
```

The token program appears as a seed because the same wallet can hold the same
mint under classic SPL Token and under Token-2022, and those must be different
addresses.

## What this buys you

**Uniform behavior.** Transfer semantics are the same for every SPL token,
because it is the same program every time. The fee-on-transfer surprise cannot
exist in the base token program — a token cannot override transfer, because it
has no code of its own. (Token-2022 adds opt-in extensions such as transfer
hooks, and those are explicitly opt-in and visible on the mint.)

**No approval to interact.** Reading a balance is reading an account. No view
function, no RPC call into a contract, no ABI.

## What it costs you

**Token accounts must exist before they can receive.** There is no "send to an
address and it just works". If the recipient has no token account for that mint,
someone must create it — and pay its rent. That is the single most common
surprise for EVM developers.

The pattern that answers it is **create-if-missing, then transfer**: derive the
ATA, and prepend `getCreateAssociatedTokenIdempotentInstruction({...})` to the
same transaction as your transfer. *Idempotent* is the load-bearing word — it
succeeds whether or not the account already exists, so you never have to read
the chain first to decide, and two clients racing to fund the same recipient do
not knock each other over. One transaction, still atomic.

> **In the wild (checked 2026-07-30).** You will constantly read
> `getOrCreateAssociatedTokenAccount(connection, payer, mint, owner)` from
> `@solana/spl-token`, and its sibling `getAssociatedTokenAddress`. Both are the
> superseded v1-era stack, and the first quietly sends its own transaction
> before yours. Read them; write the derive-plus-idempotent-instruction pair.

**A wallet holds many token accounts.** One per mint. Your SOL balance is the
`lamports` field on your wallet account; every SPL token you hold is a separate
account keyed to that mint.

## Decimals, and the arithmetic you must not get wrong

Like ERC-20, amounts are integers and `decimals` is display metadata. Unlike
ERC-20's near-universal 18, Solana decimals vary widely — USDC uses 6, many NFTs
use 0, wrapped SOL uses 9.

Because the number is not conventional, hardcoding it is a live bug rather than a
mostly-safe assumption. Converting a human amount to base units is
`uiAmount × 10^decimals`, and that is what you will implement.
