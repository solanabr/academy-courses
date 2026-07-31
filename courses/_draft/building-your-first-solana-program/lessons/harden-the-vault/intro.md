# Harden the Vault

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` and `anchor-lang-error 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · edition 2021. Every error code and message below was read out of `anchor-lang-error 1.1.2`'s enum; every compiler and macro error was produced by compiling this lesson's file. Every line of example code here is ours.

Your vault works. Now assume the caller is hostile.

Every account your instruction touches arrived in the transaction because somebody put it there, and that somebody is not always you. `Deposit` and `Withdraw` are safe not because you wrote careful handler bodies but because of four or five words inside `#[account(...)]`. This lesson is about what those words do, which attack each one stops, and the one thing Anchor 1.0 changed underneath them.

## Constraints run before your code does

`#[derive(Accounts)]` is not documentation. It expands into a function that runs before your handler body and returns `Err` if anything fails to check out. By the time your first statement executes, every constraint has passed.

That is why validation belongs in the attribute rather than the body: an attribute cannot be forgotten halfway down a function, and it applies on every code path including the ones you did not think about.

## The constraints you have used, and what each one refuses

**`init, payer = user, space = N`** — allocates the account, moves the rent-exempt deposit from the payer, assigns the account to your program, and writes the 8-byte discriminator. Used exactly once per account, in `initialize_vault`.

**`mut`** — required of any account whose data or lamports change. Anchor checks it at runtime and returns error **2000, "A mut constraint was violated"** if the account was not marked writable in the transaction. Note that this is a runtime failure, not a compile error: a struct missing `mut` compiles perfectly and fails on the first real call.

**`Signer<'info>`** — the account signed the transaction. It says nothing about mutability, which is why a payer needs both `Signer` and `#[account(mut)]`.

**`seeds = [...]` + `bump`** — re-derives the address and compares it to the account passed in. Mismatch is error **2006, "A seeds constraint was violated"**. This is the single most load-bearing constraint in your program, and the next section is about why.

**`Account<'info, T>`** — not a constraint but a type, and it validates three things on deserialisation: the account is owned by your program (**3007**), the first eight bytes match `T`'s discriminator (**3002, "Account discriminator did not match what was expected"**), and the account has actually been initialised (**3012**). Compare with `UncheckedAccount<'info>`, which validates *nothing*. Convention — and the Anchor CLI — asks you to write a `/// CHECK:` comment above one, documenting why skipping validation is safe here. Be precise about who enforces that: it is a **lint in the Anchor CLI's IDL build**, not a `cargo-build-sbf` error. Our build server shells `cargo-build-sbf` and nothing else, so on this platform the missing comment builds green. Write it anyway — the reviewer who reads your program is the one enforcing it, and on a real Anchor project `anchor build` will stop you.

**`has_one = field`** — checks `account.field == field_account.key()`, where `field` must be both a field on the stored struct and an account in the same struct. Failure is error **2001**. Precision worth having: `has_one = owner` only works if there is an account *named* `owner` in the accounts struct. Ours is named `user`, so `Withdraw` uses the explicit form instead:

```rust
constraint = vault.owner == user.key() @ VaultError::NotOwner
```

Same check, one extra line, and it returns your own error instead of Anchor's generic one — which is what finally spends the `owner` field and the `NotOwner` variant you have been carrying since Course 2.

## Three attacks, and the words that stop them

### Attack 1 — someone else's vault

The caller signs as themselves and passes *your* vault as the `vault` account. Nothing in the handler body would notice: it is a well-formed `VaultState` owned by the right program.

`seeds = [b"vault", user.key().as_ref()]` + `bump = vault.bump` stops it. The address is recomputed from the signer's key, so the only account that can satisfy the constraint is that signer's own vault. `constraint = vault.owner == user.key()` is a second, independent check on the same question — belt and braces, because the two would only disagree if something else had already gone badly wrong, and finding out cheaply is worth one comparison.

### Attack 2 — an account that is not a vault

The caller passes an account of a different type, or one that has never been initialised, hoping your program interprets whatever bytes are there as `owner`, `balance` and `bump`.

`Account<'info, VaultState>` stops it, in the type system rather than in your logic. The 8-byte discriminator that `#[account]` writes at creation is checked on every deserialisation, so an account of the wrong type fails with 3002 and an empty one with 3012. Reach for `UncheckedAccount` instead and you have opted out of all three checks — which is occasionally right, and is why the `/// CHECK:` comment exists to make you say so out loud.

### Attack 3 — the same account passed twice

This is the interesting one, and the one Anchor 1.0 changed.

Suppose an instruction moves balance from one vault to another and takes both as mutable accounts. Now suppose the caller passes the *same* account as both. Your handler does:

```rust
ctx.accounts.from_vault.withdraw(amount)?;
ctx.accounts.to_vault.deposit(amount)?;
```

Anchor deserialised the account twice, into two independent copies. The first line subtracts from copy A. The second line adds to copy B — which never saw the subtraction. At the end of the instruction both copies are serialised back to the same account, and whichever writes last wins. The caller has minted `amount` out of nothing, and every individual line of your code was correct.

**In Anchor 1.0 this is refused by default.** Two mutable accounts resolving to the same key produce error **2040, "A duplicate mutable account constraint was violated"** before your handler runs. If you genuinely want aliasing — and there are instructions where it is harmless — you opt in per account:

```rust
#[account(mut, dup)]
```

Two cautions.

The syntax is `dup`. You will find `#[account(mut, unsafe(dup))]` written down; that is the v2 alpha (the `no_std` Pinocchio line), and on the toolchain that grades you it fails at macro-parse time with `error: expected =`. Verified.

And the default is protection, not a suggestion. Adding `dup` because an error appeared is how you reintroduce the bug the error was reporting. The right question is never "how do I silence 2040" but "is aliasing safe in this specific instruction" — and for anything that moves value between two accounts, the answer is no.

## The wider list

Aliasing, missing signer checks, missing owner checks, type confusion, arithmetic overflow, unchecked account closing, PDA seed collisions: these are *classes*, and the classes are stable even as the framework changes. Helius maintains a good public write-up of nineteen of them, and it makes a fine review checklist to read against your own program. Read it as a list of questions to ask, not as code to copy — the widely-circulated example repositories for these attacks ship with no licence at all, so every line of example code in this course is written by us.

## The exercise

Independent write. You get the signature of a new instruction and a written spec — no more. The three instructions you already wrote are still in the file above it, and `Withdraw`'s constraint block in particular is close to what `from_vault` needs; reading your own earlier work and adapting it is the intended move, not cheating. What no one gives you is which constraints this instruction needs, in what combination, and what `to_vault` must derive from.

Be clear-eyed about what the grading can see. The verification harness pins the field names and types, so **attack (b) is genuinely compile-enforced**: get `Account<'info, VaultState>` wrong and the build fails. The seeds, the ownership constraint and the `dup` decision are runtime behaviour, and a compile-only grader is blind to all three. Module 4 shows you the LiteSVM test that would exercise them, and why compiling cannot — read it before you spend a lamport of devnet SOL.
