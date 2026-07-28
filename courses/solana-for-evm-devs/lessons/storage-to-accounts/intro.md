# Your program owns no storage

In Solidity, a contract and its state are the same deployed thing. `uint256
public totalDeposits;` lives at a slot inside the contract's own storage trie,
and only that contract's code can write it. Deploy the contract, and the storage
comes with it — empty, but there.

On Solana a program is an account holding executable bytes, and that account is
**immutable at runtime**. Your program cannot write to itself. There is no slot
0. `totalDeposits` has to live in a *different* account, one that you create,
size, and pay for.

## The ownership rule that replaces `private`

Solidity gives you visibility keywords. Solana gives you one runtime rule, and
it is stricter than anything in the EVM:

> A program may only write to accounts it owns.

Every account carries an `owner` field — the public key of the program allowed
to modify its data. When your program writes to an account it does not own, the
runtime rejects the transaction. Not a revert your code chose; a refusal before
your logic gets a say.

This is why you will see Solana developers say "the program owns the account"
where you would say "the contract holds the state". The account exists
independently. Your program is merely the only thing permitted to change it.

## Three consequences you will feel immediately

**State is passed in, not looked up.** Solidity reads `totalDeposits` from
ambient storage. A Solana instruction receives the accounts it may touch as
explicit arguments. If the caller does not pass the account, your program cannot
read it — there is no ambient anything.

**One program, many state accounts.** An ERC-20 is one contract per token, with
balances in an internal mapping. SPL Token is *one* program, deployed once, and
every mint and every balance is a separate account it owns. You will meet this
inversion again in module 3; it is the same rule you are reading now.

**Upgrades do not migrate state.** Upgrading a Solidity contract means proxies,
because state and code are welded together. On Solana you replace the
executable bytes of the program account, and the state accounts are untouched —
they were never inside the program to begin with. The flip side: if your new
code reads the old layout differently, nothing stops it. Nobody migrated
anything, because nobody moved anything.

## Reading an account

Every account, whoever owns it, has the same envelope:

```text
lamports    u64      balance in the base unit of SOL
owner       Pubkey   the program allowed to write `data`
executable  bool     is this account a program?
data        bytes    opaque to the runtime; meaningful only to `owner`
```

`data` is bytes. The runtime does not know or care what is in there — no types,
no ABI, no storage layout. Interpreting those bytes is entirely your program's
job, which is why Solana programs spend real effort on serialization in a way
Solidity developers never have to think about.

Hold on to the envelope. `lamports` and `owner` come back in the next lesson, in
a way that has no EVM analogue at all: on Solana, keeping data alive costs money
continuously, and an account that stops paying can be erased.
