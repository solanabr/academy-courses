//! Checked lamport math.
//!
//! Three signatures, no bodies. The spec is in the lesson text; the
//! acceptance criteria are the assertions at the bottom of this file, and
//! the compiler evaluates them while it builds.
//!
//! This file does not build as it ships. Build it anyway. The first build
//! reports all ten assertions failing with `error[E0080]: evaluation
//! panicked: not yet implemented` — that is `todo!()` being reached, and
//! the comment above each group of assertions names the cases.
//!
//! Once a function has a real body, a failing assertion prints itself
//! verbatim instead, for example:
//!
//!   error[E0080]: evaluation panicked:
//!     assertion failed: matches!(sub_lamports(5, 7), None)
//!
//! which names the exact input that is wrong.
//!
//! Constraints:
//!   * every function stays a `const fn` — the assertions can only run if
//!     the compiler can evaluate your code
//!   * no `?` — it is not allowed in a `const fn`. Use `match`.
//!   * no `unwrap()`, no `expect()`, no `panic!`, no `as`
//!   * no bare `+` or `-` on lamports

/// `Some(balance + amount)`, or `None` if that would exceed `u64::MAX`.
///
/// An `amount` of zero is valid and returns `Some(balance)`.
pub const fn add_lamports(balance: u64, amount: u64) -> Option<u64> {
    todo!()
}

/// `Some(balance - amount)`, or `None` if `amount` is greater than
/// `balance` — a `u64` has no negative values to land on.
///
/// Withdrawing the whole balance is valid and returns `Some(0)`.
pub const fn sub_lamports(balance: u64, amount: u64) -> Option<u64> {
    todo!()
}

/// Move `amount` from `from` to `to`, all or nothing.
///
/// `Some((new_from, new_to))` when both halves succeed. `None` when either
/// half is impossible — and in that case neither balance moved, so there
/// is nothing partial to report.
///
/// You cannot use `?` here. Look at the first result with `match` before
/// committing to the second.
pub const fn transfer_lamports(from: u64, to: u64, amount: u64) -> Option<(u64, u64)> {
    todo!()
}

// ════════════════════════════════════════════════════════════════════════
// DO NOT EDIT BELOW THIS LINE
//
// The first three bindings pin the signatures. The rest are assertions the
// compiler evaluates during the build, so a wrong answer is a red build
// rather than a silent pass. Ten specific inputs is not a test suite, and
// this only works because the functions are `const fn` — but it is a real
// behavioural check, which is more than the rest of this module gets.
// ════════════════════════════════════════════════════════════════════════

const _: fn(u64, u64) -> Option<u64> = add_lamports;
const _: fn(u64, u64) -> Option<u64> = sub_lamports;
const _: fn(u64, u64, u64) -> Option<(u64, u64)> = transfer_lamports;

// add: the ordinary case, the ceiling, and zero
const _: () = assert!(matches!(add_lamports(1, 2), Some(3)));
const _: () = assert!(matches!(add_lamports(u64::MAX, 1), None));
const _: () = assert!(matches!(add_lamports(u64::MAX, 0), Some(u64::MAX)));

// sub: the ordinary case, the floor, and draining to exactly zero
const _: () = assert!(matches!(sub_lamports(7, 5), Some(2)));
const _: () = assert!(matches!(sub_lamports(5, 7), None));
const _: () = assert!(matches!(sub_lamports(5, 5), Some(0)));

// transfer: both sides move, or neither does
const _: () = assert!(matches!(transfer_lamports(10, 0, 4), Some((6, 4))));
const _: () = assert!(matches!(transfer_lamports(10, 0, 0), Some((10, 0))));
const _: () = assert!(matches!(transfer_lamports(3, 0, 4), None));
const _: () = assert!(matches!(transfer_lamports(10, u64::MAX, 1), None));
