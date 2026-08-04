// ---------------------------------------------------------------------------
// Lesson 7 — Fix the Borrow Checker.
//
// Reference solution — all four bugs repaired, and the decoys explained.
//
// Toolchain: anchor-lang 1.1.2 · borsh 1.5.7 declared / 1.8.0 resolved · edition 2021.
// ---------------------------------------------------------------------------

use anchor_lang::prelude::*;

#[derive(Clone, Debug)]
pub struct VaultLike {
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}

pub const fn add_lamports(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_add(amount)
}

pub const fn sub_lamports(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_sub(amount)
}

/// From lesson 6, unchanged.
pub fn credit(v: &mut VaultLike, amount: u64) {
    if let Some(new_balance) = add_lamports(v.balance, amount) {
        v.balance = new_balance;
    }
}

/// Given. Takes the vault BY VALUE, so calling it hands the vault over for
/// good. You are not allowed to change this signature — plenty of real APIs
/// consume their argument and you do not get to rewrite them either.
pub fn consume_and_log(v: VaultLike) {
    msg!("vault {} holds {} lamports", v.owner, v.balance);
}

// ===== Bug 1 of 4 — E0382, use of moved value ==============================
// Answer: (1), (2), (3).
//
// `consume_and_log` insists on owning the vault, so the vault is gone after the
// call and no ordering saves you. Take the one number you need out FIRST. It is
// a `u64`, so `snapshot` is an independent copy that owes nothing to `vault`.
//
// Decoys (4) + (5): `&vault.balance` is a borrow, and you cannot move a value
// out while it is borrowed. That trades E0382 for E0505 — a different error,
// same underlying mistake of trying to keep a thread back to a value you gave
// away. `.clone()` would also work here (the compiler suggests it), at the cost
// of copying the whole struct to read 8 bytes of it.
pub fn audit_then_read(vault: VaultLike) -> u64 {
    let snapshot = vault.balance;
    consume_and_log(vault);
    snapshot
}

// ===== Bug 2 of 4 — E0502, mutable while shared ============================
// Answer: (1) and (4). Note `sub_lamports(target, before)` loses its `*` too,
// because `before` is now a `u64` rather than a `&u64`.
//
// The shared borrow was only fatal because the last line used it again, which
// kept it alive across `credit`. Copying the number out removes the borrow
// entirely, and a copied `u64` cannot conflict with anything.
//
// Decoy (3): `&mut v.balance` held across `credit(v, …)` is E0499 instead —
// two mutable borrows. Decoy (5): `*before` on a `u64` is E0614, you cannot
// dereference something that is not a reference.
pub fn top_up_to(v: &mut VaultLike, target: u64) -> u64 {
    let before = v.balance;

    let shortfall = match sub_lamports(target, before) {
        Some(gap) => gap,
        None => 0,
    };
    credit(v, shortfall);

    before
}

// ===== Bug 3 of 4 — E0499, two mutable borrows =============================
// Answer: (1) then (2). Both named borrows go away.
//
// Naming a `&mut` keeps it alive for as long as the name is used. Passing `v`
// straight into the call lets each call take its exclusive borrow, finish with
// it, and give it back before the next line begins. Nothing overlaps, so there
// is nothing to reject.
//
// Decoy (5) `credit(one, first);` only makes sense if you kept (3), which is
// the bug. Sequencing beats scoping: no blocks, no `drop()`, no `RefCell`.
pub fn credit_twice(v: &mut VaultLike, first: u64, second: u64) {
    credit(v, first);
    credit(v, second);
}

// ===== Bug 4 of 4 — E0515, returning a reference to a local ================
// Answer: (1), alone.
//
// `data.to_vec()` allocates a fresh buffer owned by this function, and that
// buffer is dropped on the way out. Borrow from `data` instead: it belongs to
// the caller, so it is still alive when the caller reads the result.
//
// Decoy (2) `&data.to_vec()[..8]` is the same bug written on one line — the
// temporary Vec is dropped at the end of the statement, so it is E0515 again.
//
// This version still panics if `data` is shorter than 8 bytes, because `[..8]`
// on a 3-byte slice is a runtime bounds failure. The next lesson replaces the
// signature with `Option<&[u8]>` and the panic with `None` — and makes you say
// out loud, in the signature, that the returned reference came from `data`.
pub fn head(data: &[u8]) -> &[u8] {
    &data[..8]
}

// ===== Given: callers, so a green build means the shapes still line up =====
pub fn demo() -> (u64, u64, usize) {
    let mut vault = VaultLike {
        owner: Pubkey::default(),
        balance: 500_000,
        bump: 254,
    };

    let before = top_up_to(&mut vault, 750_000);
    credit_twice(&mut vault, 1_000, 2_000);

    let account_data = vec![0u8; 49];
    let disc = head(&account_data);

    let logged = audit_then_read(vault);

    (before, logged, disc.len())
}

// ── DO NOT EDIT ──────────────────────────────────────────────────────────────
// Pins the five functions above to their exact signatures. Deleting a broken
// function is the one "fix" that would otherwise compile, so this block exists
// to close it: a missing name, a borrow that turned into a by-value parameter,
// or a drifted return type all stop the build.
//
// `head`'s pin is written `for<'a> fn(&'a [u8]) -> &'a [u8]`, which is the
// signature you get from elision — and be precise about what it buys: a `head`
// returning `&'static [u8]` also satisfies it, because a longer-lived reference
// is accepted wherever a shorter-lived one is wanted. What stops bug 4's wrong
// fix is E0515 in the body, not this line. Lesson 8 is where the annotation
// itself becomes the subject, and where a WRONG annotation does fail the pin.
//
// Names, types and borrows only. Nothing here checks what a body computes: a
// `credit_twice` that applies `second` before `first` compiles fine.
#[allow(dead_code)]
mod verify {
    use super::*;

    const _AUDIT_THEN_READ: fn(VaultLike) -> u64 = audit_then_read;
    const _TOP_UP_TO: fn(&mut VaultLike, u64) -> u64 = top_up_to;
    const _CREDIT_TWICE: fn(&mut VaultLike, u64, u64) = credit_twice;
    const _HEAD: for<'a> fn(&'a [u8]) -> &'a [u8] = head;
    const _CONSUME_AND_LOG: fn(VaultLike) = consume_and_log;
    const _DEMO: fn() -> (u64, u64, usize) = demo;
}
