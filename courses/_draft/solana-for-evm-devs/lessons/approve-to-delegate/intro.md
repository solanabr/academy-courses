# `approve` becomes a delegate on the account itself

The ERC-20 allowance pattern is two transactions and a well-known footgun. You
`approve(spender, amount)`, the spender later calls `transferFrom`, and the
allowance sits in a nested mapping inside the token contract. Everyone has seen
the infinite-approval problem, and everyone has seen a wallet drained through a
stale one.

SPL Token keeps the idea and moves where it lives. A token account has two extra
fields:

```text
delegate         Option<Pubkey>   who may move tokens from this account
delegated_amount u64              how much they may still move
```

The approval is not in a mapping keyed by `(owner, spender)`. It is **on the
token account being spent from**.

## What changes as a result

**One delegate at a time.** Approving a second spender overwrites the first. In
ERC-20 you can hold approvals for a dozen contracts simultaneously; here the
field is singular. That is a real constraint, and it pushes protocols toward PDAs
that own accounts outright rather than accumulating standing approvals.

**Revocation is a single instruction with no amount.** `revoke` clears the
delegate and zeroes the delegated amount. There is no "approve to 0" dance and no
race between reading and resetting an allowance — the ERC-20 front-running hazard
that produced `increaseAllowance` does not arise, because you are not comparing
against a previous value.

**Delegation is visible where the tokens are.** Reading a token account tells you
whether something can move its balance and how much. In ERC-20 you would have to
know which spender to ask about; here, one account read answers it.

## The pattern that mostly replaces it

Much of what ERC-20 uses `approve` for, Solana does with ownership instead. A
protocol derives a PDA, that PDA owns a token account, and the user transfers into
it. Now the protocol does not need standing permission over the user's wallet —
it controls an account it already owns, and the user's own account has no
outstanding delegate at all.

Whether that is *safer* depends on the protocol. It is certainly narrower: the
blast radius is whatever the user deposited, not whatever remains in their wallet
under an old infinite approval. When you port an EVM protocol, look for approvals
first, and ask whether ownership transfer models the intent better.

## Where they still coincide

Both models share the core hazard: an approval you forgot about is authority
someone still holds. The Solana version limits it — one delegate, a decrementing
amount, one-instruction revocation — but it does not eliminate it. Auditing your
delegates is still worth doing, and now it is a field you can read rather than a
mapping you have to guess at.
