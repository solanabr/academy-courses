// Reference solution: the `check` hook enforces the floor, exactly as the V2
// `AccountConstraint` trait would when codegen dispatches `quarters::min_balance`.
//
// The rule lives in `MinBalance::satisfies`, a `const fn`, so the verification
// harness at the bottom can evaluate it at compile time. A trait method cannot
// be `const` on stable Rust; splitting the condition out is what buys the
// compile-time proof, and it costs nothing at run time.

pub trait AccountConstraint {
    /// The check hook Anchor V2 runs after the account is loaded.
    fn check(&self, balance: u64) -> Result<(), String>;
}

pub struct MinBalance {
    pub min: u64,
}

impl MinBalance {
    /// The floor is inclusive: a balance equal to `self.min` satisfies it.
    const fn satisfies(&self, balance: u64) -> bool {
        balance >= self.min
    }
}

impl AccountConstraint for MinBalance {
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
