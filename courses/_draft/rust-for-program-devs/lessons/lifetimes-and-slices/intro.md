# Lifetimes and Slices

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (crates.io latest stable) · `borsh` 1.5.7 declared / **1.8.0** resolved · edition 2021 · Agave ≥ 3.1.10 / rustc per the build server's pinned platform-tools. `anchor-lang` 1.0 removed three of the four redundant lifetime parameters from `Context`, so the surface described here is smaller than in any tutorial written before June 2026.

Bug 4 in the last lesson ended with a question the compiler asked and you answered by accident: **which value does the returned reference borrow from?**

For `head(data: &[u8]) -> &[u8]` there is exactly one candidate, so the compiler filled it in silently. Add a second reference parameter and it stops guessing, because guessing wrong would mean returning a reference to something already freed.

That is all a lifetime annotation is. Say it out loud once:

> **A lifetime does not change how long anything lives. It tells the compiler which input a returned reference came from.**

Nothing is allocated, nothing is freed, nothing is reference-counted, and no code is generated. Annotations are a compile-time claim, checked and then discarded.

## What `'a` actually says

```rust
pub fn discriminator<'a>(data: &'a [u8]) -> Option<&'a [u8]>
```

Word for word: *for some lifetime `'a`, I take a slice that is valid for `'a`, and if I return a slice it is valid for the same `'a`.* Which means: the thing I hand back points into `data`, so it stays valid exactly as long as `data` does — and the compiler will reject any caller who drops `data` while still holding the result.

You can drop the annotations here:

```rust
pub fn discriminator(data: &[u8]) -> Option<&[u8]>
```

This is the identical signature. **Lifetime elision** fills in the same `'a` for you when there is exactly one input reference, because there is only one possible answer. That is why you have written a dozen functions taking `&VaultLike` without ever typing a `'`.

Elision has two other rules worth knowing, and no more:

- One input reference → the output borrows from it.
- A method with `&self` or `&mut self` → the output borrows from `self`, no matter how many other reference parameters there are.
- Otherwise → **you must say.**

`balance_of(&self) -> u64` returns no reference, so it never comes up. `deposit(&mut self, amount: u64)` likewise. Which is why module 4's vault has no annotations in it anywhere, and why meeting them here rather than there is the right order.

## The case elision cannot solve

```rust
pub fn discriminator_if(data: &[u8], expected: &[u8; 8]) -> Option<&[u8]>
```

Two input references, one output reference, no `self`. The compiler will not choose:

```text
error[E0106]: missing lifetime specifier
  |
  | pub fn discriminator_if(data: &[u8], expected: &[u8; 8]) -> Option<&[u8]>
  |                               -----            --------            ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the
          signature does not say whether it is borrowed from `data` or `expected`
```

The help text *is* the lesson. There is a right answer here and it is a design decision, not a formality: the discriminator you return is bytes out of the account, so it borrows from `data`. `expected` is a caller's constant that this function only reads and never hands back — it gets its own lifetime, unrelated to the return value.

```rust
pub fn discriminator_if<'a, 'b>(data: &'a [u8], expected: &'b [u8; 8]) -> Option<&'a [u8]>
```

Two names because there are two independent answers. Writing `<'a>` on both would compile the function and quietly over-constrain every caller — it would demand that the account buffer and the expected constant live equally long, which is a promise callers have no reason to be able to make.

## Slices, and why the first 8 bytes matter

`&[u8]` is a **borrowed view** into bytes someone else owns: an address plus a length, 16 bytes total, no allocation, no copy. `Vec<u8>` owns its buffer; `&[u8]` looks at it. That distinction is the whole of bug 4.

The bytes you are about to look at are real. Every Anchor account starts with an 8-byte discriminator — a hash of the account type's name — followed by the fields:

| Offset | Bytes | Field |
| --- | --- | --- |
| 0 | 8 | discriminator |
| 8 | 32 | `owner: Pubkey` |
| 40 | 8 | `balance: u64`, little-endian |
| 48 | 1 | `bump: u8` |

Total 49. This is the same table you decoded byte by byte in Course 1, and the same layout `#[account]` will generate for you in module 3. Anchor checks that discriminator before it deserializes anything — wrong discriminator, wrong account type, refuse — which is exactly the function you are about to write.

## What you are writing

Two functions, signatures given, spec given, no worked pattern. Both are `const fn`, for a reason worth stating plainly:

**Grading on this platform is compile-only.** A submission passes when the build server reports that it compiled. There are no hidden tests on the Rust path, so a function whose body was `todo!()` would normally sail through.

`const fn` closes that. The exercise ships a `mod verify` block you may not edit, which calls your functions at **compile time** and asserts on the answers:

```rust
const _: () = assert!(super::discriminator(&[0u8; 7]).is_none());
```

A `const fn` that panics — including `todo!()` — fails const evaluation, which is a compile error. A `discriminator` that returns 4 bytes fails the length assertion, which is a compile error. So for this lesson the build genuinely does check your logic, not just your syntax.

Be clear about the limit: this works because both functions are `const fn` over plain slices. It will not work for `deposit(&mut self) -> Result<()>` in module 4 — you cannot const-evaluate a method that mutates account state — and that lesson's harness therefore checks names and types only, and says so.
