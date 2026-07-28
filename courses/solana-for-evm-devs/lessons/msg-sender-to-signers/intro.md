# `msg.sender` becomes a list the runtime already verified

`msg.sender` is a single address, and its meaning shifts with context: at the top
level it is an externally-owned account, and one `call` deeper it is the calling
contract. Half of Solidity's access-control folklore exists to manage that shift.

Solana replaces it with something flatter. A transaction carries a list of
accounts, and each one is flagged `is_signer` or not. Before your program runs,
the runtime has already verified every signature against every account marked as
a signer. By the time your code executes, `is_signer` is a fact, not a claim.

So the question is never "who called me?" It is "which of the accounts I was
handed are signers, and is the right one among them?"

```rust
require!(ctx.accounts.owner.is_signer, VaultError::NotOwner);
require_keys_eq!(ctx.accounts.vault.owner, ctx.accounts.owner.key());
```

Two separate checks, and both are load-bearing:

- **Did they sign?** Proves the transaction was authorized by that keypair.
- **Are they the right one?** Proves the signer is the account your state says
  is allowed.

A signature from *somebody* is worthless on its own. That is the Solana version
of the bug where a Solidity function checks that a caller exists but not that it
is the owner.

## Multiple signers are ordinary

A Solana transaction can carry several signers, all verified up front — a user
and a fee payer, or two parties to an escrow, in a single atomic transaction. In
Solidity, the equivalent means signature-verification plumbing you write yourself
(`ecrecover`, EIP-712, replay nonces). Here it is a property of the transaction
envelope, and there is nothing to implement.

## Where `tx.origin` went

It does not exist, and its absence is not a gap. `tx.origin` is dangerous in
Solidity precisely because it collapses the call chain into one address that
phishing can exploit. On Solana, when your program calls another program via CPI,
the runtime tracks which signatures propagate: the callee sees the original
signers, plus any PDA the caller signed for with its seeds. The chain stays
explicit instead of being flattened into a single ambient value.

## The habit to build now

Coming from Solidity, the reflex is to ask what the current call context is.
Retrain it into two questions you can answer by reading the account list:

1. Which accounts are signers?
2. Does my state say those specific keys are authorized?

Everything else in access control on Solana is a variation on those two lines.
