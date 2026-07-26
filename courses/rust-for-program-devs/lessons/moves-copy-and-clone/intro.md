# Moves, Copy, and Clone

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (crates.io latest stable) · `borsh` 1.5.7 declared / **1.8.0** resolved · edition 2021 · Agave ≥ 3.1.10 / rustc per the build server's pinned platform-tools. Every line in this lesson's exercise was compiled against that toolchain.

The two functions you wrote in the last lesson never argued with you about ownership. That is not because you got it right — it is because `u64` is a special case. Four lessons in, every value you have touched has been a number small enough to sit in a register, and Rust duplicates those for free.

The moment a value owns something — a `Vec<u8>` of account data, a struct you defined yourself — assignment stops meaning what it means in JavaScript, and the compiler starts refusing code that looks obviously fine.

## The one sentence

In JavaScript, `const b = a` gives you **two names for one object**.

```js
const vault = { owner: "…", balance: 500n };
const backup = vault;
backup.balance = 0n;
vault.balance; // 0n — there was only ever one object
```

In Rust, `let b = a` gives you **one name and invalidates the other**.

```rust
let vault = VaultLike { /* … */ };
let backup = vault;   // `vault` is no longer usable
```

Nothing was copied and nothing was freed. The value did not move in memory — the *right to use it* moved, from `vault` to `backup`, at compile time. Reading `vault` afterwards is not a runtime error you can catch. It is a build failure, `E0382: use of moved value`, and the message points at both lines:

```text
error[E0382]: use of moved value: `vault`
    |
    |     let vault = sample_vault();
    |         ----- move occurs because `vault` has type `VaultLike`,
    |               which does not implement the `Copy` trait
    |     let backup = vault;
    |                  ----- value moved here
    |     let _oops = vault.balance;
    |                 ^^^^^^^^^^^^^ value used here after move
```

Two lines named, and the reason stated in the middle of them: *does not implement the `Copy` trait*. Almost every ownership error you will read has that shape — the line that failed, the line that caused it, and the trait that would have made it legal.

## Three rules, and they are actually three

1. Every value has exactly one owner.
2. When the owner goes out of scope, the value is dropped.
3. Ownership can be **moved**, **copied** (only for types that opt in), or **borrowed** (next lesson).

That is the whole model. There is no garbage collector deciding when rule 2 fires; the compiler inserts the drop at the closing brace, and that is why Rust has no runtime and no GC pauses — which is why programs on Solana are written in it.

## Why you already know this shape

You met this rule in Course 1 without it being called ownership. A Solana transaction declares, up front, which accounts it will **read** and which it will **write**, and the runtime will not schedule two transactions that write the same account at the same time. Many readers, or one writer. Never both.

Rust's rule for references is the same sentence: many shared borrows, or one mutable borrow, never both. The runtime enforces it across transactions at runtime; the compiler enforces it inside your function at build time. Same invariant, two places. When `&mut` starts feeling arbitrary, this is the analogy to come back to.

## `Copy` is opt-in, and that is the whole trick

A type is `Copy` only if it says so. Assignment copies instead of moving for:

| Type | Why |
| --- | --- |
| `u8`, `u64`, `i64`, `usize`, `bool` | Fixed size, no heap, no destructor |
| `Pubkey` | 32 plain bytes; the type derives `Copy` |
| `[u8; 8]` | An array of `Copy` values is `Copy` |
| `&T` (a shared reference) | A reference is itself just an address |

Assignment **moves** for:

| Type | Why |
| --- | --- |
| `Vec<u8>` — a raw account buffer | Owns a heap allocation. Two owners would mean two frees |
| `String` | Same |
| `VaultLike`, and any struct you write | **`Copy` is not inferred.** A struct whose every field is `Copy` still moves unless you derive `Copy` on the struct itself |

That last row is the one that catches people. `VaultLike { owner: Pubkey, balance: u64, bump: u8 }` is 41 bytes of pure `Copy` fields, and it still moves — because `Copy` is a claim about the *type*, not a computed property of its fields, and Rust makes you state it.

## `.clone()` is allowed

There is a genre of Rust advice that treats `.clone()` as a confession. Ignore it. `.clone()` is an explicit, visible, correct way to get a second independent value, and reaching for it while the model is still settling is the right call. It has exactly one cost: the copy actually happens at runtime, and for a `Vec` that is a heap allocation.

The reason it barely matters here is that the vault you are building is 41 bytes. The reason it will matter in Course 3 is that account data can be kilobytes and compute units are metered. Clone now, and learn where the borrow goes later — in that order.

## What you are about to do

This exercise is complete and correct. There is nothing to fix. Build it, then read it top to bottom against its six numbered sections and check each one against the prediction you would have made before this page.

Section 6 is a single-line experiment: uncomment the line marked `BREAK IT`, build, read the error the compiler gives you, then comment it back. That error message is the one you will see most often for the next week, and meeting it deliberately once is cheaper than meeting it by accident five times.
