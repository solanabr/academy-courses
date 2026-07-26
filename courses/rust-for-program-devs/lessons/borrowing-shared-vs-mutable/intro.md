# Borrowing: `&T` and `&mut T`

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (crates.io latest stable) · `borsh` 1.5.7 declared / **1.8.0** resolved · edition 2021 · Agave ≥ 3.1.10 / rustc per the build server's pinned platform-tools.

Moving a value into a function and getting it back out is unbearable. Real code needs to look at a vault without consuming it, and change a vault without rebuilding it. That is borrowing, and there are exactly two kinds.

| | Written | How many at once | What it grants |
| --- | --- | --- | --- |
| **Shared borrow** | `&VaultLike` | Any number | Read every field |
| **Mutable borrow** | `&mut VaultLike` | Exactly one | Read and write every field |

And the rule that connects them: **while a mutable borrow is alive, no other borrow of that value may exist — shared or mutable.** Many readers, or one writer. Never both.

This is the same invariant as the runtime's account locks from Course 1. A transaction declares each account as readonly or writable, and the scheduler will not run two transactions that write the same account concurrently. Rust applies that rule to a value inside your function, at compile time, for free. The reason you are learning it here rather than in Course 3 is that `&mut Account<'info, Vault>` is unreadable until this table is boring.

## Two signatures, and they are the ones you keep

```rust
pub fn balance_of(v: &VaultLike) -> u64;
pub fn credit(v: &mut VaultLike, amount: u64);
```

Read them out loud as promises to the caller. The first says: *I will look and I will not touch, and you keep your vault.* The second says: *for the duration of this call the vault is mine alone, and then you get it back changed.*

In module 4 these become methods and the receiver moves to the front, which is the only difference:

```rust
impl VaultState {
    pub fn balance(&self) -> u64;
    pub fn deposit(&mut self, amount: u64) -> Result<()>;
}
```

`&self` is `&VaultState`. `&mut self` is `&mut VaultState`. There is no third thing to learn later.

## `mut` in two places, meaning two things

```rust
let mut vault = sample_vault();   // (1) this BINDING may be reassigned or mutated
credit(&mut vault, 1_000);        // (2) hand out an exclusive borrow
```

`let mut` is about the variable you own. `&mut` is about the loan you give away. A caller needs (1) before it can produce (2) — you cannot lend out write access to something you were not allowed to write yourself.

## When does a borrow end?

Here is where most "why is this still borrowed?" confusion comes from. Since the 2018 edition, a borrow ends **at its last use**, not at the closing brace of the block. This is called non-lexical lifetimes, and it is why this compiles:

```rust
let mut vault = sample_vault();
let before = balance_of(&vault);   // shared borrow starts and ends on this line
credit(&mut vault, 1_000);         // fine — nothing is borrowed any more
```

and why the near-identical version below does **not**:

```rust
let mut vault = sample_vault();
let before = &vault.balance;       // shared borrow starts …
credit(&mut vault, 1_000);         // … mutable borrow while it is alive
msg!("{}", before);                // … and it is used AFTER, so it is still alive here
```

Delete that last line and the second version compiles too. The borrow was never a problem in itself — holding it *across* the mutation was. A great deal of Rust advice written before 2018 tells you borrows run to end of scope. It is out of date, and following it makes you write scopes and `drop()` calls you do not need.

The fix in the first version is worth naming, because it is the payoff from the previous lesson: `balance_of` returns a `u64` **by value**. `u64` is `Copy`, so you hold a number, not a loan, and there is nothing left to conflict with.

## One honest gap

`credit` has no way to report failure. Its return type is `()`.

So when `add_lamports` hands back `None` — the addition would have wrapped a `u64` — the only thing this version can do is leave the balance untouched and say nothing. **That is not acceptable in a program.** A deposit that silently does nothing is a support ticket at best and an exploit at worst.

It is a deliberate placeholder, and closing it is what module 3 is for: `credit` grows a `-> Result<()>`, the `None` arm becomes `err!(VaultError::Overflow)`, and the caller is forced by the compiler to deal with it. Write the placeholder now, know that it is one, and do not carry the pattern into anything real.

## Three subgoals

The exercise ships three numbered subgoals as comment headers. The first is done for you as a worked reference; you write the second and third. Read all three headers before you start — the third one is the borrow-checker lesson, and knowing it is coming changes how you write the second.

The file does not build as delivered: each unwritten body carries a `compile_error!` naming its subgoal, so your first build is a to-do list rather than a green tick. Delete each one along with the `todo!()` beneath it.

A note on what the grader can see. The build server checks that your file **compiles**; it does not run your functions. The `mod verify` block at the bottom is not editable and pins all three functions to their exact signatures, borrows included — rename one, or take `VaultLike` by value instead of by reference, and the build stops. That is a check on names and types only. Nothing verifies that `credit` adds rather than subtracts.

Nothing verifies subgoal 3's ordering either, and it is worth being exact about why, because the temptation is to assume the borrow checker has your back here. It does not. `E0502` fires for **one specific shape**: holding `&v.balance` across the call to `credit` and then using it afterwards. The spec tells you not to write that — bind the `u64` by value instead — and once you do, there is no borrow left for the checker to have an opinion about. Write `credit(v, …)` first and read the balance second and the file compiles clean.

So the compile-time guarantee here is narrow and real: *if* you hold a reference across a mutation, you are stopped. The wider thing — that the read happens before the write — is checked by you. Which is the honest version of every exercise in this course, and the reason lesson 9 spends a whole lesson on diagnosing code you have not built.
