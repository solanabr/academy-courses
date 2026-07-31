# Error Plumbing with `?`

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (latest on crates.io, published 2026-06-26) · `borsh 1.8.0` · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · edition 2021. Every compiler message quoted below was produced by compiling this lesson's file; the `require!` / `require_keys_eq!` expansions were read out of `anchor-lang 1.1.2`'s source.

Lesson 11 gave every failure a name. Now they have to travel.

A real instruction is a chain: read the bytes, validate what they say, apply the policy, do the arithmetic. Any link can fail, and the failure has to arrive at the top intact. Written by hand, the middle of that chain is nothing but plumbing:

```rust
let amount = match read_amount(data) {
    Ok(amount) => amount,
    Err(e) => return Err(e.into()),
};
```

Four lines to say "if this failed, stop." `?` is that, in one character.

## What `?` actually does

```rust
let amount = read_amount(data)?;
```

desugars to roughly:

1. Evaluate `read_amount(data)`.
2. If it is `Ok(v)`, the expression's value is `v` and execution continues.
3. If it is `Err(e)`, **return `Err(From::from(e))` from the enclosing function immediately.**

Step 3 is the whole lesson. `?` does not just propagate the error, it **converts** it — by calling `From::from` on the way out. Which means `?` only works across two different error types if a `From` impl connects them.

That is why `?` stops feeling arbitrary once you have lesson 10's trait material. It is not a special language feature that knows about your errors. It is one trait lookup, and you can write the trait impl yourself.

## The seam this file has

The exercise chain has three layers and two error types.

| Layer | Function | Fails with |
| --- | --- | --- |
| Layout | `read_amount(&[u8])` | `LayoutError` — a plain enum |
| Policy | `check_amount(u64)`, `check_owner(&Pubkey, &Pubkey)` | `VaultError` — the `#[error_code]` enum |
| Caller | `withdraw_amount(...)` | `anchor_lang::error::Error` |

`LayoutError` is deliberately **not** an `#[error_code]` enum. Lesson 11's rule holds: one `#[error_code]` per program, because a second one numbers from 6000 too and collides on the wire. A malformed instruction is an internal concern, so `LayoutError` stays a plain enum and is translated at the boundary:

```rust
impl From<LayoutError> for anchor_lang::error::Error {
    fn from(e: LayoutError) -> Self {
        match e {
            LayoutError::TooShort => error!(ErrorCode::InstructionDidNotDeserialize),
        }
    }
}
```

`ErrorCode` there is Anchor's own built-in enum — the one that lives below 6000 — and `InstructionDidNotDeserialize` is exactly what a client should be told when the instruction data was the wrong shape.

Delete that impl and the chain stops compiling:

```
error[E0277]: `?` couldn't convert the error to `anchor_lang::prelude::Error`
   |
   | ) -> Result<u64> {
   |      ----------- expected `anchor_lang::prelude::Error` because of this
   |     let amount = read_amount(data)?;
   |                  -----------------^ the trait `std::convert::From<LayoutError>` is not implemented for `anchor_lang::prelude::Error`
   |                  |
   |                  this can't be annotated with `?` because it has type `Result<_, LayoutError>`
   |
note: `LayoutError` needs to implement `Into<anchor_lang::prelude::Error>`
```

Read that error twice. It is not saying `?` is broken. It is naming the exact trait impl that is missing, which is the most useful error message in this course.

## `require!` and `err!`, with the covers off

Both are shorthands you have already written longhand.

```rust
require!(amount > 0, VaultError::ZeroAmount);
// expands to:
if !(amount > 0) { return Err(error!(VaultError::ZeroAmount)); }

err!(VaultError::Overflow)
// expands to:
Err(error!(VaultError::Overflow))
```

That is the entire definition of each. `require!` is for testing a condition; `err!` is for returning an error you have already decided on — a `match` arm, usually. Neither does anything you could not type yourself, and both make the intent legible at a glance, which in a program full of guard clauses is worth a lot.

One near-miss worth knowing: for pubkeys, use `require_keys_eq!`, not `require_eq!`. `require_eq!` will compile — `Pubkey` implements `ToString`, so the generic version accepts it — but Anchor's own docs reserve it for non-pubkey values, and `require_keys_eq!` records the two operands as pubkeys and logs them with `Pubkey::log()` rather than allocating two strings. Same check, fewer compute units, and the log shape a client expects.

## `?` does not work on `Option`

This is the error you will hit most, and the compiler is unusually helpful about it:

```
error[E0277]: the `?` operator can only be used on `Result`s, not `Option`s,
              in a function that returns `Result`
   |
   |     let remaining = balance.checked_sub(amount)?;
   |                                                ^ use `.ok_or(...)?` to provide
   |                                                  an error compatible with
   |                                                  `Result<u64, Error>`
```

The reason follows from step 3 above: `?` converts the error by calling `From::from` on it, and `None` is not an error value — there is nothing to convert. So you supply one:

```rust
let remaining = balance
    .checked_sub(amount)
    .ok_or(error!(VaultError::InsufficientFunds))?;
```

`ok_or` turns `Option<u64>` into `Result<u64, Error>` by naming the failure, and *then* `?` has something to carry. This single line is the bridge from lesson 4's `Option`-returning helpers to a program that reports real errors — and it is where `checked_sub` finally becomes `VaultState::withdraw`.

## Closing the lesson-4 loop

Lesson 4's helpers were `const fn`, and they matched on `Option` by hand instead of using `?`. Now you know why: **`?` is not allowed in a `const fn`.**

```
error[E0015]: `?` is not allowed on `Option<u64>` in constant functions
  = note: calls in constant functions are limited to constant functions, tuple
          structs and tuple variants
```

The note is the mechanism: `?` desugars to calls into the `Try` machinery, those functions are not `const fn`, and a `const fn` may only call `const fn`. So the compiler does not report a missing feature — it reports a non-const call. `match` works in a `const fn`; `?` does not. That constraint is what made the compile-time assertion harness in lesson 4 possible, and it is why those two functions look different from everything in this file.

## The exercise

The file does not compile as given, and that is the exercise. Every line you need is present; two functions have had their statements shuffled, and one of them carries two decoy lines.

Build first. The compiler's first pass gives you four errors, and each one is a location:

- `error[E0425]: cannot find value 'head' in this scope` — a line uses a name that a later line defines. Order.
- `error[E0425]: cannot find value 'amount' in this scope` — same problem, other function.
- `error[E0308]: mismatched types ... expected 'u64', found 'Result<u64, LayoutError>'` — decoy: somebody dropped the `?`.
- `error[E0277]: the '?' operator can only be used on 'Result's, not 'Option's` — decoy: `?` on `checked_sub`.

Work in two passes: fix the order first, then delete the decoys. Do not try to repair a decoy — it is meant to go.

Both decoys are compile errors rather than silent bugs, which buys something real and less than it sounds. What the grader can tell is that **neither decoy survived** — leave either one in place and the build is red. What it cannot tell is whether you kept every *correct* line, or what order you put them in. Delete `let amount = check_amount(amount)?;` as though it were a third decoy and the file compiles green with the zero-amount policy simply gone. Move `check_owner(signer, owner)?;` to the bottom and it compiles green with the authorisation check running after the arithmetic — the exact inversion of step 1 above.

So: the two decoys are enforced, the ordering and the completeness are on you. That is the same limit as every other exercise in this course, narrowed by two lines rather than removed. It is still why the lesson is shaped as an ordering-and-discrimination task — a decoy that fails loudly is worth more than a blank page — but do not read a green build as "the chain is right".
