# PDAs: An Address With No Key

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · `borsh` resolves to **1.8.0** · edition 2021. Every compiler message quoted below was produced by compiling this lesson's starter on that toolchain.

You know how to size the vault. You do not yet know **where it lives**, and the answer is stranger than
it looks: at an address that nobody can sign for.

## The problem a PDA solves

Right now your `initialize_vault` creates an account at whatever address the client hands it. That has
two failures, and the compiler can see neither of them.

The first is that nothing ties the account to the user. Any address will do. There is no rule that
Alice gets one vault, or that Alice's vault is Alice's.

The second is worse. A normal account's address **is** a public key, and somewhere there is a private
key that matches it. Whoever holds that private key can sign for the account. An account your program
is supposed to control, sitting at an address someone can sign for, is not controlled by your program.

A **Program Derived Address** fixes both at once. It is an address computed from inputs you choose, and
it is deliberately an address for which no private key exists.

## Why no private key exists

Solana signatures are Ed25519, and every Ed25519 public key is a point on a particular elliptic curve.
Not every 32-byte string is such a point — most are not. A byte string that is not on the curve cannot
be a public key, so no private key can correspond to it, so nothing can ever produce a signature for
it.

PDA derivation exploits exactly that. It searches for a 32-byte result that is **off** the curve:

```
for bump in (0..=255).rev() {
    candidate = sha256(seeds || [bump] || program_id || "ProgramDerivedAddress")
    if !is_on_curve(candidate) {
        return (candidate, bump)
    }
}
```

Three things to take from that loop.

**The search runs downward from 255.** The first bump that yields an off-curve result is the
**canonical bump** for those seeds. Roughly half of all candidates are off-curve, so it is usually 255
and almost always found within the first few tries — but "usually" is not "always", which is why the
bump is an output of the derivation and not a constant you can assume.

**The program id is part of the hash.** The same seeds under a different program id give a different
address. A PDA belongs to one program by construction, and no other program can derive into your
address space.

**The literal `"ProgramDerivedAddress"` marker is in there too.** It is what keeps PDA derivation from
colliding with the other ways addresses get derived on Solana.

The output is an address your program can *authorize* — not by signing, but by presenting the seeds
during a cross-program invocation and letting the runtime re-derive and check them. That is module 3's
lesson. What matters here is the consequence: **the seeds are the credential**. Knowing them is what
lets your program act for the address, and no key theft can substitute for them.

## In Anchor, you do not run the loop

`Pubkey::find_program_address` and `Pubkey::try_find_program_address` are the primitives, and you will
see them in client code and in native programs. Inside an Anchor accounts struct you almost never call
them, because two constraints do it declaratively:

```rust
#[account(
    init,
    payer = user,
    space = 8 + 32 + 8 + 1,
    seeds = [b"vault", user.key().as_ref()],
    bump
)]
pub vault: Account<'info, VaultState>,
```

`seeds` says what the address is derived from. Bare `bump` says "derive the canonical bump and check
that the account you were handed is at that exact address." If the client passes anything else, the
instruction fails before your handler body runs — you do not write that check, and you cannot forget
to.

The two constraints are a pair, and Anchor refuses either one alone — with a different
message for each direction, which is a free hint about which half you forgot.
`seeds` without `bump` reports `bump must be provided with seeds`. `bump` without
`seeds` reports `seeds must be provided before bump`, because there is nothing to bump.

Anchor also hands you the bump it just computed, on `ctx.bumps`, named after the field:

```rust
vault.bump = ctx.bumps.vault;
```

`ctx.bumps.vault` exists **only** if that account carries a `bump` constraint. Reference it without
one and the compiler says `no field 'vault' on type 'InitializeVaultBumps'`. That is a useful property:
it makes one of this lesson's subgoals something the build can actually check.

## Store the bump. Always store the bump.

You already have `bump: u8` in `VaultState`. This is the lesson where the reason lands.

Every later instruction — `deposit`, `withdraw`, anything that touches the vault — has to prove the
account it was given is the right PDA. It can do that two ways:

```rust
// re-derives: runs the search loop again, on-chain, every call
#[account(mut, seeds = [b"vault", user.key().as_ref()], bump)]

// reads the byte you stored, then verifies that one candidate
#[account(mut, seeds = [b"vault", user.key().as_ref()], bump = vault.bump)]
```

The first form pays for the search in compute units. Each candidate is a SHA-256 over the seeds plus
the program id plus the marker, followed by a curve check, and the loop stops at the first off-curve
result — typically after one round, occasionally after several. The second form does one hash with a
known bump and compares. It is strictly cheaper, and the difference is unbounded in the bad case
whereas the stored form is constant.

There is a correctness argument too, and it is the more important one. `bump = vault.bump` pins the
account to the bump that was canonical **when the vault was created**, which is the only bump that
address should ever be reachable by. Accepting a bump the caller supplies, without checking it against
a stored value, is how programs end up with several valid addresses for what was meant to be one
account. The stored byte is a one-byte commitment that removes the question.

One byte of `space`, about 0.000014 SOL of deposit, for a cheaper and tighter check on every future
instruction. That is the trade, and it is not close.

## Designing seeds

Seeds are an ordered list of byte slices. Up to 16 of them, each at most 32 bytes, and the bump counts
as one — so 15 is your practical ceiling, which is far more than you will want.

**One per user** — what the vault uses:

```rust
seeds = [b"vault", user.key().as_ref()]
```

The `b"vault"` prefix is a namespace. Without it, a second per-user account in the same program would
derive to the same address as the first.

**One per user per resource** — the shape for orders, escrows, positions:

```rust
seeds = [b"order", user.key().as_ref(), &order_id.to_le_bytes()]
```

Note `to_le_bytes()`. Numbers go into seeds as their fixed-width little-endian bytes, never as decimal
text, because a fixed width is what keeps them from running into the next seed.

**One per program** — config and authority singletons:

```rust
seeds = [b"config"]
```

The hazard to actually avoid is **ambiguity between adjacent variable-length seeds**. If you derive
from two user-supplied strings back to back, `["alice", "bob"]` and `["alic", "ebob"]` are different
seed lists — but a design that lets a caller choose where the boundary falls is a design where two
different logical keys can land on one address. Fixed-width seeds (a `Pubkey` is always 32 bytes, a
`u64` is always 8) do not have this problem, which is why almost every PDA in production is a constant
prefix plus fixed-width parts.

## What the build can and cannot see

The compiler will catch a `bump` with no `seeds`, and it will catch `ctx.bumps.vault` when no `bump`
constraint exists. It will not catch seeds that are simply *wrong* — `b"vaults"`, or the user's key
where the mint's belongs. Those derive a perfectly valid PDA at an address your client never looks at,
and the failure shows up as a puzzling `ConstraintSeeds` error at runtime, or as an account that
initializes fine and is then unreachable.

The check for that is the derivation you already wrote on the client side in Course 1: derive the
address off-chain, derive it on-chain, and require that they match. Module 4 shows you exactly that assertion written as a
LiteSVM test — read it before you spend a lamport on devnet.

---

## Your task

Build the starter before you change anything. It reports **one** error, and the error is not in your
program:

```
error[E0609]: no field `vault` on type `&InitializeVaultBumps`
```

That comes from the verification harness at the bottom of the file. Your `initialize_vault` compiles
perfectly well as it stands — it creates a 49-byte account at whatever address the client passes, with
no seeds, no bump, and no relationship to the user whatsoever. That is a legal Anchor program and a
broken vault, and nothing except the harness can tell the difference. Which is the whole reason the
harness is there.

Three labeled subgoals turn it into a PDA. Do them in order; each one is what makes the next one
possible.
