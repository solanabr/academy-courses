// ---------------------------------------------------------------------------
// Lesson 6 — Borrowing: &T and &mut T.
//
// Reference solution. All three subgoals filled in.
//
// Toolchain: anchor-lang 1.1.2 (declared) · borsh 1.5.7 declared / 1.8.0
// resolved · edition 2021.
// ---------------------------------------------------------------------------

use anchor_lang::prelude::*;

#[derive(Clone, Debug)]
pub struct VaultLike {
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}

pub fn sample_vault() -> VaultLike {
    VaultLike {
        owner: Pubkey::default(),
        balance: 500_000,
        bump: 254,
    }
}

// Carried forward from lesson 4, unchanged — same names, same parameter names,
// same signatures. `None` means the operation would have wrapped a u64, which is
// the failure the whole course exists to prevent.
pub const fn add_lamports(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_add(amount)
}

pub const fn sub_lamports(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_sub(amount)
}

// ===== Subgoal 1 of 3 — read through a shared borrow =======================
//
// DONE FOR YOU. Read it before you write anything else.
//
// `&VaultLike` is a shared borrow: read access to every field, no write access,
// and the caller keeps the vault. The return type is `u64` BY VALUE, not
// `&u64` — `u64` is `Copy`, so the caller walks away holding a number rather
// than a loan. That choice is what makes subgoal 3 possible.
pub fn balance_of(v: &VaultLike) -> u64 {
    v.balance
}

// ===== Subgoal 2 of 3 — write through a mutable borrow =====================
//
// `&mut` grants write access to every field, so `v.balance = …` is legal here
// and would not be legal through `&VaultLike`.
//
// `if let Some(x) = …` is `match` with one arm named and the other ignored. The
// ignored arm is the honest gap: on overflow the balance is left as it was and
// nobody is told. Module 3 turns that arm into `err!(VaultError::Overflow)`.
pub fn credit(v: &mut VaultLike, amount: u64) {
    if let Some(new_balance) = add_lamports(v.balance, amount) {
        v.balance = new_balance;
    }
}

// ===== Subgoal 3 of 3 — read, then write, without overlapping =============
//
// `balance_of(v)` takes `&*v` — a shared reborrow of the exclusive borrow you
// were handed — and returns a `u64` by value. The reborrow is over by the end
// of the line, so `credit(v, …)` gets its exclusive access with nothing to
// argue about.
//
// Had step 1 been `let current = &v.balance;`, the shared borrow would still be
// alive at step 3 only if something used it afterwards. Holding a `Copy` value
// instead of a reference removes the question from the code entirely, which is
// the reason `balance_of` returns `u64` and not `&u64`.
//
// And note what that costs: with no borrow held, nothing stops you writing the
// two statements the other way round. `credit(v, 1); let current = balance_of(v);`
// compiles perfectly well. The order below is correct because it is the order a
// transfer has to happen in, not because the compiler insisted.
pub fn top_up_to(v: &mut VaultLike, target: u64) -> u64 {
    let current = balance_of(v);

    let shortfall = match sub_lamports(target, current) {
        Some(gap) => gap,
        None => 0, // already at or above target
    };

    credit(v, shortfall);
    shortfall
}

// ===== Given: a caller, so you can see the shapes meet ====================
//
// Note `let mut` on the binding. Without it you could not produce `&mut vault`
// on the following line — you cannot lend out write access you do not have.
pub fn demo() -> (u64, u64) {
    let mut vault = sample_vault();

    let added = top_up_to(&mut vault, 750_000);
    let now = balance_of(&vault);

    (added, now) // expect (250_000, 750_000)
}

// ── DO NOT EDIT ──────────────────────────────────────────────────────────────
// Pins the three functions above to their exact signatures. If one is renamed,
// or a borrow turns into a by-value parameter, or a return type drifts, this
// stops compiling.
//
// It checks names and types — NOT what your bodies compute. It cannot tell
// `add_lamports` from `sub_lamports`, and it cannot see whether subgoal 3 reads
// before it writes either. The borrow checker catches exactly one wrong shape —
// holding `&v.balance` across `credit` and using it afterwards, which is E0502.
// Read the balance by value, as the spec says, and there is no borrow left to
// complain about: writing first and reading second compiles clean. The ordering
// is on you.
#[allow(dead_code)]
mod verify {
    use super::*;

    const _BALANCE_OF: fn(&VaultLike) -> u64 = balance_of;
    const _CREDIT: fn(&mut VaultLike, u64) = credit;
    const _TOP_UP_TO: fn(&mut VaultLike, u64) -> u64 = top_up_to;
}
