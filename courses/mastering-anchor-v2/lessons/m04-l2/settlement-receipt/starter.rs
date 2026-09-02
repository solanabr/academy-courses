// A settlement receipt: read the vault's balance on BOTH sides of the CPI.
//
// The types below are a hand-rolled stand-in for the V2 account API, cut down
// far enough to compile on plain rustc while keeping the one property this
// lesson is about: `cpi_handle_mut()` hands the CPI a real `&mut` borrow of a
// single account, `cpi_handle()` hands it a shared one, and the `CpiContext`
// keeps every borrow it carries alive until `transfer_checked` consumes it.
// Read a mutably-handled account inside that span and you get the same E0502
// the probe crate gave you.
//
// `settle` moves `amount` out of the vault and returns `(opening, closing)`:
//   * `opening` — the vault's balance as it stands BEFORE the transfer
//   * `closing` — the vault's balance as it stands AFTER the transfer
// Both must be read off `vault_ta`, not computed from `amount`.
//
// The starter does not compile: both reads sit in the span where the CPI holds
// a mutable handle on the vault. Their placement is the whole exercise, so
// leave everything above `fn settle` as it is.

struct TokenAccount {
    amount: u64,
}

impl TokenAccount {
    /// A typed read of the account's live data: needs a shared borrow.
    fn amount(&self) -> u64 {
        self.amount
    }

    /// Hand this account to a CPI that may WRITE it: a mutable borrow.
    fn cpi_handle_mut(&mut self) -> CpiHandleMut<'_> {
        CpiHandleMut(self)
    }
}

struct Mint {
    decimals: u8,
}

impl Mint {
    fn decimals(&self) -> u8 {
        self.decimals
    }

    /// Hand this account to a CPI that only READS it: a shared borrow.
    fn cpi_handle(&self) -> CpiHandle<'_> {
        CpiHandle(self)
    }
}

struct CpiHandleMut<'a>(&'a mut TokenAccount);
struct CpiHandle<'a>(&'a Mint);

struct TransferChecked<'a> {
    from: CpiHandleMut<'a>,
    mint: CpiHandle<'a>,
    to: CpiHandleMut<'a>,
}

struct CpiContext<'a> {
    accounts: TransferChecked<'a>,
}

impl<'a> CpiContext<'a> {
    fn new(accounts: TransferChecked<'a>) -> Self {
        CpiContext { accounts }
    }
}

mod token_interface {
    /// Taking the `CpiContext` by value is what ends every borrow it carries.
    pub fn transfer_checked(cpi: super::CpiContext<'_>, amount: u64, decimals: u8) {
        let accounts = cpi.accounts;
        // The "checked" half: the caller states the mint's decimals and the
        // callee proves them against the mint account it was handed.
        assert_eq!(accounts.mint.0.decimals, decimals, "mint decimals mismatch");
        accounts.from.0.amount = accounts.from.0.amount
            .checked_sub(amount)
            .expect("settlement exceeds the vault balance");
        accounts.to.0.amount = accounts.to.0.amount
            .checked_add(amount)
            .expect("recipient balance overflow");
    }
}

struct Accounts {
    mint: Mint,
    recipient_ta: TokenAccount,
    vault_ta: TokenAccount,
}

struct Context {
    accounts: Accounts,
}

fn settle(vault_start: u64, recipient_start: u64, amount: u64) -> (u64, u64) {
    let mut ctx = Context {
        accounts: Accounts {
            mint: Mint { decimals: 6 },
            recipient_ta: TokenAccount { amount: recipient_start },
            vault_ta: TokenAccount { amount: vault_start },
        },
    };

    let cpi = CpiContext::new(TransferChecked {
        from: ctx.accounts.vault_ta.cpi_handle_mut(),
        mint: ctx.accounts.mint.cpi_handle(),
        to: ctx.accounts.recipient_ta.cpi_handle_mut(),
    });

    // TODO: v1 muscle memory put both reads here, and here neither one is
    // legal. Place each so `opening` is the vault's balance before the
    // transfer and `closing` is its balance after — without touching the
    // `CpiContext` above or the `transfer_checked` call below.
    let opening = ctx.accounts.vault_ta.amount();
    let closing = ctx.accounts.vault_ta.amount();

    token_interface::transfer_checked(cpi, amount, ctx.accounts.mint.decimals());

    (opening, closing)
}
