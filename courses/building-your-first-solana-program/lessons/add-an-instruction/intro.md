# Add an Instruction

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · `borsh` resolves to **1.8.0** · edition 2021. Every compiler message quoted below was produced by compiling this lesson's starter on that toolchain.

You have named the four parts. Now add an instruction without being shown where each line goes.

Adding one is always the same three steps:

1. a `pub fn` inside `#[program]`;
2. an accounts struct with `#[derive(Accounts)]`, named exactly what the handler's `Context<T>` says;
3. inside that struct, the accounts the instruction needs — and the constraints they must satisfy.

Step 3 is module 2, and it is most of the rest of the course. This lesson is steps 1 and 2, on a handler that reads nothing, so that the mechanics are the only thing in front of you: `vault_info`, which logs the program's own id and returns.

## What adding an instruction does to your program's API

An instruction is selected by the 8-byte discriminator Anchor derives from its name, so adding a handler is a purely **additive** change: every existing caller keeps working, because their discriminators still resolve. Renaming or removing one is a **breaking** change for exactly the same reason. Program upgrades on Solana are in-place — the address does not change and clients do not get told — so this is the version-compatibility surface you actually have. `vault_info` is safe to add today and expensive to rename tomorrow.

## The exercise is a completion, not a blank page

Every line you need is already in the file, commented out and shuffled into two pools. Your job is discrimination and order, not invention:

- **Pool A**, inside `#[program]`: five lines, four of which make one complete handler. Rust is whitespace-insensitive, so indentation is not what you are being asked about — the sequence is. One line opens the block, one closes it, one is the body, one is the return value.
- **Pool B**, at the top level: four lines, two of which belong. This is where the exercise gets interesting.

Three of the nine lines belong nowhere. **Two of the three are caught by the compiler; one is not**, and knowing which one slips through is the actual content of this lesson.

### `#[account]` is not `#[derive(Accounts)]`

Pool B offers you both. They are one letter apart in conversation and unrelated in function:

| | Goes on | Means |
| --- | --- | --- |
| `#[account]` | a **state struct** | "these are the bytes stored inside a program-owned account" — generates the serialization and the 8-byte type discriminator |
| `#[derive(Accounts)]` | an **account list** | "these are the accounts one instruction may touch" — generates the deserialization and every validation check |

`VaultState` in this file has the first. `VaultInfo` needs the second. Choose `#[account]` for `VaultInfo` and the build fails on `the trait bound VaultInfo: Bumps is not satisfied` — `Bumps` is one of the traits the `Accounts` derive generates, and `Context<T>` requires it. That error message names the missing trait rather than the missing attribute, which is why it is worth meeting once deliberately.

### The line the compiler will not save you from

Pool B also offers a second error enum:

```rust
#[error_code] pub enum InfoError { #[msg("no vault at that address")] NoVault }
```

**A program should carry exactly one `#[error_code]` enum, and nothing enforces it.** You already have yours — `VaultError`, from Course 2, sitting at the bottom of this file.

Here is the part that matters. Paste that line in and **the build is green.** Not because our grader is lenient — a second enum is legal Anchor, and the Anchor CLI would accept it too. There is no rule being skipped here. What you get instead of an error is silent and worse.

`#[error_code]` generates `impl From<YourEnum> for u32` as `variant_index + ERROR_CODE_OFFSET`, and `ERROR_CODE_OFFSET` is `6000`. Two enums, neither given an explicit offset, both start numbering at 6000. So `VaultError::Overflow` and `InfoError::NoVault` are **both error code 6000**, and a client that catches 6000 cannot tell which failure it is looking at. Your error messages stop being diagnostic at the exact moment you need them.

Two correct moves, in order of preference:

1. Add a variant to `VaultError`. One namespace, monotonic numbering, no collision. This is what module 3 does when `withdraw` needs `InsufficientFunds`.
2. If you genuinely need a separate enum, `#[error_code(offset = 7000)]` moves its base. You will almost never need this, and reaching for it is usually a sign the first option was right.

Take the general lesson too: a green build here proves your file compiles against the real Anchor 1.x toolchain and nothing more. It does not prove the file is correct, and this lesson contains a live example of the gap.

## Your job

Assemble `vault_info` from pool A and `VaultInfo` from pool B. Leave the lines that belong nowhere commented out. The verification harness at the bottom of the file names both items by signature — build first, and the errors will tell you exactly what is still missing.
