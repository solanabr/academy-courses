// ---------------------------------------------------------------------------
// Lesson 7 — Fix the Borrow Checker.
//
// THIS FILE DOES NOT COMPILE, AND THAT IS THE POINT. Four functions, four real
// borrow-checker errors. Build it FIRST and read only the first error.
//
// Every fix is already on the page. Under each broken function is a numbered
// list of candidate lines: some belong, some are decoys that fail in a
// different way. For each bug:
//
//   1. Read the error.
//   2. Choose the candidates that belong, and the order they go in.
//   3. Comment out the lines marked BUG, uncomment your choice, build again.
//
// Nothing above the first bug needs changing.
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

// ===== Bug 1 of 4 =========================================================
// Should log the vault, then return the balance it had.
//
// Candidates:
//   (1) let snapshot = vault.balance;
//   (2) consume_and_log(vault);
//   (3) snapshot
//   (4) let snapshot = &vault.balance;
//   (5) *snapshot
pub fn audit_then_read(vault: VaultLike) -> u64 {
    consume_and_log(vault); // BUG
    vault.balance // BUG
}

// ===== Bug 2 of 4 =========================================================
// Should raise the balance to at least `target` and return the balance as it
// was BEFORE the top-up.
//
// Candidates:
//   (1) let before = v.balance;
//   (2) let before = &v.balance;
//   (3) let before = &mut v.balance;
//   (4) before
//   (5) *before
pub fn top_up_to(v: &mut VaultLike, target: u64) -> u64 {
    let before = &v.balance; // BUG

    let shortfall = match sub_lamports(target, *before) {
        Some(gap) => gap,
        None => 0,
    };
    credit(v, shortfall);

    *before // BUG
}

// ===== Bug 3 of 4 =========================================================
// Should apply two credits to the same vault, in order.
//
// Candidates:
//   (1) credit(v, first);
//   (2) credit(v, second);
//   (3) let one = &mut *v;
//   (4) let two = &mut *v;
//   (5) credit(one, first);
pub fn credit_twice(v: &mut VaultLike, first: u64, second: u64) {
    let one = &mut *v; // BUG
    let two = &mut *v; // BUG

    credit(one, first);
    credit(two, second);
}

// ===== Bug 4 of 4 =========================================================
// Should return the first 8 bytes of `data` — an Anchor account's
// discriminator. One candidate line replaces both BUG lines.
//
// Candidates:
//   (1) &data[..8]
//   (2) &data.to_vec()[..8]
//   (3) let owned = data.to_vec();
//   (4) &owned[..8]
pub fn head(data: &[u8]) -> &[u8] {
    let owned = data.to_vec(); // BUG
    &owned[..8] // BUG
}

// ===== Given: callers, so a green build means the shapes still line up =====
//
// Also: `demo` is what makes `head` reachable and gives the other three a call
// site. Do not delete it. See the harness note at the bottom.
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
