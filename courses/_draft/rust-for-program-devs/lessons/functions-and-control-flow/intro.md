# Functions, Expressions, and the Missing `return`

> **Version stamp — checked 2026-07-26.** `anchor-lang` **1.1.2** · Rust **edition 2021** · Agave **≥ 3.1.10**. This lesson's exercise is plain Rust and imports nothing.

Read this function and decide, before scrolling, whether it compiles.

```rust
fn min_balance(a: u64, b: u64) -> u64 {
    if a < b { a; } else { b; }
}
```

It does not. There are two semicolons in it and they are the entire problem. Everything below is about why one character does that, because it is the error JavaScript developers hit in their first week of Rust and it does not go away by being told about it once.

## Almost everything is an expression

In JavaScript there are statements and there are expressions, and `if` is firmly a statement. You cannot assign one to a variable. That is why the ternary exists.

In Rust the split is much narrower. `if` is an expression. So is `match`. So is a block, `{ … }`. Each of them evaluates to a value, so each can appear anywhere a value is wanted:

```rust
let tier = if balance == 0 { "empty" } else { "funded" };
```

No ternary needed, because `if` already does that job. Rust has no `?:` operator at all, and this is why.

## What a semicolon actually does

A semicolon does not end a line. It **discards a value** and turns the expression into a statement whose value is `()` — the unit type, "nothing useful".

That is exact, and it is the whole rule:

| Written | Value of the block |
| --- | --- |
| `{ a }` | `a` |
| `{ a; }` | `()` |

Which explains the broken function. `{ a; }` evaluates to `()`, so both arms of the `if` are `()`, so the `if` is `()`, while the signature promises `u64`. The compiler type-checks each arm against what is expected, so it reports this **twice** — once per arm:

```
error[E0308]: mismatched types
 --> src/lib.rs:2:14
  |
1 | fn min_balance(a: u64, b: u64) -> u64 {
  |                                   --- expected `u64` because of return type
2 |     if a < b { a; } else { b; }
  |                 ^  help: remove this semicolon to return this value
  |                 |
  |                 expected `u64`, found `()`
```

Note the `help:`. When you see "remove this semicolon to return this value" you have found this bug and you can stop reading the rest of the error.

## Tail expressions, and why `return` is rare

A function's value is the value of its body block — which means the **last expression, with no semicolon**. That is called the tail expression:

```rust
fn min_balance(a: u64, b: u64) -> u64 {
    if a < b { a } else { b }
}
```

No `return`. Idiomatic Rust uses `return` for one thing: leaving early, from somewhere other than the end.

```rust
fn withdraw(balance: u64, amount: u64) -> u64 {
    if amount == 0 {
        return balance;      // early exit, `return` earns its keep
    }
    balance - amount         // tail expression, no `return`
}
```

A `return` on the last line is not an error and nobody will fail your review for it. It is just noise, and once you have the habit of reading the last line as *the answer*, functions get easier to skim.

Two things people trip on:

- **`return x` needs its semicolon.** It is a statement. `return` and a tail expression are different mechanisms, not two spellings of one.
- **A block's tail works too**, not only a function's. `let x = { let a = 2; a * 3 };` binds 6. This becomes useful in module 2, where the size of a block decides how long a borrow lasts.

## `match` on ranges

`match` is Rust's pattern dispatch, and for our purposes it is a `switch` with two upgrades that matter.

```rust
fn describe_state(balance: u64) -> &'static str {
    match balance {
        0 => "empty",
        1..=999_999 => "dust",
        1_000_000..=999_999_999 => "funded",
        _ => "whale",
    }
}
```

**Upgrade one: it is an expression.** Every arm produces a value, all arms must agree on the type, and the whole `match` is that value. No `break`, and therefore no accidental fall-through — the bug class that eats a `switch` does not exist here.

**Upgrade two: it must be exhaustive.** Every possible value of `balance` has to be matched by some arm, and the compiler proves it. Leave a gap and you get `error[E0004]: non-exhaustive patterns`, which is a compile-time guarantee that a missing case cannot ship. `_` is the catch-all that closes the gap, and `..=` is an inclusive range.

Two details about ordering, because they are the difference between this working and this looking like it works:

**Arms are tried top to bottom, first match wins.** Not most-specific-wins. Order is semantics.

**An arm the earlier ones already cover is a warning, not an error.** Put `0..=999_999 => "dust"` above `0 => "empty"` and the `0` arm becomes dead:

```
warning: unreachable pattern
 --> src/lib.rs:5:9
  |
4 |         0..=999_999 => "dust",
  |         ----------- matches all the values already
5 |         0 => "empty",
  |         ^ no value can reach this
```

That file **compiles**, and this lesson is graded on compilation, so it passes — and `describe_state(0)` returns `"dust"`. This is a good moment to notice how much work a warning can be doing. `unreachable_patterns` is one of the highest-value warnings Rust emits, and nothing forces you to read it.

`&'static str` in the return type is a borrowed string that lives for the whole program — which is true of a literal baked into the binary. The apostrophe is a lifetime, and module 2 is where you learn to write your own. Take it as given for now.

## The exercise

Both function bodies are empty, so the file does not build. Every line you need is in the parts bin at the top of the file, commented out and out of order. **Two of the lines are decoys.** One of them compiles and does the wrong thing; one of them does not compile at all. Working out which is which is the exercise.

Do not add lines that are not in the bin. Assembling exactly the right subset in the right order is a smaller job than writing it from scratch, and it is the whole job here — lesson 4 is where the blank page arrives.
