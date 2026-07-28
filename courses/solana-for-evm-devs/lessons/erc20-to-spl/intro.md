# One token program, and your balance is an account

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
practice its address is an **associated token account**: a PDA derived from
`(owner, mint)`, exactly the derivation from lesson 3. Owner and mint are part of
the address.

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
surprise for EVM developers, and the reason client code so often uses
`getOrCreateAssociatedTokenAccount`.

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
