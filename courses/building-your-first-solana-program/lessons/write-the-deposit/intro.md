# Write the Deposit

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · edition 2021. Every compiler message quoted below was produced by compiling this lesson's file.

You read a working `deposit` in the last lesson. Now write one.

The starter has the vault program with `deposit` and its accounts struct removed, and three numbered subgoals in their place. The previous lesson is still open in another tab; that is deliberate. At this rung the work is not recall, it is knowing which piece goes where and why each line is there — and subgoal 3 asks for something no tutorial can hand you: a call into a file *you* wrote.

## Subgoal 1 — `Deposit<'info>`

Three accounts. `system_program` is pre-filled, because you already know why it stays even though `CpiContext::new` no longer takes it. The other two you write:

- **the vault** — mutable, and re-derived from its seeds so Anchor can prove the account passed in really is *this* caller's vault.
- **the user** — a signer, and mutable.

`user` being `mut` is the one that catches people. In Module 2's `initialize_vault` the user was `mut` because they paid rent. Here nobody pays rent — the account already exists — but lamports still leave the user's wallet, and any account whose lamports change must be declared mutable. Same constraint, different reason.

### `bump = vault.bump`, not bare `bump`

Both are legal. They do different amounts of work:

| Written | What Anchor does |
| --- | --- |
| `bump` | searches from 255 downwards for the canonical bump, hashing at each step |
| `bump = vault.bump` | one hash, using the byte you stored at initialize time |

The stored bump is *why* you spent a byte on it in Module 2. Use it. A bare `bump` here would be paying compute units to recompute an answer the account is already carrying.

## Subgoal 2 — the CPI

Ask the System Program to move `amount` lamports from the user to the vault. The shape from the last lesson, with the Anchor 1.0 first argument:

```
transfer( CpiContext::new( <program id>, Transfer { from, to } ), amount )?
```

`Transfer`'s two fields are `AccountInfo`s, so both accounts need `.to_account_info()`. Get the direction right: the user is the source.

## Subgoal 3 — call your own code

```rust
ctx.accounts.vault.deposit(amount)?;
```

One line. That line is the whole reason Course 2 exists.

Everything it does — reject a zero amount, add with `checked_add`, return `VaultError::Overflow` instead of wrapping — is already written, already reasoned about, and lives in a module with no accounts in it. You are not going to retype `checked_add` here, and if you find yourself writing `ctx.accounts.vault.balance += amount` then stop: that is a fresh, unchecked, untested copy of a function you already own, and the version you already own is better.

This is what "import, don't retype" means as a mechanical fact rather than a slogan. Look at your finished file afterwards and note that the arithmetic appears exactly once in it.

## Does the order matter?

You have two statements — the CPI and the bookkeeping — and you may be wondering which goes first.

It does not matter, and the reason is worth knowing: **a transaction either completes or is entirely rolled back.** If `vault.deposit(amount)` returns `Err` after the CPI has already moved lamports, the transfer is undone along with everything else. There is no state where the lamports moved and the balance did not.

That guarantee is what lets you write the happy path in the order that reads best.

## What the build server can and cannot tell you

At the bottom of the starter is a `mod verify` block marked do-not-edit. It compiles only if `deposit` exists with exactly the right signature and `Deposit` has all three fields with the right types. Get a name or a type wrong and you get a compile error naming it.

It is a **type** check, not a behaviour check. It cannot see whether your CPI moves lamports in the right direction, and it cannot see whether you called `vault.deposit` at all — an empty handler body with a correct signature compiles. The harness comment lists exactly which mistakes it is blind to. Read that list; it is more useful than the part it does catch.
