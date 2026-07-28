# Storage costs rent, and the size is yours to choose

On the EVM you pay for storage once, at write time, in gas. `SSTORE` is
expensive precisely because the network stores that word forever. There is no
ongoing cost and no size declaration — you write to slot 7 and slot 7 exists.

Solana charges differently. An account occupies validator memory for as long as
it exists, so it must hold a minimum lamport balance proportional to its size.
That balance is called **rent**, and an account holding at least two years'
worth is **rent-exempt** — the only state anyone actually creates in practice.

Two things follow, and both are alien coming from Solidity.

## You declare the size up front

Creating an account means saying how many bytes it needs. Not "roughly"; exactly.
The runtime allocates that many bytes, zeroed, and charges you the rent-exempt
minimum for that size.

There is no `mapping` that grows as you insert. There is no dynamic array that
just gets longer. If you want to store more later, you either allocate for the
worst case now or `realloc` the account and top up the rent difference.

Sizing is therefore a design decision you make before writing a line of logic.
Get it too small and you cannot store what you need; too large and every user
overpays to create their account.

## Rent is refundable

Rent is a deposit, not a fee. Close the account and the lamports go back to
whoever you nominate. This is why Solana programs have a `close` instruction and
Solidity contracts do not bother with `selfdestruct` for state — reclaiming
space is a normal, profitable operation rather than a curiosity.

## Counting bytes

An Anchor account starts with an 8-byte **discriminator** — a hash of the account
type name, used to tell a `Vault` from a `Config` when both are just bytes owned
by your program. After that you pay for exactly what you declare:

```text
8                discriminator (Anchor)
32               Pubkey
8                u64  (lamports, timestamps)
1                u8   (a PDA bump)
1                bool
4 + len          String or Vec (4-byte length prefix, then contents)
```

A vault with an owner `Pubkey`, a `u64` balance and a `u8` bump is
`8 + 32 + 8 + 1 = 49` bytes. That number is not trivia — it is an argument you
pass at creation, and getting it wrong is one of the most common first bugs for
developers arriving from the EVM.

In the exercise you will compute exactly that number.
