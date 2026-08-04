//! Types, mutability, and why the compiler argues.
//!
//! Four subgoals, in order. Subgoal 1 is already done — read it first.
//! Subgoals 2, 3 and 4 do not compile as they ship. That is the exercise.
//!
//! Build it now, before changing anything. `rustc` reports every
//! independent error in one pass, so the first build hands you the whole
//! to-do list at once.
//!
//! Rules: no `unwrap()`, no `expect()`, no `as` casts.

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

// ── Subgoal 1 of 4 — DONE. Read it. ─────────────────────────────────────
// One SOL is 1_000_000_000 lamports. Declared as a `u64` constant, with
// underscores as digit separators — the compiler ignores them and your
// eyes do not. Note that the type is written out: a `const` never infers.

pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub fn one_sol() -> u64 {
    LAMPORTS_PER_SOL
}

// ── Subgoal 2 of 4 — `mut`, in both of the places it appears ────────────
// `set_balance` writes through a reference. `seed_bump` reassigns a local,
// walking downward from 255 the way Anchor's bump search does. Neither is
// currently allowed to write to what it has. Fix both.
//
// (255 - 1 - 1 cannot underflow, so plain subtraction is safe here.
//  Lesson 4 is about the case where you cannot know that in advance.)

pub fn set_balance(vault: &VaultLike, lamports: u64) {
    vault.balance = lamports;
}

pub fn seed_bump() -> u8 {
    let bump: u8 = 255;
    bump -= 1;
    bump -= 1;
    bump
}

// ── Subgoal 3 of 4 — widths do not convert themselves ───────────────────
// Rust performs no implicit numeric conversion, not even the lossless
// widening ones C and Java do silently.
//
// In `bump_as_u64`, make the conversion explicit — it cannot fail, so
// `From` is the right tool and no error handling is needed.
//
// In `is_after`, do NOT convert. Ask which width is correct: the cluster
// clock hands out `i64`, and `VaultLike::last_deposit_at` above is `i64`
// for that reason. Fix the parameter, not the comparison.

pub fn bump_as_u64(bump: u8) -> u64 {
    bump
}

pub fn is_after(now: i64, last_deposit_at: u64) -> bool {
    now > last_deposit_at
}

// ── Subgoal 4 of 4 — you index with `usize` ─────────────────────────────
// Anchor puts an 8-byte discriminator at the front of every account, so
// reading one byte out of raw account data is an everyday operation.
// Slices are indexed by `usize`, and that is a trait bound rather than a
// conversion — which is why the error code is E0277, not E0308.
//
// (`data[index]` panics if `index` is past the end. That is a real defect
//  and it is deliberate; lesson 8 replaces it with `data.get(index)`.)

pub fn byte_at(data: &[u8], index: u64) -> u8 {
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
