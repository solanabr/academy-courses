# Checked Lamport Math

> **Version stamp — checked 2026-07-26.** `anchor-lang` **1.1.2** · Rust **edition 2021** · Agave **≥ 3.1.10**. The build crate sets `overflow-checks = true` in `[profile.release]`, which this lesson depends on and discusses. The exercise is plain Rust and imports nothing.

Here is a withdrawal, written the way it reads:

```rust
let new_balance = balance - amount;
```

A vault holding 5 lamports, someone asking for 7. Decide what that line does before reading on, and be specific — there are three different answers depending on how the program was compiled, and none of them is the one you want.

## The three answers

**In a debug build, or any build with overflow checks on: it panics.** The subtraction is detected as an underflow and the program aborts. On-chain, an aborted program means a failed transaction, a log line about a panic, and no way for a caller to distinguish "you asked for too much" from "the program has a bug". You cannot handle it, you cannot match on it, and you cannot return a useful error alongside it.

**In a release build with overflow checks off: it wraps.** `5u64 - 7` becomes 18 446 744 073 709 551 614. No panic, no log, no signal of any kind. Your vault now believes it holds eighteen quintillion lamports, and the very next line of code will act on that belief. This is not a hypothetical failure mode — unchecked arithmetic is one of the most-exploited bug classes in on-chain programs, and it is exploited precisely because it is silent.

**The third answer is the one you build today: it returns `None`.** No panic, no wrap. The impossible result is represented as a value, handed back to the caller, and the caller has to deal with it before it can get at a number.

Our build server compiles with `overflow-checks = true`, so a wrap becomes a panic here. Be clear about what that does and does not buy you. It converts a silent corruption into a loud crash, which is strictly better and is why it is switched on. It is **not** a substitute for checked arithmetic: a crash is still not an error you can return, and the flag is a property of the build profile rather than of your code. Code whose correctness depends on a profile setting is code that is one `Cargo.toml` away from being wrong.

This is the Rust half of the `u64` problem lesson 2 planted on the JavaScript side. There, `Number(hugeU64)` handed you a nearby number with no complaint. Here, `balance - amount` hands you a wrong number or a crash. The languages fail differently and both failures are silent-or-useless by default. The fix in Rust is to ask for the checked operation by name.

## `Option<T>`: a value or the absence of one

```rust
pub enum Option<T> {
    Some(T),
    None,
}
```

That is the whole definition. `Option<u64>` is *either* a `u64` wrapped in `Some`, *or* `None`. It is an ordinary enum from the standard library, not compiler magic, and it is what Rust has instead of `null`.

The difference from `null` is not philosophical, it is mechanical: **`Option<u64>` and `u64` are different types.** You cannot add 1 to an `Option<u64>`, pass it where a `u64` is wanted, or compare it to a number. To get at the value you have to handle the `None` case, and the compiler will not let you skip it. There is no equivalent of a null-pointer dereference to defer to run time, because the check is not at run time.

Rust's standard library gives every integer type a `checked_` operation for exactly this shape:

```rust
5u64.checked_sub(7)   // None
7u64.checked_sub(5)   // Some(2)
u64::MAX.checked_add(1)  // None
1u64.checked_add(2)   // Some(3)
```

`checked_add`, `checked_sub`, `checked_mul`, `checked_div`. Each returns `Option`, each returns `None` exactly when the true result cannot be represented, and using them is not defensive programming — it is the only way to write arithmetic on untrusted numbers that says what it means.

## Getting the value out with `match`

`match` from lesson 3 works on `Option`, and this is where its exhaustiveness earns its keep — there are two variants, so there are two arms, and forgetting one is `error[E0004]`:

```rust
match balance.checked_sub(amount) {
    None => { /* the caller asked for too much */ }
    Some(remaining) => { /* `remaining` is a plain u64 here */ }
}
```

`Some(remaining)` both tests the variant and binds the value out of it in one move. That is destructuring, and it is the normal way to read any enum in Rust.

You may have seen `.unwrap()` do the same job in one word. `unwrap()` panics on `None`. **It must never appear in program code**, it does not appear anywhere in this course's code, and every tutorial you find will be full of it. Lesson 11 makes the full argument for why; take the rule on trust until then.

There is also `?`, the operator that propagates the failure case for you and turns a five-line `match` into one character. You will meet it in lesson 12, and there is a specific reason it is not available today — see the constraints below.

## Why these are `const fn`

Every function you write in this exercise is declared `const fn`. That means the compiler is able to evaluate it during compilation, given constant inputs.

This is not decoration and it is not for performance. It is what makes the exercise gradeable. Grading a Rust submission here means compiling it — that is the whole signal. A file that compiles has told the grader nothing about whether `sub_lamports` subtracts or adds. But a `const fn` can be called by an assertion the compiler must evaluate:

```rust
const _: () = assert!(matches!(sub_lamports(5, 7), None));
```

If your `sub_lamports(5, 7)` returns anything other than `None`, that assertion fails **during compilation** and the build goes red, with the assertion reproduced in the error:

```
error[E0080]: evaluation panicked: assertion failed: matches!(sub_lamports(5, 7), None)
```

Ten of those sit at the bottom of the file. They are the first behavioural check in this course rather than a signature check, and they are what makes a `sub_lamports` that quietly calls `checked_add` fail rather than pass.

Be clear about the size of that win. Ten specific inputs is not a test suite, and this trick reaches only as far as `const fn` does — by module 4, when the same logic becomes a method taking `&mut self` and returning `Result`, it is no longer const-evaluable and the harness goes back to pinning shapes. Today it works, so today we use it.

Two consequences of `const fn` you need before you start:

- **`?` is not allowed in a `const fn`.** You get `error[E0015]` — "`?` is not allowed on `Option<u64>` in constant functions" — with the note "calls in constant functions are limited to constant functions, tuple structs and tuple variants". That note is the actual reason: `?` desugars to calls into the `Try` machinery, and those functions are not `const fn`, so from the compiler's point of view you made a non-const call. Use `match`. This is the reason the spec says `match`, and lesson 12 is where `?` finally arrives — on `Result`, in a function that is not const.
- **`panic!`, and anything built on it, ends the build rather than the program.** A `panic!` reached during const evaluation is a compile error. `unwrap()` on a `None` in a `const fn` called from an assertion does not fail at run time; it fails at *your* keyboard.

`checked_add` and `checked_sub` are themselves `const fn`, so they are available to you inside yours.

## The spec

Three functions, no bodies. Signatures are fixed and pinned by the harness.

### `add_lamports(balance, amount) -> Option<u64>`

`Some(sum)` when `balance + amount` is representable as a `u64`. `None` when it is not.

`amount` of zero is fine and returns `Some(balance)`. Rejecting a zero-amount deposit is a *policy*, and it belongs in the vault's `deposit` method in lesson 13, not in an arithmetic helper. Keep the two separate.

### `sub_lamports(balance, amount) -> Option<u64>`

`Some(remaining)` when `amount <= balance`. `None` when it is not, because a `u64` has no negative values to land on. Withdrawing exactly the whole balance is valid and returns `Some(0)`.

### `transfer_lamports(from, to, amount) -> Option<(u64, u64)>`

Move `amount` from one balance to the other and return **both new balances** as `Some((new_from, new_to))`. Return `None` if either half is impossible.

This one is the point of the lesson. The requirement is *all or nothing*: if the debit works but the credit would overflow, nothing moved, and the answer is `None` — not `Some` with one side updated. Which means you cannot compute the two halves independently and hope; you have to look at the first result before committing to the second, and you have to do it with `match` because `?` is unavailable. This is the shape of every value transfer you will ever write on-chain, and it is why `withdraw` in lesson 13 does its checked subtraction *before* it writes anything.

Build the empty file first. Ten `error[E0080]` lines come back, one per assertion, all of them reporting `not yet implemented` — that is `todo!()` doing its job. As you fill each body in, the failures that remain start naming the specific input they disagree with, and at that point the harness is a better-specified to-do list than this page is.
