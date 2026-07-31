// vault_core.rs — errors that travel.

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ── GIVEN: the program's one error enum ──────────────────────────────────────
#[error_code]
pub enum VaultError {
    #[msg("Deposit would overflow the vault balance")]
    Overflow,
    #[msg("Withdrawal exceeds the vault balance")]
    InsufficientFunds,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Signer is not the vault owner")]
    NotOwner,
}

// ── GIVEN: a SECOND error type, deliberately not an #[error_code] ────────────
//
// One `#[error_code]` enum per program: a second one would also number from
// 6000 and collide on the wire. Layout failures are an internal concern, so
// `LayoutError` is a plain enum that never reaches a client under its own name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutError {
    TooShort,
}

// ── GIVEN: the conversion that makes `?` work across the seam ────────────────
//
// This is the payoff from lesson 10. `?` does not know anything about your error
// types; it calls `From::from` on the error it is carrying. Write the `From` impl
// and `?` starts crossing the boundary. Delete it and `?` stops compiling — the
// operator is not magic, it is one trait lookup.
impl From<LayoutError> for anchor_lang::error::Error {
    fn from(e: LayoutError) -> Self {
        match e {
            LayoutError::TooShort => error!(ErrorCode::InstructionDidNotDeserialize),
        }
    }
}

// ── LAYER 1: layout. Fails with LayoutError. ─────────────────────────────────
//
// Note the return type: `core::result::Result<u64, LayoutError>`, spelled out,
// because bare `Result<u64>` in this file means Anchor's alias.
fn read_amount(data: &[u8]) -> core::result::Result<u64, LayoutError> {
    let head = data.get(..8).ok_or(LayoutError::TooShort)?;
    let bytes: [u8; 8] = head.try_into().map_err(|_| LayoutError::TooShort)?;
    Ok(u64::from_le_bytes(bytes))
}

// ── LAYER 2: policy. Fails with VaultError. ──────────────────────────────────
//
// `require!(cond, E)` expands to `if !(cond) { return Err(error!(E)); }`.
// Nothing more. It reads better than the `if`, and that is its whole value.
fn check_amount(amount: u64) -> Result<u64> {
    require!(amount > 0, VaultError::ZeroAmount);
    Ok(amount)
}

// `require_eq!` would also compile here — `Pubkey` implements `ToString`. Use
// `require_keys_eq!` anyway: Anchor's own docs reserve `require_eq!` for
// non-pubkey values, and the pubkey version records the two operands as pubkeys
// and logs them with `Pubkey::log()` instead of allocating two strings. Same
// check, fewer compute units, and the log a client expects.
fn check_owner(signer: &Pubkey, owner: &Pubkey) -> Result<()> {
    require_keys_eq!(*signer, *owner, VaultError::NotOwner);
    Ok(())
}

// ── LAYER 3: the caller. Chains both layers with `?`. ────────────────────────
pub fn withdraw_amount(
    data: &[u8],
    balance: u64,
    signer: &Pubkey,
    owner: &Pubkey,
) -> Result<u64> {
    check_owner(signer, owner)?;
    let amount = read_amount(data)?;
    let amount = check_amount(amount)?;
    let remaining = balance
        .checked_sub(amount)
        .ok_or(error!(VaultError::InsufficientFunds))?;
    Ok(remaining)
}

/// The deposit half. Two `?` in one expression, and `err!(E)` — which is exactly
/// `Err(error!(E))`, for when you are returning the error rather than testing a
/// condition.
pub fn deposit_amount(data: &[u8], balance: u64) -> Result<u64> {
    let amount = check_amount(read_amount(data)?)?;
    match balance.checked_add(amount) {
        Some(new_balance) => Ok(new_balance),
        None => err!(VaultError::Overflow),
    }
}

/// GIVEN, to read rather than to call: `?` with the sugar taken off.
/// It is a `match`, an early `return`, and one call to `From::from`.
pub fn read_amount_the_long_way(data: &[u8]) -> Result<u64> {
    match read_amount(data) {
        Ok(amount) => Ok(amount),
        Err(layout) => Err(anchor_lang::error::Error::from(layout)),
    }
}

// ── DO NOT EDIT ──────────────────────────────────────────────────────────────
// Pins the chain's shape: the three names below must exist with exactly these
// types. Names and types only — nothing here checks what your bodies compute.
#[allow(dead_code)]
mod verify {
    use super::*;

    const _WITHDRAW: fn(&[u8], u64, &Pubkey, &Pubkey) -> Result<u64> = withdraw_amount;
    const _DEPOSIT: fn(&[u8], u64) -> Result<u64> = deposit_amount;
    const _LONG_WAY: fn(&[u8]) -> Result<u64> = read_amount_the_long_way;
}
