// vault_core.rs — the error type, and Result instead of panic.

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ═════════════════════════════════════════════════════════════════════════════
// SUBGOAL 1 — the error enum. PRE-FILLED. Read it; do not change it.
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
    #[msg("Vault is frozen and cannot move lamports")]
    VaultFrozen, // 6004  <- added in SUBGOAL 3
}

// ═════════════════════════════════════════════════════════════════════════════
// SUBGOAL 2 — turn lesson 4's `Option` into a `Result` that says why.
// ═════════════════════════════════════════════════════════════════════════════

/// 2a. `checked_add` answers "did it fit?". A program has to answer "why not?".
///
/// Note what is NOT here: a zero-amount check. `add_lamports(500, 0)` is
/// `Ok(500)`. Rejecting a zero deposit is a policy, it belongs in the method
/// that owns the vault, and lesson 13's `VaultState::deposit` is where
/// `ZeroAmount` is finally raised — exactly as lesson 4 promised. The variant is
/// declared here because the enum is the program's complete failure set; the
/// enum being complete does not mean every variant is raised in this file.
pub fn add_lamports(balance: u64, amount: u64) -> Result<u64> {
    match balance.checked_add(amount) {
        Some(new_balance) => Ok(new_balance),
        None => Err(error!(VaultError::Overflow)),
    }
}

/// 2b. The same shape, and a different reason for the same shape of failure.
/// `sub_lamports(500, 500)` is `Ok(0)` — withdrawing the whole balance is legal.
pub fn sub_lamports(balance: u64, amount: u64) -> Result<u64> {
    match balance.checked_sub(amount) {
        Some(remaining) => Ok(remaining),
        None => Err(error!(VaultError::InsufficientFunds)),
    }
}

/// 2c. Consuming a `Result` by matching on it. A rejected deposit leaves the
/// balance where it was — no `.unwrap()`, no panic, no lost lamports.
pub fn balance_after_deposit(balance: u64, amount: u64) -> u64 {
    match add_lamports(balance, amount) {
        Ok(new_balance) => new_balance,
        Err(_) => balance,
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// SUBGOAL 3 — exhaustive matching, then add a fifth variant and watch.
// ═════════════════════════════════════════════════════════════════════════════

/// 3a. A stable code for logs and clients. Exhaustive `match`, no `_ =>` arm.
/// `#[error_code]` derived `Copy`, so this takes the error by value — lesson 5's
/// rule, applied.
pub fn error_slug(e: VaultError) -> &'static str {
    match e {
        VaultError::Overflow => "overflow",
        VaultError::InsufficientFunds => "insufficient_funds",
        VaultError::ZeroAmount => "zero_amount",
        VaultError::NotOwner => "not_owner",
        VaultError::VaultFrozen => "vault_frozen",
    }
}

/// 3b. A second exhaustive `match` over the same enum: can the caller do
/// anything about this, or is retrying pointless?
pub fn is_caller_fixable(e: VaultError) -> bool {
    match e {
        VaultError::Overflow => false,
        VaultError::InsufficientFunds => true,
        VaultError::ZeroAmount => true,
        VaultError::NotOwner => false,
        VaultError::VaultFrozen => false,
    }
}

// ── DO NOT EDIT ──────────────────────────────────────────────────────────────
// Pins the four names above to their exact signatures. If one is missing, or
// its types drift, this stops compiling. It checks names and types — not what
// your bodies compute.
#[allow(dead_code)]
mod verify {
    use super::*;

    const _ADD: fn(u64, u64) -> Result<u64> = add_lamports;
    const _SUB: fn(u64, u64) -> Result<u64> = sub_lamports;
    const _AFTER: fn(u64, u64) -> u64 = balance_after_deposit;
    const _SLUG: fn(VaultError) -> &'static str = error_slug;
    const _FIXABLE: fn(VaultError) -> bool = is_caller_fixable;
}
