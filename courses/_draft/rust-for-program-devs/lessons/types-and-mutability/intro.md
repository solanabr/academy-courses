# Types, Mutability, and Why the Compiler Argues

> **Version stamp — checked 2026-07-26.** `anchor-lang` **1.1.2** · Rust **edition 2021** · Agave **≥ 3.1.10**. This lesson's exercise is plain Rust and imports nothing — the arguments you are about to have with the compiler are not Anchor's.

The compiler is about to reject four things a JavaScript engine would have accepted without comment. All four rejections are about the same disagreement: **JavaScript defers, Rust decides.**

JavaScript has one number type and it decides what to do with your value at run time. Rust has twelve integer types and decides what to do with your value while compiling, which means it has to be told. Being told is what the next twenty minutes are.

## The four widths the vault needs

You are not going to learn twelve integer types. You need four, and each one is fixed by something outside your control.

| Type | What it holds | Why that width |
| --- | --- | --- |
| `u64` | lamports — a balance, a deposit amount, a slot number | The runtime stores lamport balances as unsigned 64-bit integers. This is not a choice you get to make. |
| `u8` | a PDA bump seed | A bump is one byte, `0..=255`, by construction. The search that finds it walks downward from 255. |
| `i64` | a UNIX timestamp from the cluster clock | The clock's `unix_timestamp` is **signed**. It is `i64`, not `u64`, so a timestamp field in your struct is `i64` too. |
| `usize` | an index into a byte slice | The type Rust uses to index collections. On the SBF target it is 8 bytes wide, exactly like `u64` — which is precisely why treating the two as interchangeable is a habit that breaks the day something is not 64-bit. |

`u` is unsigned, `i` is signed, the number is the bit width. `u8` holds `0..=255`; `i64` holds roughly ±9.2 × 10¹⁸.

## The one that will bite you outside of Rust

`u64` holds up to 18 446 744 073 709 551 615. JavaScript's `Number` is an IEEE-754 double, so it represents every integer exactly only up to 2⁵³ − 1, which is 9 007 199 254 740 991 — smaller by a factor of 2¹¹, about two thousand times.

Both of those are large. The gap between them is not theoretical: it is roughly nine million SOL expressed in lamports, and mainnet holds considerably more than that.

The failure mode is what makes this worth planting now. JavaScript does not throw when you exceed it. It does not warn. It hands you a nearby number and continues:

```js
Number(18_446_744_073_709_551_615n); // → 18446744073709552000
```

Nothing in that line is an error, and nothing downstream can tell that a value was quietly rounded. This is why a Solana client reads `u64` fields as `BigInt`, why an amount arriving over the wire as a JSON number is a bug waiting for a large account, and — the reason it is in *this* lesson — why the balance field you will write in module 4 is a `u64` and stays one.

Rust's version of the same class of problem does not get to be silent. That is lesson 4.

## `mut` is not a type, and it appears in two places

Everything in Rust is immutable unless declared otherwise, including local variables. That trips people up in a specific way, because `mut` shows up in two syntactically unrelated positions.

**On a binding** — permission to reassign the variable:

```rust
let bump: u8 = 255;
bump -= 1;          // error[E0384]: cannot assign twice to immutable variable

let mut bump: u8 = 255;
bump -= 1;          // fine
```

**On a reference** — permission to write through it:

```rust
fn credit(vault: &VaultLike, amount: u64) {
    vault.balance = amount;   // error[E0594]: cannot assign to `vault.balance`,
}                             // which is behind a `&` reference

fn credit(vault: &mut VaultLike, amount: u64) {
    vault.balance = amount;   // fine
}
```

Note what the second one is *not*: it is not about `pub`. Making a field public controls who can name it, not who can change it. And it is not about the caller — a caller holding a perfectly mutable value still cannot write through a `&T` you asked for. The function's signature is the contract, and `&T` is a promise not to write.

`&T` and `&mut T` are a much bigger subject than "add the keyword" — module 2 is entirely about the rules that make them safe. For now, take the compiler's suggestion and move on.

## Rust converts nothing for you

This is the rule with the widest reach and the shortest statement: **there are no implicit numeric conversions in Rust.** None. Not even the lossless ones.

```rust
fn bump_as_u64(bump: u8) -> u64 {
    bump    // error[E0308]: expected `u64`, found `u8`
}
```

Every `u8` value fits in a `u64` with room to spare. C widens it for you, Java widens it for you, JavaScript does not have the distinction. Rust makes you write it:

```rust
u64::from(bump)
```

`From` is the conversion that cannot fail, so it needs no error handling and no decision from you. It exists between exactly those pairs of types where the target can hold every value of the source. `u64::from(u8)`, yes. `u64::from(i64)`, no — negative numbers have nowhere to go.

You will also see `as`:

```rust
let small = big as u8;   // compiles for any integer `big`
```

`as` always compiles and silently discards whatever does not fit. `300 as u8` is 44. `-1i64 as u64` is 18 446 744 073 709 551 615. That is a wrapping cast wearing the costume of a conversion, and it is a genuinely common source of exploitable arithmetic in on-chain programs. **This course does not use `as` on numbers.** When the conversion cannot fail, `u64::from` says so. When it can, module 3 gives you `TryFrom` and a `Result` to handle.

The other side of the same rule: when two values need to be compared, do not convert one to match the other by reflex. Ask which width is *correct*. A timestamp is `i64` because the clock says so, so the parameter you compare it against should have been `i64` in the first place.

## Indexing wants `usize`

```rust
fn byte_at(data: &[u8], index: u64) -> u8 {
    data[index]   // error[E0277]: the type `[u8]` cannot be indexed by `u64`
}
```

`&[u8]` is a **slice** — a borrowed window onto a run of bytes, which is how raw account data arrives. Indexing it requires `usize`, the platform's pointer-sized integer, and that requirement is a trait bound rather than a coercion, which is why the error is `E0277` (a trait is not implemented) rather than `E0308` (mismatched types). Two different error codes for what feels like one mistake; recognising which one you have is worth the ten seconds.

One warning about the exercise, so it does not surprise you later: `data[index]` **panics** if the index is past the end of the slice, and a panic in a program aborts the transaction with no useful error. That is a real defect, it is deliberate, and lesson 8 replaces it with `data.get(index)` — which returns an `Option` instead of exploding. We are not equipped for `Option` yet. Lesson 4 is where that starts.

## The exercise

The file ships broken. That is the exercise: four numbered subgoals, the first already done and worth reading, the other three each producing errors you now know how to read.

Below the marked line there is a block you must not edit. It is a set of bindings that pin every function's name and signature, so that deleting a function you cannot fix is a compile error rather than a way through. It checks **shapes only** — nothing in it can tell a correct body from a plausible wrong one. That limit is real, and lesson 4 is where it partially lifts.

No `unwrap()`, no `expect()`, no `as`.
