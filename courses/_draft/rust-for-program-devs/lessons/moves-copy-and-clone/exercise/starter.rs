// ---------------------------------------------------------------------------
// Lesson 5 — Moves, Copy, and Clone.
//
// WORKED EXAMPLE. This file is complete and it compiles. Nothing here is
// broken and nothing is missing.
//
// Build it once, then read the six numbered sections in order. Each one is
// three or four lines and demonstrates exactly one rule. Section 6 is the only
// place you touch: one line to uncomment, build, read, and comment back.
//
// Toolchain: anchor-lang 1.1.2 · borsh 1.5.7 declared / 1.8.0 resolved · edition 2021.
// ---------------------------------------------------------------------------

use anchor_lang::prelude::*;

/// The vault, as a plain Rust struct — field for field the account layout you
/// decoded in Course 1. In module 3 this grows an `#[account]` attribute and
/// becomes the real thing; for now it is an ordinary struct so that ownership
/// is the only thing in the room.
///
/// `Clone` is derived. `Copy` deliberately is NOT, even though all three fields
/// are `Copy` types. That single omission is what makes every assignment in
/// section 3 a move.
///
/// Note the shape change from lesson 2: `last_deposit_at` is gone and
/// `owner: Pubkey` has arrived. Lesson 2 needed somewhere to put an `i64`;
/// from here on the stand-in matches module 4's real `VaultState`
/// — `{ owner, balance, bump }` — because `owner` is the field that makes
/// the move rules interesting.
#[derive(Clone, Debug)]
pub struct VaultLike {
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}

/// A fresh vault to experiment on. `Pubkey::default()` is the all-zero address.
pub fn sample_vault() -> VaultLike {
    VaultLike {
        owner: Pubkey::default(),
        balance: 500_000,
        bump: 254,
    }
}

// === 1. `u64` is Copy: assignment duplicates the value =====================
// `snapshot` is a second, independent 8 bytes. `balance` is untouched and
// still readable, which is why lesson 4 never made you think about ownership.
pub fn copy_a_number() -> u64 {
    let balance: u64 = 1_000_000;
    let snapshot = balance;
    balance + snapshot // 2_000_000 — both names are live
}

// === 2. `Pubkey` is Copy too ==============================================
// 32 plain bytes, no heap, no destructor, and the type derives `Copy`. So you
// can read the field out of a borrowed vault as many times as you like.
pub fn copy_an_address(v: &VaultLike) -> (Pubkey, Pubkey) {
    let owner = v.owner;
    let owner_again = v.owner;
    (owner, owner_again)
}

// === 3. `VaultLike` is NOT Copy: assignment MOVES =========================
// Every field is `Copy` and the struct still moves, because `Copy` is a claim
// you make about a type, not something inferred from its fields. After the
// second line, the name `vault` is dead — the compiler will not let you read
// it, and there is no runtime check involved.
pub fn move_a_struct() -> u64 {
    let vault = sample_vault();
    let moved = vault;
    moved.balance
}

// === 4. A `Vec<u8>` can never be Copy =====================================
// Raw account data is a heap buffer with exactly one owner, because two owners
// would mean two frees. Passing it by value into `consume_bytes` moves it, and
// `data` is unusable on the next line.
pub fn move_account_data() -> usize {
    let data: Vec<u8> = vec![0u8; 49]; // 8 discriminator + 32 owner + 8 balance + 1 bump
    consume_bytes(data)
}

fn consume_bytes(bytes: Vec<u8>) -> usize {
    bytes.len()
}

// === 5. `.clone()` is the escape hatch, and it is legitimate ==============
// An explicit, visible request for a second independent value. Both names are
// live afterwards. The cost is real but it is written down at the call site,
// which is the entire design intent.
pub fn clone_a_struct() -> (u64, u64) {
    let vault = sample_vault();
    let mut backup = vault.clone();
    backup.balance = 0;
    (vault.balance, backup.balance) // (500_000, 0) — two separate vaults
}

// === 6. Borrowing: pass a reference and nothing moves at all ==============
// `&VaultLike` is a shared borrow. It grants read access and takes nothing, so
// the caller still owns the vault after the call returns — which is why you can
// call it twice. This signature is the first function you write in the next
// lesson, and it is a method on the real `VaultState` in module 4.
pub fn balance_of(v: &VaultLike) -> u64 {
    v.balance
}

pub fn read_twice() -> (u64, u64) {
    let vault = sample_vault();
    let a = balance_of(&vault);
    let b = balance_of(&vault);
    (a, b)
}

// === The experiment =======================================================
// Uncomment the line marked BREAK IT. Build. Read the error — note that it
// names the line where the move happened, not just the line that failed. Then
// comment it back and build again to confirm you are green.
//
// If you lose track of the file, Reset Code restores it.
pub fn experiment() -> u64 {
    let vault = sample_vault();
    let moved = vault;

    // let _oops = vault.balance; // <-- BREAK IT

    moved.balance
}
