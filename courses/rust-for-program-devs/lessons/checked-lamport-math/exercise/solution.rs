//! Checked lamport math — solved.

/// `Some(balance + amount)`, or `None` if that would exceed `u64::MAX`.
///
/// An `amount` of zero is valid and returns `Some(balance)`.
pub const fn add_lamports(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_add(amount)
}

/// `Some(balance - amount)`, or `None` if `amount` is greater than
/// `balance` — a `u64` has no negative values to land on.
///
/// Withdrawing the whole balance is valid and returns `Some(0)`.
pub const fn sub_lamports(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_sub(amount)
}

/// Move `amount` from `from` to `to`, all or nothing.
///
/// The debit is inspected before the credit is committed to. If the credit
/// cannot happen, the whole operation reports `None` and neither balance is
/// returned — which is why the debited value stays inside the `match` arm
/// instead of being written anywhere.
pub const fn transfer_lamports(from: u64, to: u64, amount: u64) -> Option<(u64, u64)> {
    match sub_lamports(from, amount) {
        None => None,
        Some(debited) => match add_lamports(to, amount) {
            None => None,
            Some(credited) => Some((debited, credited)),
        },
    }
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
