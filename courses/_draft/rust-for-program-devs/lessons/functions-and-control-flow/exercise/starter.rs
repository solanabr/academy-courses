//! Functions, expressions, and the missing `return`.
//!
//! Both bodies below are empty, so this file does not build.
//!
//! Every line you need is in the parts bin. Two of the seven lines are
//! decoys: one compiles and quietly does the wrong thing, one does not
//! compile at all. Copy the lines you want into the bodies, in the right
//! order. Do not write lines that are not in the bin.
//!
//! ┌─ PARTS BIN ─────────────────────────────────────────────────────────┐
//! │                                                                     │
//! │  Arms for `describe_state`. Four of these five belong. Arms are     │
//! │  tried top to bottom and the first match wins, so order is part of  │
//! │  the answer — and `match` must cover every possible `u64`.          │
//! │                                                                     │
//! │    (A)    _ => "whale",                                             │
//! │    (B)    1_000_000..=999_999_999 => "funded",                      │
//! │    (C)    0..=999_999 => "dust",                                    │
//! │    (D)    0 => "empty",                                             │
//! │    (E)    1..=999_999 => "dust",                                    │
//! │                                                                     │
//! │  Body for `min_balance`. One of these two belongs.                  │
//! │                                                                     │
//! │    (F)    if a < b { a; } else { b; }                               │
//! │    (G)    if a < b { a } else { b }                                 │
//! │                                                                     │
//! └─────────────────────────────────────────────────────────────────────┘

/// Classify a lamport balance. Every `u64` gets exactly one answer.
///
///   0                          -> "empty"
///   1 .. 999_999               -> "dust"
///   1_000_000 .. 999_999_999   -> "funded"
///   1_000_000_000 and above    -> "whale"
pub fn describe_state(balance: u64) -> &'static str {
    match balance {
        // Four arms from the bin go here, in the right order.
    }
}

/// The smaller of two balances.
pub fn min_balance(a: u64, b: u64) -> u64 {
    // One line from the bin goes here.
}

// ════════════════════════════════════════════════════════════════════════
// DO NOT EDIT BELOW THIS LINE
//
// These bindings pin both signatures so that renaming or deleting a
// function is a compile error rather than a way through. They check
// shapes, not behaviour. What checks behaviour here is `match`
// exhaustiveness — the compiler will not accept a set of arms that leaves
// a `u64` unmatched — plus the type of the `if`, which a stray semicolon
// changes to `()`.
// ════════════════════════════════════════════════════════════════════════

const _: fn(u64) -> &'static str = describe_state;
const _: fn(u64, u64) -> u64 = min_balance;
