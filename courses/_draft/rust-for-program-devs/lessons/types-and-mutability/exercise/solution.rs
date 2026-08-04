//! Types, mutability, and why the compiler argues — solved.

/// A stand-in for the vault account you write in module 4.
/// The widths are not decoration — each one is fixed by the runtime.
///
/// This stand-in is shaped for THIS lesson: it carries a timestamp so that
/// `i64` has somewhere to live. It is not the final shape. Lesson 5 drops
/// `last_deposit_at` and adds `owner: Pubkey`, and module 4's real
/// `VaultState` is `{ owner, balance, bump }`. Only `balance` and `bump`
/// survive all the way through.
pub struct VaultLike {
    /// Lamports. The runtime stores balances as u64, so you do too.
    pub balance: u64,
    /// A PDA bump seed. One byte, 0..=255.
    pub bump: u8,
    /// A UNIX timestamp from the cluster clock. Signed, 64-bit.
    pub last_deposit_at: i64,
}

// ── Subgoal 1 of 4 ──────────────────────────────────────────────────────

pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub fn one_sol() -> u64 {
    LAMPORTS_PER_SOL
}

// ── Subgoal 2 of 4 — `mut` on the reference, `mut` on the binding ───────

pub fn set_balance(vault: &mut VaultLike, lamports: u64) {
    vault.balance = lamports;
}

pub fn seed_bump() -> u8 {
    let mut bump: u8 = 255;
    bump -= 1;
    bump -= 1;
    bump
}

// ── Subgoal 3 of 4 — one explicit conversion, one corrected width ───────

pub fn bump_as_u64(bump: u8) -> u64 {
    u64::from(bump)
}

pub fn is_after(now: i64, last_deposit_at: i64) -> bool {
    now > last_deposit_at
}

// ── Subgoal 4 of 4 — slices are indexed by `usize` ──────────────────────

pub fn byte_at(data: &[u8], index: usize) -> u8 {
    data[index]
}

// ════════════════════════════════════════════════════════════════════════
// DO NOT EDIT BELOW THIS LINE
//
// The grader compiles this file and nothing more. These bindings exist so
// that a missing item, or one with the wrong type, is a compile error
// instead of a silent pass. They check shapes, not behaviour — nothing
// here can tell a correct body from a plausible wrong one.
// ════════════════════════════════════════════════════════════════════════

const _: u64 = LAMPORTS_PER_SOL;
const _: fn() -> u64 = one_sol;
const _: fn(&mut VaultLike, u64) = set_balance;
const _: fn() -> u8 = seed_bump;
const _: fn(u8) -> u64 = bump_as_u64;
const _: fn(i64, i64) -> bool = is_after;
const _: fn(&[u8], usize) -> u8 = byte_at;

#[allow(dead_code)]
fn verify_vault_fields(v: &VaultLike) -> (u64, u8, i64) {
    (v.balance, v.bump, v.last_deposit_at)
}
