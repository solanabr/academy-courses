# Worked Answers

> **Version stamp — checked 2026-07-26.** Every message below is real `rustc` output, produced by compiling the snippet above it against `anchor-lang 1.1.2` / borsh 1.5.7 declared / 1.8.0 resolved / edition 2021 — the same toolchain the build server uses. Nothing here is paraphrased.

Read this after the quiz, not before. Getting a question wrong and then reading why is worth several times more than reading first and nodding.

Two of the eight questions had nothing to fail to compile — the misread-balance one and the `BigInt` one. Those two are from Course 1, they are diagnoses rather than predictions, and they are here because you are about to write the Rust struct whose bytes you were decoding in JavaScript six lessons ago.

---

## 1 — A `for` loop consumes what it iterates

```rust
pub fn total(vaults: Vec<VaultLike>) -> (u64, usize) {
    let mut sum = 0u64;
    for v in vaults {
        sum += v.balance;
    }
    (sum, vaults.len())
}
```

```text
error[E0382]: borrow of moved value: `vaults`
   |
   | pub fn total(vaults: Vec<VaultLike>) -> (u64, usize) {
   |              ------ move occurs because `vaults` has type `Vec<VaultLike>`,
   |                     which does not implement the `Copy` trait
   |     for v in vaults {
   |              ------ `vaults` moved due to this implicit call to `.into_iter()`
...
   |     (sum, vaults.len())
   |           ^^^^^^ value borrowed here after move
   |
note: `into_iter` takes ownership of the receiver `self`, which moves `vaults`
help: consider iterating over a slice of the `Vec<VaultLike>`'s content to avoid
      moving into the `for` loop
   |
   |     for v in &vaults {
   |              +
```

`for x in collection` calls `.into_iter()`, which takes `self`. The loop is not doing anything unusual; it is an ordinary by-value call written with sugar, and the sugar is what hides it. `for v in &vaults` iterates `&VaultLike` instead, which is the fix the compiler hands you and the one you want almost every time.

The one-character diff is worth internalising because you will write this loop over `remaining_accounts` in Course 3.

---

## 2 — A trailing semicolon changes the type of the function

```rust
pub fn describe(balance: u64) -> &'static str {
    match balance { 0 => "empty", _ => "funded" };
}
```

```text
error[E0308]: mismatched types
  |
  | pub fn describe(balance: u64) -> &'static str {
  |        --------                  ^^^^^^^^^^^^ expected `&str`, found `()`
  |        |
  |        implicitly returns `()` as its body has no tail or `return` expression
  |     match balance { 0 => "empty", _ => "funded" };
  |                                                  - help: remove this semicolon
  |                                                         to return this value
```

`match` is an expression and produces `&str`. The semicolon throws that value away and turns the line into a statement, so the function body ends with nothing and evaluates to `()`.

Note how precise the diagnostic is — it names the semicolon and offers to delete it. This is the single most common first-week Rust error for anyone arriving from JavaScript, where `return` is mandatory and a stray semicolon is free.

---

## 3 — A balance in the quintillions, from an offset that is 8 bytes early

No compiler involved. Anchor's layout for the account you decoded in Course 1:

| Offset | Bytes | Field |
| --- | --- | --- |
| 0 | 8 | discriminator |
| 8 | 32 | `owner: Pubkey` |
| **40** | 8 | `balance: u64`, little-endian |
| 48 | 1 | `bump: u8` |

49 bytes. `balance` starts at 40, because the discriminator is 8 and a `Pubkey` is 32 — and `8 + 32` is arithmetic you will write literally, as `space = 8 + 32 + 8 + 1`, in Course 3's account constraint. `discriminator(data)` from the last lesson returns the first row of that table.

A read at offset 32 therefore takes the **last eight bytes of the owner's address** and calls them a balance. That is why the symptom is a plausible-looking wrong number rather than a crash: a `Pubkey` is 32 bytes of hash output, so interpreting any eight of them as a little-endian `u64` gives you something in the 10^18 range roughly half the time.

Two things to take from the shape of that bug. First, an off-by-8 in an account decoder produces *data*, not an error — nothing in the byte array is self-describing, which is the whole reason the discriminator exists and the whole reason lesson 8 makes you write the length check. Second, the two other suspects on the list are both real and both worth ruling out in order: wrong endianness (Borsh is little-endian; `getU64Decoder` already knows that) and `Number` precision above 2^53 − 1 (which is why Kit hands you a `BigInt`, and which question 7 is about). Diagnose offset first, because it is the only one of the three that can be wrong while the other two are right.

---

## 4 — `E0502` runs in both directions

```rust
pub fn stash(v: &mut VaultLike) -> u64 {
    let slot = &mut v.balance;
    let total = balance_of(v);
    *slot = total;
    total
}
```

```text
error[E0502]: cannot borrow `*v` as immutable because it is also borrowed as mutable
   |
   |     let slot = &mut v.balance;
   |                -------------- mutable borrow occurs here
   |     let total = balance_of(v);
   |                            ^ immutable borrow occurs here
   |     *slot = total;
   |     ------------- mutable borrow later used here
```

In lesson 7 you read the mirror image of this — *cannot borrow as mutable because it is also borrowed as immutable*. Same rule, reported from whichever side arrived second. Both times the third line named is the one that keeps the first borrow alive.

The fix is the same as it was: `let total = balance_of(v);` **first**, then take the mutable borrow, or skip it entirely and write `v.balance = total;`.

---

## 5 — Rust does not widen integers for you, and the compiler's suggestion here is wrong for us

```rust
pub fn space(field_count: u32) -> usize {
    8 + field_count
}
```

```text
error[E0308]: mismatched types
   |
   | pub fn space(field_count: u32) -> usize {
   |                                   ----- expected `usize` because of return type
   |     8 + field_count
   |     ^^^^^^^^^^^^^^^ expected `usize`, found `u32`
   |
help: you can convert a `u32` to a `usize` and panic if the converted value
      doesn't fit
   |
   |     (8 + field_count).try_into().unwrap()
   |     +               +++++++++++++++++++++
```

Three things to take from this.

First, the error itself: no implicit numeric coercion, ever, not even the apparently-lossless `u32` → `usize`. The literal `8` infers as `u32` from its neighbour, the sum is a `u32`, and the return type is a `usize`.

Second: the obvious infallible conversion is not available. `usize::from(field_count)` does not compile — `impl From<u32> for usize` is not in the standard library, and the reason is that `usize` is 16 bits on some targets, so the conversion is not lossless everywhere. (`impl From<u16> for usize` does exist, for exactly that reason.) What `u32` gets is `TryFrom`:

```rust
pub fn space(field_count: u32) -> Option<usize> {
    match usize::try_from(field_count) {
        Ok(n) => n.checked_add(8),
        Err(_) => None,
    }
}
```

That is the shape lesson 4 taught, applied to a conversion rather than to arithmetic: a fallible operation returns a value that names the failure, and the caller deals with it. Note the return type had to change. That is not a workaround — it is the honest signature.

Third, and most important: **read the suggestion and refuse the second half of it.** `.try_into()` is right; `.unwrap()` is not. `unwrap()` in a program is a panic waiting for the input that triggers it. The compiler does not know it is writing on-chain code. Its diagnostics are excellent and its suggestions are not policy — this course's rule is that `unwrap()` and `expect()` never appear in program code, and module 3 gives you the `Result` machinery that makes obeying it painless.

**And what about `field_count as usize`?** It compiles, it is a widening cast on SBF (a 64-bit target, so `usize` is 8 bytes and nothing can be lost), and lesson 2 still told you not to write it. That rule stands and the carve-out is narrow: `as` is defensible only when you have *pinned the target* and can therefore prove the cast widens. Course 3 compiles for one target and you could argue it there. Here you cannot, because `space` is an ordinary function in a crate that also builds `lib` for the host — and more to the point, `usize::try_from` costs you one `match` and removes the argument entirely. Prefer the conversion that cannot be wrong when the platform changes under you.

---

## 6 — `to_bytes()` hands you an owned array

```rust
pub fn owner_bytes(v: &VaultLike) -> &[u8] {
    let bytes = v.owner.to_bytes();
    &bytes
}
```

```text
error[E0515]: cannot return reference to local variable `bytes`
   |
   |     &bytes
   |     ^^^^^^ returns a reference to data owned by the current function
```

`Pubkey::to_bytes` returns `[u8; 32]` **by value** — a fresh 32-byte array that this function owns and drops on the way out. Same shape as lesson 7's bug 4, different disguise: nothing here looks like an allocation, and the array is on the stack rather than the heap, but ownership is ownership.

The fix names the borrow properly: `v.owner.as_ref()` gives you a `&[u8]` view of the `Pubkey` the caller already owns, and lives as long as `v` does. When you find yourself borrowing from a method's return value, check whether that method returned a reference or a value.

---

## 7 — Why Course 1 handed you a `BigInt`

`balance` is a `u64`, so its maximum is 18,446,744,073,709,551,615. JavaScript's `Number` is a double: integers above 2^53 − 1 (9,007,199,254,740,991) stop being exactly representable, and the excess is silently rounded rather than reported. A whale's lamport balance clears 2^53 comfortably.

So a decoder that returned `Number` would hand you a value that is *nearly* right, which is worse than an error, and that is why Kit's `getU64Decoder` gives you a `BigInt`.

That constraint is the origin of `u64` in the struct you are about to write. In Rust the width is in the type, `u64 + u64` never silently becomes a float, and the only way to lose a lamport is to let the arithmetic wrap — which is what the next answer is about.

---

## 8 — `-` on a `u64` is not safe, and `overflow-checks` is not the fix

```rust
pub fn withdraw(v: &mut VaultLike, amount: u64) {
    v.balance = v.balance - amount;
}
```

This compiles. That is the problem.

With `amount` greater than the balance, `u64` subtraction underflows. What happens next depends on the build profile, which is the worst possible thing for it to depend on:

- With overflow checks **on** — and the build server's release profile does set `overflow-checks = true` — the subtraction panics. In a program a panic aborts the instruction, and the caller gets a failure with no useful message.
- With overflow checks **off**, `5 - 7` wraps to 18,446,744,073,709,551,614. Your vault now reports a balance of eighteen quintillion lamports, and there is no error anywhere.

`overflow-checks = true` is a safety net for your own testing. It is **not** a substitute for checked arithmetic, because the correct behaviour is not "panic" — it is "return an error the caller can act on". That is `checked_sub`, which you wrote in lesson 4 as `sub_lamports` returning `Option<u64>`, and which becomes `VaultState::withdraw` returning `Result<()>` with `VaultError::InsufficientFunds` in module 4.

Three lessons from now the `None` arm gets a name. That is the whole arc: `Option` first because it is the smaller idea, `Result` second because it carries a reason, and `unwrap()` never, because it discards both.
