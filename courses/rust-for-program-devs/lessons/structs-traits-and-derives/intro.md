# Structs, Traits, and What `#[derive]` Really Does

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (latest on crates.io, published 2026-06-26) · `borsh 1.8.0` (what the build server's lockfile resolves) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · edition 2021. Every number in the byte table below was read out of `anchor-derive-space 1.1.2`'s source on that date, and every compiler message quoted was produced by compiling the code in this lesson.

In lesson 8 you wrote `discriminator(data: &[u8]) -> Option<&[u8]>` — a borrowed view over the first eight bytes of an account. It worked, and you still do not know who put those eight bytes there or what decides they are eight.

This lesson is that answer. It is also the first file in this course that Course 3 will actually deploy.

## The struct is the layout

```rust
#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}
```

In JavaScript a `class Vault { owner; balance; bump }` says nothing about memory. Field order is irrelevant, adding a field costs nothing, and there is no such thing as "how many bytes is a Vault."

On Solana that question has one answer and you have to know it before the account exists, because you pay rent for the bytes and you allocate them once. So a Rust struct is not a bag of names. It is a **layout**: this many bytes, in this order.

Borsh — the serialization format Anchor uses — encodes each field back to back, little-endian, with no padding and no field names on the wire. The sizes, straight out of `anchor-derive-space 1.1.2`:

| Rust type | Bytes |
| --- | --- |
| `bool`, `u8`, `i8` | 1 |
| `u16`, `i16` | 2 |
| `u32`, `i32`, `f32` | 4 |
| `u64`, `i64`, `f64` | 8 |
| `u128`, `i128` | 16 |
| `Pubkey` | 32 |
| `[T; N]` | `N` × size of `T` |
| `Option<T>` | 1 (present/absent tag) + size of `T` |
| `String` | 4 (length prefix) + `max_len` |
| `Vec<T>` | 4 (length prefix) + size of `T` × `max_len` |

So `VaultState` is `32 + 8 + 1 = 41` bytes of data. The account it lives in is **49** bytes, because eight more go in front. Those are lesson 8's eight bytes.

Two traps in that table, both of which produce wrong-sized accounts in real programs:

- **The length prefix is 4 bytes, not 1, and not zero.** A `String` capped at 16 characters costs `4 + 16 = 20`, not 16. Forgetting the prefix is the classic under-allocation.
- **`Option<T>` costs the tag even when it is `None`.** Borsh writes a fixed layout; there is no "absent field" on the wire.

## A trait is a promise about a type. `impl` is where you keep it

A trait is a named set of function signatures. A type that has those functions "implements" the trait.

If you know TypeScript interfaces, the shape is familiar and one detail is not: **a Rust trait implementation lives outside the type.**

```ts
// TypeScript: the promise is declared inside the type
class Vault implements Serializable {
  serialize() { /* ... */ }
}
```

```rust
// Rust: the struct is one block, each promise is another
pub struct VaultState { /* fields only */ }

impl Default for VaultState { /* one promise */ }
impl Serializable for VaultState { /* another */ }
```

That separation is not cosmetic. It means you can implement a trait for a type long after it was defined, in a different block, and — as long as either the trait or the type is yours — in a different crate entirely. It also means an `impl` block is just *some code that sits next to your struct*.

Hold on to that sentence, because it is the whole explanation of `#[derive]`.

Two kinds of `impl` show up in the exercise, and they are different things:

```rust
impl VaultState {              // inherent impl: methods that are just VaultState's own
    pub fn new(/* ... */) -> Self { /* ... */ }
}

impl Default for VaultState {  // trait impl: keeping a promise the language defined
    fn default() -> Self { /* ... */ }
}
```

`VaultState::new` exists because you wrote it and nothing else in the language knows about it. `VaultState::default()` exists because `Default` is a standard trait, which is why generic code — including a lot of Anchor — can call it on any type that implements it without knowing your type at all.

## What `#[derive]` really does

`#[derive(Default)]` is a **macro**. At compile time it reads your struct's fields and writes out an `impl Default for VaultState` block, then hands that block to the compiler along with your file. That is the entire mechanism.

There is no runtime reflection here. Nothing inspects your struct while the program runs. JavaScript decorators and `Object.keys()` have no equivalent in the compiled binary — by the time your program is bytes on-chain, the macro has already run, the field names are gone, and only the generated code remains.

Which means the honest way to stop `#[derive]` being magic is to write, once, by hand, the impl it would have generated. That is what the exercise does with `Default`. Afterwards, derive it — you will not learn anything the second time.

## What `#[account]` hands you

`#[account]` is the big one. On `VaultState` it generates:

| What it generates | Why you care |
| --- | --- |
| `#[derive(AnchorSerialize, AnchorDeserialize, Clone)]` on your struct | borsh encode/decode, plus `Clone` |
| `impl AccountSerialize` | `try_serialize` — writes the discriminator, then the fields |
| `impl AccountDeserialize` | `try_deserialize` — **checks** the discriminator, then reads the fields |
| `impl Discriminator` | `const DISCRIMINATOR: &'static [u8]` |
| `impl Owner` | `fn owner() -> Pubkey { crate::ID }` |

The discriminator is the first eight bytes of `sha256("account:VaultState")`. For this struct that is `[228, 196, 82, 165, 98, 210, 235, 152]`, and you can assert on it at compile time because it is a `const`. Rename the struct and every byte changes.

Its job is type safety across a trust boundary. An account is a bag of bytes; nothing about the bytes says what they are. So `try_deserialize` reads the first eight, compares them against `VaultState::DISCRIMINATOR`, and if they disagree it returns `Err(AccountDiscriminatorMismatch)` **without attempting to decode the rest**. Hand a program somebody else's account and you get a named error, not a garbage `VaultState` with a plausible-looking balance.

One currency note: in Anchor 1.x `DISCRIMINATOR` is a `&'static [u8]`. Older tutorials treat it as `[u8; 8]` and index or destructure it as an array. That does not compile against 1.1.2.

### `declare_id!` is not decoration

Look at the `Owner` impl again: it returns `crate::ID`. `declare_id!` is what defines `ID`. Delete the `declare_id!` line and the compiler says:

```
error[E0425]: cannot find value `ID` in the crate root
 --> src/lib.rs:3:1
  |
3 | #[account]
  | ^^^^^^^^^^ not found in the crate root
```

Read that carefully, because the arrow points at `#[account]` and the problem is somewhere else entirely. `#[account]` is fine. The program id is missing. This is the single most confusing error in the lesson and it is worth meeting it here, deliberately, rather than in module 4 when you have forty lines of your own code to suspect.

The id in the starter — `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS` — is a placeholder. Course 3 replaces it with the id of the program you actually deploy.

## `#[derive(InitSpace)]` and the off-by-eight

`InitSpace` generates `impl Space for VaultState { const INIT_SPACE: usize = 32 + 8 + 1; }` — the arithmetic from the table above, done by the macro, at compile time. `INIT_SPACE` is a `const`, so you can assert on it and the assertion is checked while the file compiles.

**`INIT_SPACE` does not include the discriminator.** It is 41, not 49. Anchor's allocation looks like this:

```rust
space = 8 + VaultState::INIT_SPACE
```

That `8 +` is on you every single time, and forgetting it is the most common allocation bug in Anchor programs: the account is eight bytes too small, the write runs off the end, and the failure surfaces at serialization time as something that reads nothing like "you allocated wrong."

`InitSpace` also refuses to guess about variable-length fields. Add a `String` with no cap and the build stops:

```
error: Expected max_len attribute.
```

Which is the correct behaviour — a `String` has no size until you name one, and an account has to have a size.

## What is deliberately not in this file

There is no `#[program]` and no `#[derive(Accounts)]`. That is not an omission you should fix.

This course produces `vault_core.rs`: the data model and the arithmetic, compiling on its own. Course 3 is what wraps instruction handlers and account-validation structs around it. Keeping them apart is the reason your file is readable, and it is the reason you can be confident the struct is right before anything is deployed.

## The exercise

The starter is complete and correct. Build it first, unchanged, and read it.

Then do one experiment, in this order:

1. Add `pub created_at: i64,` to `VaultState`.
2. Build. The `INIT_SPACE` assertion fails, and the failure names the new number — 41 became 49, and the account went from 49 bytes to 57. The struct *is* the layout, and you just watched the layout move.
3. Undo it (Reset Code is the fast way) and build again so the file is green.

One honest note on grading: the build server checks that your file **compiles** against the real Anchor 1.1.2 toolchain. It does not run your code, and nothing here is unit-tested. The compile-time assertions in the file are doing the real verification work, which is exactly why they are written as `const _: () = assert!(...)` rather than as tests.
