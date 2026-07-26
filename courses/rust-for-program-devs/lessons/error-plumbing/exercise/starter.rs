// vault_core.rs — errors that travel.
//
// THIS FILE DOES NOT COMPILE AS GIVEN. That is the exercise.
//
// Two functions have had their lines shuffled, and one of them carries two decoy
// lines that look plausible and are not. Every piece you need is already here —
// nothing has to be invented. Build first and let the compiler tell you what is
// out of order; then reorder, and delete the two decoys.

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

// ── LAYER 1: layout. SCRAMBLED — two correct lines, wrong order. ─────────────
//
// Note the return type: `core::result::Result<u64, LayoutError>`, spelled out,
// because bare `Result<u64>` in this file means Anchor's alias.
//
// Swap the two statements. Think about which name has to exist before the next
// line can use it. The tail expression at the bottom is correct — leave it.
fn read_amount(data: &[u8]) -> core::result::Result<u64, LayoutError> {
    let bytes: [u8; 8] = head.try_into().map_err(|_| LayoutError::TooShort)?;
    let head = data.get(..8).ok_or(LayoutError::TooShort)?;

    Ok(u64::from_le_bytes(bytes))
}

// ── LAYER 2: policy. GIVEN — read these two, they are your reference. ────────
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

// ── LAYER 3: the caller. SCRAMBLED, plus two decoys. ─────────────────────────
//
// Six statements below. FOUR are correct and in the wrong order. TWO are decoys:
// delete them, do not try to make them work. The tail expression is correct.
//
// Do it in two passes.
//
// Pass 1 — order. The chain you want, in words:
//   1. refuse anyone who is not the owner
//   2. read the amount out of the instruction data
//   3. check the amount against policy
//   4. subtract it from the balance, refusing to underflow
//
// Pass 2 — discrimination. Delete the two decoys. Both are compile errors rather
// than silent bugs, so the build server genuinely cannot pass this block while
// one is still here. Read each error before you delete the line; the decoy that
// calls `?` on an `Option` produces the single most useful message in this
// lesson, and the compiler suggests the fix in the error text itself.
//
// What pass 2 does NOT get you: the ordering from pass 1 is not checked by
// anything. Delete a correct line as though it were a third decoy, or run the
// owner check last, and this file still compiles. Two lines are enforced; the
// chain is yours.
pub fn withdraw_amount(
    data: &[u8],
    balance: u64,
    signer: &Pubkey,
    owner: &Pubkey,
) -> Result<u64> {
    let amount = check_amount(amount)?;
    let amount = read_amount(data);
    check_owner(signer, owner)?;
    let remaining = balance.checked_sub(amount)?;
    let amount = read_amount(data)?;
    let remaining = balance
        .checked_sub(amount)
        .ok_or(error!(VaultError::InsufficientFunds))?;

    Ok(remaining)
}

/// GIVEN: the deposit half. Two `?` in one expression, and `err!(E)` — which is
/// exactly `Err(error!(E))`, for when you are returning the error rather than
/// testing a condition.
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

// ── Why lesson 4 could not use `?` ───────────────────────────────────────────
// Lesson 4's helpers were `const fn`, and matched on `Option` by hand. Now you
// know why: `?` is not allowed in a `const fn`. Try it and the compiler says
// `error[E0015]: '?' is not allowed on 'Option<u64>' in constant functions`,
// because the `Try` trait is not yet const. `match` works in a `const fn`;
// `?` does not. That is the entire reason those two functions look different
// from everything in this file.
