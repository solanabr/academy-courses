// Anchor V2 lets you ship your OWN constraint namespace by implementing the
// `AccountConstraint` trait's hooks (init / check / update / exit). Downstream
// crates then write `#[account(quarters::min_balance = N)]` with no framework
// change. This exercise distills the `check` hook to plain Rust so you can prove
// the pattern without the full macro.
//
// TODO: implement `MinBalance::check` so it returns Err when `balance` is below
// `self.min`, and Ok otherwise. `run_constraint` already wires the constraint up.

pub trait AccountConstraint {
    /// The check hook Anchor V2 runs after the account is loaded.
    fn check(&self, balance: u64) -> Result<(), String>;
}

pub struct MinBalance {
    pub min: u64,
}

impl AccountConstraint for MinBalance {
    fn check(&self, _balance: u64) -> Result<(), String> {
        // TODO: enforce the floor. Right now the constraint never rejects anything.
        Ok(())
    }
}

pub fn run_constraint(balance: u64, min: u64) -> bool {
    MinBalance { min }.check(balance).is_ok()
}
