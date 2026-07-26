// ---------------------------------------------------------------------------
// Lesson 6 — Borrowing: &T and &mut T.
//
// Three numbered subgoals below. Subgoal 1 is done for you as a reference;
// you write 2 and 3. Replace each `todo!()` — it is a placeholder that panics,
// and nothing that panics belongs in a finished program.
//
// As shipped this file does NOT build. Each unwritten body carries a
// `compile_error!` line whose message names the subgoal it belongs to, so the
// first build tells you exactly what is outstanding. Delete that line together
// with the `todo!()` under it when you write the body. This is deliberate:
// `todo!()` alone type-checks fine, so a file full of them would compile green
// and tell you nothing.
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
// YOUR CODE. Add `amount` to the vault's balance.
//
//   - `&mut VaultLike` is exclusive: for the duration of this call nothing else
//     may read or write this vault.
//   - Use `add_lamports`, not `+`. Never let a u64 wrap.
//   - `add_lamports` returns `Option<u64>`. Handle BOTH arms with `match` or
//     `if let`. On `Some(new_balance)`, store it. On `None`, leave the balance
//     exactly as it was — this function has no way to report a failure, which
//     is a known gap that module 3 closes with `Result`.
pub fn credit(v: &mut VaultLike, amount: u64) {
    compile_error!("Subgoal 2 of 3 is not written yet: give `credit` a body that adds `amount` via `add_lamports` and handles both arms. Delete this line and the `todo!()` below it.");
    todo!()
}

// ===== Subgoal 3 of 3 — read, then write, without overlapping =============
//
// YOUR CODE. Raise the balance up to `target` and return how much you added.
// If the balance already meets or exceeds `target`, add nothing and return 0.
//
// The order matters. The borrow checker will only tell you so if you write step 1
// the wrong way — see the note under step 4:
//
//   1. Get the current balance. Call `balance_of(v)` and bind the u64 it
//      returns. Do NOT write `let before = &v.balance;` — see the note below.
//   2. Compute the shortfall with `sub_lamports(target, current)`. On `None`
//      (target is below the current balance) the answer is 0.
//   3. Call `credit(v, shortfall)`. This needs the mutable borrow, and it can
//      only have it if step 1 is finished with the vault.
//   4. Return the shortfall.
//
// Note for step 1: `&v.balance` would hold a shared borrow, and step 3 needs an
// exclusive one. Whether that is an error depends on whether the shared borrow
// is still used after step 3 — a borrow ends at its LAST USE, not at the
// closing brace. Binding the u64 by value sidesteps the question entirely.
//
// Which means: once you follow that advice, nothing enforces the order. Writing
// step 3 before step 1 compiles clean. The order is correct because a transfer
// has to read before it commits, not because rustc made you.
pub fn top_up_to(v: &mut VaultLike, target: u64) -> u64 {
    compile_error!("Subgoal 3 of 3 is not written yet: read the balance, compute the shortfall, credit it, return it. Delete this line and the `todo!()` below it.");
    todo!()
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
