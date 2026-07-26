## What Course 3 adds, and why none of it is in your file

Your file has no `#[program]` module and no `#[derive(Accounts)]` struct. Read it cold and it looks like a program someone abandoned two-thirds of the way in. It is not. It is the two-thirds you can check by reading, deliberately separated from the third you cannot.

| Course 3 adds                                                                                                                                       | Why it is not here                                                                                                                                                       |
| --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `#[program] pub mod vault_program { … }` with `deposit` and `withdraw` **instruction handlers**                                                       | A handler is an entrypoint. An entrypoint needs a deployed program id and a real account list, and its body is two lines: validate, then call the method you already wrote. |
| `#[derive(Accounts)] pub struct Deposit<'info>` with `#[account(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump)]`                     | Constraints validate *accounts*. Nothing in your file is an account — that is precisely why it can be reasoned about without a runtime. This is also where your stored `bump` field gets spent, and where `NotOwner` is finally returned. |
| `init` with `space = 8 + VaultState::INIT_SPACE` — 49 bytes, and the rent-exemption number that follows from it                                        | Space and rent are properties of an account, not of a struct. Your struct only had to know its own size, and `#[derive(InitSpace)]` is how it knows.                       |
| A CPI to the System Program to move the actual lamports: `CpiContext::new(System::id(), Transfer { … })`                                              | Your `balance` field is bookkeeping. The lamports move in a separate call, and keeping the two apart is what lets a test check the arithmetic without a runtime.            |
| **LiteSVM tests that call `withdraw` and assert on the number that comes back**                                                                       | This is the one that matters. It is what catches a `checked_add` you meant to be `checked_sub` — the exact failure the build server cannot see, and the reason Course 2's grading stops at "it compiles". |
| A deploy, after which `declare_id!` stops being a placeholder and your program has an address on devnet                                              | You had nothing to deploy. You had something to be correct.                                                                                                               |

One currency note for when you start reading Course 3's material against the rest of the internet: **`CpiContext::new(System::id(), …)` is the current shape.** Every CPI tutorial written before Anchor 1.0 passes an account info instead — `CpiContext::new(ctx.accounts.system_program.to_account_info(), …)` — and it does not compile against the toolchain you have been building on. It is the single most common stale snippet in Solana material, and now you know why your copy-paste failed.

### Why the split is the right shape, not a teaching convenience

Look at what your file does *not* depend on: no accounts, no signers, no clock, no CPI, no runtime of any kind. That is what makes it the part a reviewer reads first and the part a test can exercise in microseconds. Real audit findings cluster at the seam between the two halves — a handler that forgets to compare `vault.owner` to the signer, a constraint that validates the wrong account — and you cannot see a seam until the two sides of it are separate things.

It also means Course 3 is enterable on its own. Anyone arriving without this course gets the reference `vault_core.rs`, because a course whose first lesson requires a file you do not have is a dead end, not a prerequisite. You are arriving with your own version, which is a better place to start from.

### What you can say you can do

Not "you can now write Solana programs" — you cannot yet, and one more course is why. What you can say is narrower and true: **you can model on-chain state in Rust, make its invariants unbreakable by construction, and return named errors instead of panicking.** That is the part of program development that no framework does for you.
