# Withdraw With a PDA Signer

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · edition 2021 · System Program and rent behaviour read out of Agave `solana-system-program` 3.1.14 / 4.1.2 and `solana-svm-rent-collector`. Every compiler message quoted below was produced by that toolchain — either by compiling this lesson's file or, where a counter-example is shown, by compiling that counter-example; every runtime message was read from that Agave source, not from memory.

This lesson is named after the thing everyone reaches for. By the end you will know why the vault does not, in fact, sign its own withdrawal — and exactly where that signature *is* required, because it is a mechanism you will need constantly.

Deposit was easy: the user signed the transaction, and that signature was still valid when the System Program looked at it. Withdrawal has no such luck. The lamports are leaving the *vault*, and the vault has no private key. It is an address that was chosen precisely because no key can produce it.

## The shape you would write

Anchor has an answer for exactly this, and it is real:

```rust
let user_key = ctx.accounts.user.key();
let signer_seeds: &[&[&[u8]]] = &[&[b"vault", user_key.as_ref(), &[ctx.accounts.vault.bump]]];

transfer(
    CpiContext::new_with_signer(
        System::id(),
        Transfer { from: vault_info, to: user_info },
        signer_seeds,
    ),
    amount,
)?;
```

`new_with_signer` takes the seeds and, at the moment of the nested call, the runtime re-derives an address from them **under your program id**. If the result matches the account you are claiming signed, the signature is accepted. No key exists, and none is needed: the derivation *is* the proof, and only your program can produce it because your program id is one of the inputs.

That is one of the most important primitives on Solana. Learn the shape.

Note the `let user_key = ...` on the first line. `key()` returns a `Pubkey` **by value** and `as_ref()` only borrows it, so the seeds array holds a reference to a temporary. In the exact form above you get away with it even without the binding: the initializer starts with `&`, which triggers temporary lifetime extension, and the `Pubkey` is promoted to live as long as the enclosing block. It compiles either way.

The moment you split the seeds out into their own array, the extension no longer applies and the temporary dies at the semicolon:

```rust
// Does NOT compile — no leading `&`, so no lifetime extension.
let seeds: [&[u8]; 3] = [b"vault", ctx.accounts.user.key().as_ref(), &[ctx.accounts.vault.bump]];
```

```
error[E0716]: temporary value dropped while borrowed
  |
  |     let seeds: [&[u8]; 3] = [b"vault", ctx.accounts.user.key().as_ref(), ...];
  |                                        ^^^^^^^^^^^^^^^^^^^^^^^ creates a temporary value
  |                                        which is freed while still in use
```

That refactor is extremely common — two-line seeds are easier to read — so bind the key to a named variable regardless. It costs one line and it is correct in both shapes.

## Why it does not work here

Now the part almost no tutorial tells you. From Agave's System Program, `transfer_verified`:

```rust
let mut from = instruction_context.try_borrow_instruction_account(from_account_index)?;
if !from.get_data().is_empty() {
    ic_msg!(invoke_context, "Transfer: `from` must not carry data");
    return Err(InstructionError::InvalidArgument);
}
```

The System Program will not send lamports **out of** any account that carries data. Your vault carries 49 bytes of `VaultState`. There is a second, independent reason as well: only the program that *owns* an account may debit it, and your vault is owned by your program, not by the System Program.

So the code above compiles, passes this course's grading, deploys cleanly, and fails the first time a learner calls it:

```
Transfer: `from` must not carry data
```

Authorisation was never the obstacle. Ownership was. The seeds were solving a problem you did not have.

> Read that sequence again, because it is the most expensive shape of bug in this ecosystem: **compiles, deploys, then fails.** Our own course catalogue specified the broken version of this instruction until it was compiled and checked against the runtime source. Anything you read about Solana — including this — is a claim until you have watched it run.

## What actually moves lamports out

Your program owns the vault. Owners may debit their own accounts directly, with no CPI and nothing to sign:

```rust
**vault_info.try_borrow_mut_lamports()? = remaining;
**user_info.try_borrow_mut_lamports()? = credited;
```

Two mutable borrows of the runtime's lamport fields, and the runtime reconciles the totals when the instruction ends. If the sums do not balance, the transaction fails — you cannot create lamports this way, only move them between accounts already in the instruction.

This is not a workaround. It is what Anchor's own `close = destination` constraint does internally, and what every escrow that holds native SOL in a stateful PDA does. `Withdraw` therefore needs no `system_program` in its accounts struct at all: there is no CPI to make.

## The floor you cannot go below

One more rule, and it is the one people discover in production. Rent collection is gone, but rent *exemption* is enforced at the end of every transaction. From the runtime's rent check, a transition to `RentPaying` is allowed only from `RentPaying`:

```rust
match post_rent_state {
    RentState::Uninitialized | RentState::RentExempt => true,
    RentState::RentPaying { .. } => match pre_rent_state {
        RentState::Uninitialized | RentState::RentExempt => false,
        // ...
    }
}
```

Your vault starts rent-exempt. Take it below the minimum balance for its 49 bytes and the transition is refused with `TransactionError::InsufficientFundsForRent` — the whole transaction, not just your instruction.

Which means a vault holding exactly its rent-exempt minimum plus 1 SOL cannot pay out 1 SOL *and* stay alive at the same lamport count. You have to know the floor and check against it:

```rust
let rent_floor = Rent::get()?.minimum_balance(vault_info.data_len());
```

`data_len()` reads the account's real allocated size, so this number stays correct even if the struct grows later. Do not hardcode 49.

## The error, and the one-enum question

The guard returns `VaultError::InsufficientFunds` — the variant you already wrote in Course 2. Do not add a new error enum for this.

That advice is usually given as "Anchor 1.0 allows only one `#[error_code]` per program." **That is not true, and it matters that you know why the real reason is worse.** Two `#[error_code]` enums in one program compile without complaint — this was verified on the same toolchain that grades you. What happens instead is silent:

```rust
impl From<VaultError> for u32 {
    fn from(e: VaultError) -> u32 { e as u32 + ERROR_CODE_OFFSET }   // 6000
}
```

Every `#[error_code]` enum starts counting at the same `ERROR_CODE_OFFSET` of 6000 unless you pass an explicit offset. So a second enum's first variant is *also* error code 6000, and a client that receives 6000 has no way to tell `VaultError::Overflow` from `SomeOtherError::Whatever`. You have not gained a type; you have gained a collision, and the compiler is happy about it.

One enum, four variants, already written, already carrying `InsufficientFunds`. Use it.

## The exercise

Completion, not a blank page. The accounts struct is given, the two `AccountInfo` bindings are given, and ten candidate lines sit in a comment block at the top. Eight belong in the file. Two do not, and one of those two is the CPI you just read about — present on purpose, because reaching for it is the correct instinct and rejecting it is the lesson.
