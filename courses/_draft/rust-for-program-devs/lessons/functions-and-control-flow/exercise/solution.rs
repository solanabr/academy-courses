//! Functions, expressions, and the missing `return` — solved.
//!
//! (C) was a decoy: `0..=999_999 => "dust"` covers 0, so placing it before
//! `0 => "empty"` makes that arm unreachable — a warning, not an error, so
//! the file would still have compiled while answering "dust" for an empty
//! vault. (E) is the same range with 0 excluded, which is the one that
//! belongs.
//!
//! (F) was the other decoy: `{ a; }` evaluates to `()`, so the `if` is
//! `()`, so the body does not match the `-> u64` return type. That is the
//! `help: consider removing this semicolon` error.

/// Classify a lamport balance. Every `u64` gets exactly one answer.
///
///   0                          -> "empty"
///   1 .. 999_999               -> "dust"
///   1_000_000 .. 999_999_999   -> "funded"
///   1_000_000_000 and above    -> "whale"
pub fn describe_state(balance: u64) -> &'static str {
    match balance {
        0 => "empty",
        1..=999_999 => "dust",
        1_000_000..=999_999_999 => "funded",
        _ => "whale",
    }
}

/// The smaller of two balances.
pub fn min_balance(a: u64, b: u64) -> u64 {
    if a < b { a } else { b }
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
