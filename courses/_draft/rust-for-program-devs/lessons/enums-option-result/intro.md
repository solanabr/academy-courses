# Enums, `Option`, and `Result`

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (latest on crates.io, published 2026-06-26) · `borsh 1.8.0` · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · edition 2021. `ERROR_CODE_OFFSET = 6000` and the numbering rule below were read out of `anchor-lang-error 1.1.2` and `anchor-syn 1.1.2`; the compiler messages quoted were produced by compiling this lesson's file.

Lesson 4's `sub_lamports` returned `None` when the vault did not hold enough. That was true and it was useless. `None` cannot tell a client whether the amount was zero, or larger than the balance, or whether the signer had no business asking. It says only "no".

`Option` was the training-wheels version. This lesson is the real one.

## An enum is a closed set of alternatives

In TypeScript you would reach for a union of string literals, and the compiler would help you a little:

```ts
type VaultFailure = "overflow" | "insufficient" | "zero" | "not-owner";
```

A Rust enum is that idea with two properties the union does not have. First, a variant can carry data (`Option::Some(u64)` carries a `u64`; `None` carries nothing). Second — and this is the one that matters today — **the compiler will not let you handle some of the variants and forget the rest.**

```rust
match failure {
    VaultError::Overflow => "overflow",
    VaultError::InsufficientFunds => "insufficient_funds",
    VaultError::ZeroAmount => "zero_amount",
    // forget NotOwner and the file does not compile
}
```

That property has a name — exhaustiveness — and subgoal 3 is where you feel it rather than read about it.

## `#[error_code]` and the 6000

`#[error_code]` on an ordinary enum generates:

| Generated | What it gives you |
| --- | --- |
| `#[derive(Debug, Clone, Copy)]`, `#[repr(u32)]` | the enum is cheap to pass around, by value |
| `fn name(&self) -> String` | `"ZeroAmount"` — the variant's own name |
| `impl From<VaultError> for u32` | `variant as u32 + 6000` |
| `impl Display for VaultError` | your `#[msg]` string, reachable as `.to_string()` |
| `impl From<VaultError> for Error` | why `.ok_or(VaultError::Overflow)?` converts on the way out |
| `#[msg("...")]` carried into the IDL | the string a client displays |

Two of those are worth separating, because it is easy to attribute the wrong one. **`error!(VaultError::ZeroAmount)` does not use the `From<VaultError> for Error` impl.** It expands to `Error::from(AnchorError { error_name: e.name(), error_code_number: e.into(), error_msg: e.to_string(), … })` — so it uses `name()`, the `u32` conversion, and `Display`, and builds the `Error` from an `AnchorError`. You can prove it: hand-roll an enum with only those three and `error!` compiles on it.

`From<VaultError> for Error` is what `?` needs, which is lesson 12's subject and the reason `.ok_or(VaultError::Overflow)?` works in lesson 13 without an `error!` anywhere in the expression. Two routes to the same wire format, and knowing which is which is the difference between reading an `E0277` and guessing at it.

The 6000 is `anchor_lang::error::ERROR_CODE_OFFSET`. Everything below it is taken: the Solana runtime's own errors, and Anchor's built-in `ErrorCode` (`AccountDiscriminatorMismatch`, `ConstraintSeeds`, and about a hundred others). Your first variant is therefore **6000**, your second is 6001, and so on down the declaration order.

So the order of variants in the enum is part of your program's public interface. Insert a new variant in the middle and every code after it shifts by one — clients that hard-coded `6002` are now reading the wrong failure. Add at the end.

This is also why Anchor's guidance is **one `#[error_code]` enum per program**. Nothing stops you writing two — it compiles — and that is the problem: both start numbering at 6000, so `OtherError`'s first variant and `VaultError`'s first variant are the same code on the wire and no client can tell them apart. If you genuinely need a second enum, the mechanism is an explicit offset, `#[error_code(offset = 1000)]`, which replaces 6000 rather than adding to it.

## `Result<T>` in an Anchor program

`Result` is just an enum with two variants, `Ok(T)` and `Err(E)`. Anchor gives you a type alias:

```rust
// anchor_lang::Result<T> is
core::result::Result<T, anchor_lang::error::Error>
```

That is what `Result<u64>` means everywhere in this file — one type parameter, because the error half is already decided. You will read `Result<()>` constantly in Course 3: an instruction handler that succeeds and returns nothing.

`error!(VaultError::Overflow)` builds the `Error` value, and it captures the file and line where you wrote it, which is why the log tells you *where* the failure was raised, not just what it was.

There are two shorthands for this — `err!()` and `require!()` — and they belong to lesson 12. Today you write `match` and `Err(error!(...))` by hand, because the shorthands are much easier to read once you have seen what they replace.

## Matching on `Option`, matching on `Result`

Both are enums, so both are matched the same way, and this is the whole of subgoal 2:

```rust
// Option -> Result: keep the value, name the failure
match balance.checked_add(amount) {
    Some(new_balance) => Ok(new_balance),
    None => Err(error!(VaultError::Overflow)),
}

// Result -> plain value: decide what a failure means here
match add_lamports(balance, amount) {
    Ok(new_balance) => new_balance,
    Err(_) => balance,        // a rejected deposit changes nothing
}
```

`Err(_)` is the one place a wildcard is the right answer: you are deliberately saying *any* failure means the same thing here. Contrast that with subgoal 3, where a wildcard would be a bug.

## `unwrap()`, and why this lesson is where it gets argued

Lesson 4 banned `.unwrap()` in one sentence and said module 3 would do it properly. This is that.

`.unwrap()` takes an `Option` or a `Result` and either hands you the value or panics. You will see it in nearly every Rust tutorial you read, Solana ones included, and you must never put it in program code.

A panic aborts the instruction with a generic runtime failure. The client gets no error code, no `#[msg]` string, no way to distinguish your four named failures from each other or from a bug. Every bit of the design in this lesson is thrown away at the moment you write `.unwrap()`.

That is the argument, and it is the only place in this course that makes it at length. `unwrap()` gets named again later — the compiler suggests it at you in lesson 9, and lesson 13 lists it among the things the vault core must not contain — but the reason is here.

## Subgoal 3 is the point

Write the two exhaustive `match`es, get them green, and then add a fifth variant — `VaultFrozen` — and build **before** you change anything else.

You will get two errors, one per match site:

```
error[E0004]: non-exhaustive patterns: `VaultError::VaultFrozen` not covered
  --> src/lib.rs:77:11
   |
77 |     match e {
   |           ^ pattern `VaultError::VaultFrozen` not covered
```

That is the compiler handing you a to-do list of every place in the program that now has an unhandled case. If you have ever shipped a `switch` with a missing branch and found out from a user, this is the version of that bug that cannot happen.

And this is exactly what a `_ =>` arm costs you. Add a catch-all to those matches and the fifth variant compiles silently, mapping to whatever the catch-all says — which will be wrong, and quiet about it. A wildcard is not a shortcut for "handle the rest"; it is a decision to stop being told when the set changes.

## What the grader can and cannot see

The build server checks that your file **compiles** against Anchor 1.1.2. It does not run your functions.

The `mod verify` block at the bottom is not editable, and it pins each of the five function names to its exact signature — a missing function or a drifted type stops the build. That is a real check, and it is a check on *names and types only*. Nothing verifies that your `sub_lamports` subtracts rather than adds. Subgoal 3 is different: exhaustiveness is a compile-time property, so the compiler really does verify it, and that is why the fifth variant is where the lesson spends its effort.

One more thing the build does check, for a duller reason: each stub body ships a `compile_error!` naming its subgoal, so the file as delivered cannot build. Without it the placeholder bodies would compile green and an untouched submission would pass — which would tell you nothing about whether you had learned anything. Delete each one as you replace the line beneath it.
