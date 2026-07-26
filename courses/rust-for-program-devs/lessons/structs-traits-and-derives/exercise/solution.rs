// vault_core.rs — the data model, step one.
//
// This file is COMPLETE and CORRECT. Build it unchanged first, then read it
// top to bottom. The experiment at the bottom is the only edit you make.

use anchor_lang::prelude::*;

// Defines `crate::ID`. `#[account]` below expands to an `Owner` impl that
// returns `crate::ID`, so deleting this line breaks the struct, not this line.
// Course 3 replaces this placeholder with your real program id.
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// The vault's on-chain state.
///
/// `#[account]` generates, at compile time:
///   - `#[derive(AnchorSerialize, AnchorDeserialize, Clone)]` on the struct
///   - `impl AccountSerialize`   -> `try_serialize`: writes discriminator, then fields
///   - `impl AccountDeserialize` -> `try_deserialize`: CHECKS discriminator, then reads
///   - `impl Discriminator`      -> `const DISCRIMINATOR: &'static [u8]`
///   - `impl Owner`              -> `fn owner() -> Pubkey { crate::ID }`
///
/// `#[derive(InitSpace)]` generates `impl Space`, i.e. `const INIT_SPACE: usize`,
/// summing the borsh size of every field: 32 + 8 + 1.
#[account]
#[derive(InitSpace)]
pub struct VaultState {
    /// 32 bytes. Who is allowed to withdraw.
    pub owner: Pubkey,
    /// 8 bytes, little-endian. Lamports the vault is holding.
    pub balance: u64,
    /// 1 byte. The canonical PDA bump, stored so it is never recomputed.
    pub bump: u8,
}

// An INHERENT impl. These methods belong to VaultState and to nothing else;
// no trait, no promise, no generic code can find them.
impl VaultState {
    pub fn new(owner: Pubkey, bump: u8) -> Self {
        Self {
            owner,
            balance: 0,
            bump,
        }
    }
}

// A TRAIT impl — the one we write by hand so `#[derive]` stops being magic.
//
// `#[derive(Default)]` would generate exactly this: every field set to its own
// type's default. Write it once, read it, then derive it forever after.
impl Default for VaultState {
    fn default() -> Self {
        Self {
            owner: Pubkey::default(), // the all-zero pubkey
            balance: 0,
            bump: 0,
        }
    }
}

/// The number you hand Anchor's `space =` constraint in Course 3.
///
/// `INIT_SPACE` is the FIELDS only. The 8 discriminator bytes are yours to add,
/// every single time. Getting this wrong under-allocates the account.
pub const VAULT_ACCOUNT_SIZE: usize = 8 + VaultState::INIT_SPACE;

// Compile-time proof, not tests. `INIT_SPACE` and `DISCRIMINATOR` are `const`,
// so the compiler evaluates these while it compiles the file. Break one and the
// build fails with the number that is actually true.
const _: () = assert!(VaultState::INIT_SPACE == 41);
const _: () = assert!(VaultState::DISCRIMINATOR.len() == 8);
const _: () = assert!(VAULT_ACCOUNT_SIZE == 49);

/// Uses the derived `AccountSerialize` impl: writes the 8 discriminator bytes,
/// then borsh-encodes the fields. Returns 49 for any `VaultState`, because a
/// struct of fixed-width fields has one size.
pub fn encoded_len(state: &VaultState) -> Result<usize> {
    let mut buf: Vec<u8> = Vec::new();
    state.try_serialize(&mut buf)?;
    Ok(buf.len())
}

// ── THE EXPERIMENT ────────────────────────────────────────────────────────────
// 1. Add `pub created_at: i64,` to VaultState above.
// 2. Build. The INIT_SPACE assertion fails and names the new number.
//    41 became 49; the account went from 49 bytes to 57.
// 3. Undo it (Reset Code) and build again so the file is green.
// ─────────────────────────────────────────────────────────────────────────────
