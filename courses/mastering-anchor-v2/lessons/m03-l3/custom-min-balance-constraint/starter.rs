// Anchor V2 lets you ship your OWN constraint namespace by implementing the
// `AccountConstraint` trait's hooks (init / check / update / exit). Downstream
// crates then write `#[account(quarters::min_balance = N)]` with no framework
// change. This exercise distills the `check` hook to plain Rust so you can prove
// the pattern without the full macro.
//
// The hook's CONDITION lives in `MinBalance::satisfies`, a `const fn`, for one
// reason: the compiler can evaluate a `const fn` while it builds, so the
// verification harness at the bottom of this file proves your logic at COMPILE
// time as well as against the test vectors. A trait method cannot be `const` on
// stable Rust, which is why the condition sits beside the trait impl instead of
// inside it. `check` still owns the hook's shape; `satisfies` owns its rule.
//
// TODO: implement `MinBalance::satisfies` so it is false when `balance` is below
// `self.min` and true otherwise. `check` and `run_constraint` are already wired.

pub trait AccountConstraint {
    /// The check hook Anchor V2 runs after the account is loaded.
    fn check(&self, balance: u64) -> Result<(), String>;
}

pub struct MinBalance {
    pub min: u64,
}

impl MinBalance {
    /// The floor itself, and the whole exercise. The floor is inclusive.
    const fn satisfies(&self, balance: u64) -> bool {
        // TODO: compare `balance` against `self.min`. Right now the constraint
        // never rejects anything, so every vault sails through.
        let _ = balance;
        true
    }
}

impl AccountConstraint for MinBalance {
    /// Given: the hook forwards to the condition above and turns a `false` into
    /// the error codegen would surface from the constraint.
    fn check(&self, balance: u64) -> Result<(), String> {
        if self.satisfies(balance) {
            Ok(())
        } else {
            Err(format!(
                "quarters::min_balance violated: {} < {}",
                balance, self.min
            ))
        }
    }
}

fn run_constraint(balance: u64, min: u64) -> bool {
    MinBalance { min }.check(balance).is_ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
// Compile-time assertions. Because `satisfies` is a `const fn`, the compiler
// evaluates these while building: an unfixed hook does not compile at all, and
// the message names the case it got wrong. They pin the two things the test
// vectors alone cannot force — that the rule reads `self.min` rather than a
// hardcoded constant, and that the floor is inclusive rather than off by one.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::MinBalance;

    const _: () = assert!(
        MinBalance { min: 100 }.satisfies(500),
        "a balance above the floor must pass"
    );
    const _: () = assert!(
        MinBalance { min: 100 }.satisfies(100),
        "the floor is inclusive: a balance exactly at the floor must pass"
    );
    const _: () = assert!(
        !MinBalance { min: 100 }.satisfies(99),
        "one lamport below the floor must be rejected"
    );
    const _: () = assert!(
        MinBalance { min: 0 }.satisfies(0),
        "a zero floor admits an empty account"
    );
}
