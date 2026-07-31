# Anatomy of an Anchor Program

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · `borsh` resolves to **1.8.0** · edition 2021. Every compiler message quoted below was produced by compiling this lesson's starter on that toolchain.

The file you just compiled has four parts. Every Anchor program has the same four, conventionally in the same order, and once you can name them you can read any program on Solana — including the ones you did not write, which is most of the value.

The starter below is that program stripped to exactly those four parts, each behind a labeled banner. Two of them are complete. Two contain one worked example each, and your job is the second example in both.

## Part 1 — `declare_id!`: the address

```rust
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
```

A program is reached by address, the same way an account is. `declare_id!` bakes that address into the binary and generates a `pub const ID: Pubkey` you can use in code — the starter's `version` handler logs it.

Two things it is not. It is not a keypair: the program's *upgrade* authority is a separate key, held by whoever deployed. And it is not checked at compile time — put any valid base58 pubkey here and the build is green. The value only starts mattering at deploy, when a mismatch between the declared id and the address being deployed to produces `DeclaredProgramIdMismatch`. In module 4 you will replace this placeholder with an address that is genuinely yours.

## Part 2 — `#[program]`: the instruction handlers

```rust
#[program]
pub mod vault_program {
    use super::*;

    pub fn version(_ctx: Context<Version>) -> Result<()> {
        msg!("vault program, id {}", ID);
        Ok(())
    }
}
```

This attribute is what makes the crate a program rather than a library. It expands the module into a program entrypoint plus a dispatcher, and it turns every `pub fn` inside into a callable instruction.

The dispatch is by **discriminator**: Anchor hashes the handler's name and puts the first 8 bytes at the front of the instruction data, so a caller selects `version` by sending those 8 bytes. Two consequences you should file away. Renaming a handler changes its discriminator, which breaks every client already calling it — a rename is a breaking API change, not a cosmetic one. And the same mechanism is why the four-part order is a convention rather than a requirement: the macros do not care where in the file they appear.

The signature rules are absolute:

- the first parameter is always `Context<T>`;
- the return type is always `Result<()>` — Anchor's `Result`, which `use anchor_lang::prelude::*;` brings in;
- any further parameters are instruction *arguments*, deserialized from the instruction data. `deposit(ctx, amount: u64)` in module 3 is the first one you write.

`msg!` writes to the transaction log, which is what you read in an explorer and what stands in for a debugger on-chain. It costs compute units, so it is a tool for development, not a place to dump state.

One currency note, because this is where old code bites: in Anchor 1.x the handler return type is `Result<()>`. Material written before 0.24 uses `ProgramResult`, which no longer exists — if you paste a handler signature from an older tutorial you get `cannot find type ProgramResult in this scope`, and that error means *the snippet is old*, not that you mistyped.

## Part 3 — `#[derive(Accounts)]`: the account list

```rust
#[derive(Accounts)]
pub struct Version {}
```

Solana programs are stateless. Everything an instruction reads or writes arrives as an account in the transaction, and this struct is where you declare which ones and under what conditions. The derive generates the deserialization and every validation check, and it runs *before* your handler body — by the time you are inside `version`, the account list has already been proved to satisfy whatever the struct demanded.

The name must match the `Context<T>` generic exactly. That is the only thing tying a handler to its account list, which is why forgetting the struct produces `cannot find type Greet in this scope` rather than anything more helpful.

`Version {}` is empty, and empty is a claim rather than an omission: it says this instruction may touch no accounts at all. Anything not declared here is unreachable from the handler. Module 2 is where these structs get interesting — constraints, PDAs, `init`, signers — and where the vault finally becomes an account instead of a struct.

## Part 4 — `#[account]`: the state

```rust
#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}
```

Your Course 2 struct, unchanged. `#[account]` says "this type is the contents of an account owned by this program": it generates the borsh serialization and an 8-byte **discriminator** — a different one from the instruction discriminators, doing the same job one level down. It marks the account's *type*, so a program that is handed the wrong account gets a deserialization failure instead of misreading someone else's bytes.

Note what part 4 does not do. It describes a layout; it does not allocate anything. No account exists because of this struct. Creating one takes an `init` constraint in a part-3 struct, a payer, and rent — module 2, in that order.

### The fifth thing, which is not one of the four

Your Course 2 file also has `#[error_code] pub enum VaultError`. It is not in this starter, because it is not part of the anatomy — but it is worth one line now: **a program should carry exactly one `#[error_code]` enum.** Note "should". Anchor does not stop you adding a second, and that is precisely why the habit is worth forming early — every `#[error_code]` enum starts numbering at the same offset, so a second one silently collides with the first instead of failing loudly. You already have yours. When a later lesson needs a new failure mode, it becomes a variant of `VaultError`, not a new enum. The next lesson shows you the collision itself, which is not what most people expect.

## Your job

Add the second handler and the second account list, copying the pattern that is already in each part:

1. `greet`, inside `#[program]`, taking `_ctx: Context<Greet>`, returning `Result<()>`, logging something.
2. `Greet`, an empty struct carrying `#[derive(Accounts)]`.

The starter does not compile as shipped. Build it first and read the four errors. All four come from the verification harness at the bottom of the file, because the harness is the only thing that references what you have not written yet: three of them say `cannot find type Greet` or `cannot find struct … Greet`, and the fourth says `cannot find value greet in module vault_program`. That is the harness working as intended — it is the acceptance criteria written in Rust, and it is the only thing the build server can check.
