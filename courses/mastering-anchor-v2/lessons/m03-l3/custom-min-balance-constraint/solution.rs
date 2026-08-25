// Reference solution: the `check` hook enforces the floor, exactly as the V2
// `AccountConstraint` trait would when codegen dispatches `quarters::min_balance`.

pub trait AccountConstraint {
    /// The check hook Anchor V2 runs after the account is loaded.
    fn check(&self, balance: u64) -> Result<(), String>;
}

pub struct MinBalance {
    pub min: u64,
}

impl AccountConstraint for MinBalance {
    fn check(&self, balance: u64) -> Result<(), String> {
        if balance < self.min {
            return Err(format!(
                "quarters::min_balance violated: {} < {}",
                balance, self.min
            ));
        }
        Ok(())
    }
}

pub fn run_constraint(balance: u64, min: u64) -> bool {
    MinBalance { min }.check(balance).is_ok()
}
