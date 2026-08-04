// vault_core.rs — the error type, and Result instead of panic.
//
// Three subgoals. Do them in order; each one builds on the last.
//
// This file does NOT compile as given, and the code it ships is wrong as given —
// both on purpose. Each stub body keeps the wrong line visible (read it, it is
// the mistake being deleted) above a `compile_error!` that names the subgoal.
// Build it once to see all three at once, then start at SUBGOAL 2, deleting each
// `compile_error!` as you replace the line under it.

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ═════════════════════════════════════════════════════════════════════════════
// SUBGOAL 1 — the error enum. PRE-FILLED. Read it; do not change it yet.
// ═════════════════════════════════════════════════════════════════════════════
//
// `#[error_code]` takes an ordinary enum and generates:
//   - `#[derive(Debug, Clone, Copy)]` and `#[repr(u32)]` on it
//   - `fn name(&self) -> String`, returning the variant's name
//   - `impl From<VaultError> for u32`  ->  `variant as u32 + 6000`
//   - `impl Display for VaultError`, from your `#[msg]` strings
//   - `impl From<VaultError> for anchor_lang::error::Error`
//
// `error!(VaultError::Overflow)` uses the first three, not the last one: it
// builds an `AnchorError` out of `name()`, `into()` and `to_string()`. The
// `From<VaultError> for Error` impl is what `?` needs, and lesson 12 is where
// that matters.
//
// The 6000 is `anchor_lang::error::ERROR_CODE_OFFSET`. Codes below it belong to
// the runtime and to Anchor itself, so your first variant is 6000, not 0.
// `#[msg(...)]` is the string a client displays; it is carried into the IDL.
#[error_code]
pub enum VaultError {
    #[msg("Deposit would overflow the vault balance")]
    Overflow, // 6000
    #[msg("Withdrawal exceeds the vault balance")]
    InsufficientFunds, // 6001
    #[msg("Amount must be greater than zero")]
    ZeroAmount, // 6002
    #[msg("Signer is not the vault owner")]
    NotOwner, // 6003
    // SUBGOAL 3 adds a fifth variant right here. Not yet.
}

// ═════════════════════════════════════════════════════════════════════════════
// SUBGOAL 2 — turn lesson 4's `Option` into a `Result` that says why.
// ═════════════════════════════════════════════════════════════════════════════
//
// Lesson 4 gave you `Option<u64>`: Some means it fit, None means it did not.
// A program has to tell a client WHICH failure happened, so the answer becomes
// `Result<u64>` — that is `anchor_lang::Result<u64>`, i.e.
// `core::result::Result<u64, anchor_lang::error::Error>`.
//
// Use `match` on the Option. Do NOT use `?` — that is lesson 12.
// Do NOT use `.unwrap()` — see the note at the bottom of this file.

/// 2a. `match` on `balance.checked_add(amount)`:
///       Some(new_balance) => Ok(new_balance)
///       None              => Err(error!(VaultError::Overflow))
///
///     One thing NOT to add: a zero-amount check. Lesson 4 said rejecting a
///     zero deposit is a policy and belongs in the vault's `deposit` method,
///     and that still holds — `ZeroAmount` gets raised in lesson 13, not here.
///     `add_lamports(500, 0)` is `Ok(500)`. Arithmetic answers "did it fit?".
pub fn add_lamports(balance: u64, amount: u64) -> Result<u64> {
    // TODO (2a) — replace this. `balance + amount` is the bug this whole course
    // exists to delete: the build server compiles with `overflow-checks = true`,
    // so on a u64 wrap this panics instead of returning your named error.
    compile_error!("Subgoal 2a is not written yet: match on checked_add, Some -> Ok, None -> Err(error!(VaultError::Overflow)). Delete this line and replace the `balance + amount` line below it.");
    Ok(balance + amount)
}

/// 2b. Same shape, different reason: `checked_sub`, and `None` means the vault
///     does not hold that much -> `VaultError::InsufficientFunds`.
pub fn sub_lamports(balance: u64, amount: u64) -> Result<u64> {
    // TODO (2b) — replace this.
    compile_error!("Subgoal 2b is not written yet: same shape as 2a, but checked_sub, and None means VaultError::InsufficientFunds. Delete this line and replace the line below it.");
    Ok(balance - amount)
}

/// 2c. Now consume a `Result` by matching on it. A rejected deposit must leave
///     the balance exactly where it was.
///       Ok(new_balance) => new_balance
///       Err(_)          => balance
pub fn balance_after_deposit(balance: u64, amount: u64) -> u64 {
    // TODO (2c) — replace this with a `match` on `add_lamports(balance, amount)`.
    compile_error!("Subgoal 2c is not written yet: match on add_lamports(balance, amount) — Ok(new_balance) => new_balance, Err(_) => balance. Delete this line and the two below it.");
    let _ = amount;
    balance
}

// ═════════════════════════════════════════════════════════════════════════════
// SUBGOAL 3 — exhaustive matching, then add a fifth variant and watch.
// ═════════════════════════════════════════════════════════════════════════════
//
// Order matters here. Write 3a and 3b first, build them green, and only then
// do step 3c.

/// 3a. A stable code for logs and clients. Write an exhaustive `match` over
///     every variant of `VaultError`. NO `_ =>` arm — a catch-all is how you
///     throw exhaustiveness away, which is the one thing this lesson is about.
///     Suggested strings: "overflow", "insufficient_funds", "zero_amount",
///     "not_owner".
///
///     `#[error_code]` derived `Copy`, so this takes the error by value rather
///     than by reference. That is lesson 5's rule, applied.
pub fn error_slug(e: VaultError) -> &'static str {
    // TODO (3a) — replace this with the match.
    compile_error!("Subgoal 3a is not written yet: an exhaustive match over every VaultError variant, no `_ =>` arm. Delete this line and the two below it.");
    let _ = e;
    "todo"
}

/// 3b. A second exhaustive `match` over the same enum. Can the caller do
///     anything about this, or is retrying pointless?
///       Overflow => false, InsufficientFunds => true,
///       ZeroAmount => true, NotOwner => false
pub fn is_caller_fixable(e: VaultError) -> bool {
    // TODO (3b) — replace this with the match.
    compile_error!("Subgoal 3b is not written yet: a second exhaustive match over VaultError returning bool. Delete this line and the two below it.");
    let _ = e;
    false
}

// TODO (3c) — the point of the whole lesson.
//
// Add a fifth variant to VaultError, in SUBGOAL 1:
//
//     #[msg("Vault is frozen and cannot move lamports")]
//     VaultFrozen,
//
// Then BUILD, before you change anything else. Read the errors. There will be
// two of them, one per match site, each one naming the file and the line and
// the variant you did not handle: `error[E0004]: non-exhaustive patterns:
// `VaultError::VaultFrozen` not covered`.
//
// That is the compiler enumerating your unhandled cases. It is the JavaScript
// `switch` you forgot to update, caught before the code shipped instead of by a
// user. Now add the two arms — "vault_frozen", and false — and build green.

// ── DO NOT EDIT ──────────────────────────────────────────────────────────────
// Pins the five names above to their exact signatures. If one is missing, or its
// types drift, this stops compiling. It checks names and types — not what your
// bodies compute.
#[allow(dead_code)]
mod verify {
    use super::*;

    const _ADD: fn(u64, u64) -> Result<u64> = add_lamports;
    const _SUB: fn(u64, u64) -> Result<u64> = sub_lamports;
    const _AFTER: fn(u64, u64) -> u64 = balance_after_deposit;
    const _SLUG: fn(VaultError) -> &'static str = error_slug;
    const _FIXABLE: fn(VaultError) -> bool = is_caller_fixable;
}

// ── `unwrap()` ───────────────────────────────────────────────────────────────
// Do not use it here. The reason is on the lesson page, argued in full: a panic
// aborts the instruction with no error code and no `#[msg]` string, so every
// named variant above becomes indistinguishable noise.
