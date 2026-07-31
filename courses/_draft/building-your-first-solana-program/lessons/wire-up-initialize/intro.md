# Wire Up Initialize

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · `borsh` resolves to **1.8.0** · edition 2021. Every compiler message quoted below — including the error count and the order they arrive in — was produced by compiling this lesson's deliberately-broken starter on that toolchain.

This lesson is different. The starter **does not compile**, on purpose, and the first thing you do is
press Build and read what comes back.

That is not a warm-up. Reading Anchor's build output is a distinct skill from writing Anchor, most
people never practise it deliberately, and it is the difference between a five-second fix and an
afternoon.

## Task 1 — diagnose the bug

1. Click **Build** before you change anything.
2. Read the output **from the top**.
3. Fix one thing.
4. Build again.

The `InitializeVault` struct uses `#[account(init, ...)]` to create the vault PDA, and it is missing a
required account. There is a comment where it should be.

### What the output will look like

You will get ten errors. Nine of them are noise, and knowing that is most of the skill.

The **first** line is Anchor's own, and in Anchor 1.x it is unusually direct:

```
error: a non-optional init constraint requires a non-optional system_program field
to exist in the account validation struct. Use the Program type to add the
system_program field to your validation struct.
```

Everything after it is fallout. `#[derive(Accounts)]` gave up, so the items it was going to generate
never appeared, and rustc reports each absence separately:

```
error[E0432]: unresolved import `crate`
error[E0277]: the trait bound `InitializeVault<'_>: Bumps` is not satisfied
error[E0599]: no function or associated item named `try_accounts` found for struct `InitializeVault`
error[E0277]: the trait bound `InitializeVault<'_>: anchor_lang::Accounts<'_, _>` is not satisfied
```

None of those is a real problem. There is one problem, stated once, at the top.

Older Anchor versions did not print that first line at all — you got the trait-bound cascade and
nothing else, which is why "the trait bound `Accounts` is not satisfied" is still the most-searched
Anchor error on the internet and why so many answers to it are guesswork. Anchor 1.x tells you the
answer plainly and then buries it under nine more lines. **Read from the top, fix one thing, build
again.**

One of those nine is the verification harness at the bottom of the file also noticing the missing
field. It lands last, long after Anchor has already told you.

### Why `init` needs the System Program at all

Your program cannot create accounts. Nothing can, except the System Program — allocation and ownership
assignment are its instructions, and every account on Solana was created by a call to it.

So `init` is not a local operation. It expands into a **cross-program invocation**: your program calls
the System Program's `create_account`, which allocates `space` bytes, assigns the new account's owner to
your program id, and moves the rent-exempt deposit from `payer`. A CPI needs the callee in the
transaction's account list, so `system_program` has to be a field on the struct. Anchor will not add it
for you, and it is telling you so.

Module 3 makes CPIs explicit, with transfers you write yourself. This one you have been making since
lesson 3 without seeing it.

### One thing not to do

There is a way to make the build pass that is worse than the bug. Deleting `init` — or replacing it
with `mut` — removes the constraint that needed the System Program, and the file compiles. It also no
longer creates anything.

The verification harness at the bottom of the file names `system_program` as a required field, so that
particular escape is closed: drop `init` and stop there, and the harness fails you. Be precise about
how far that goes, though — drop `init` *and* add `system_program` anyway, and the build is green on a
program that creates nothing. `init` is not something a compiler can check for. Take it as the general
warning: on this platform a green build means *the file compiled*, and "make the error go away" and
"make the program correct" are different goals that occasionally point in opposite directions.

## Task 2 — write the body

Once it builds, the handler is yours to write. No scaffolding this time.

By the time `initialize_vault` runs, the account exists: 49 bytes allocated, owner set to your program,
discriminator written, every field zero. What it does not have is meaning. Three assignments give it
some:

1. **`owner`** — the wallet this vault belongs to. Module 3's `has_one = owner` check reads exactly this
   field to reject a withdrawal signed by somebody else. Get it from the accounts struct.
2. **`balance`** — zero. The bytes are already zero, so the line changes nothing; write it anyway,
   because the next person to read the handler should not have to know that in order to trust it.
3. **`bump`** — the canonical bump Anchor derived a moment ago, so `deposit` and `withdraw` can use
   `bump = vault.bump` instead of re-running the search on every call. Anchor hands it to you on
   `ctx.bumps`, under this account's own field name.

### What the grader can and cannot see

Compile-only grading means the harness can prove your struct holds the right three accounts, and can
prove `ctx.bumps.vault` is reachable. It cannot see the body. A handler that assigns nothing at all
compiles exactly as well as a correct one.

So this rung is graded on the bug and trusted on the body — and the untrusted half is exactly what
module 4's LiteSVM lesson shows you how to check, before you spend a lamport of devnet SOL on it.
Reading that test and knowing why compiling could not replace it is the point.
