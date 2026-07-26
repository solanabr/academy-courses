# What You've Built

Congratulations! You've completed the full Solana developer lifecycle:

## Your Journey

1. **Write Code** — You wrote a counter program in Rust using the Anchor framework
2. **Compile** — The Superteam Academy Build Server compiled it to a Solana program binary
3. **Deploy** — You deployed it to Solana Devnet using the BPF Loader Upgradeable
4. **Interact** — You called instructions and watched on-chain state change in real time

## What You Learned

### Anchor Framework
- `#[program]` — defines your instruction handlers
- `#[derive(Accounts)]` — specifies account constraints
- `#[account]` — defines on-chain data structures
- `#[error_code]` — custom errors enforced by the runtime

### On-Chain Concepts
- **Accounts** are Solana's storage primitive (not key-value pairs)
- **Discriminators** identify account types (first 8 bytes)
- **Rent** prevents spam (accounts must maintain minimum balance)
- **Transactions** are batches of instructions signed by wallets

### Deployment Protocol
- Programs are uploaded in **~1000-byte chunks** via the BPF Loader
- A **buffer account** holds the binary during upload
- The **program account** is a pointer to the executable data
- Upgradeable programs can be updated by the upgrade authority

## Ship It: Your First Paid Solana Work

You now have something most applicants don't: a program you wrote, compiled, and deployed yourself, live on a public network. The next step isn't another course — it's putting that skill in front of teams that pay for it.

**Superteam Earn** lists bounties and projects from real Solana teams, judged on submissions, not resumes. This is where deployed-a-program skills convert into your first $500–$5,000 of paid Solana work:

1. **Browse open development work** — start with the [development bounties on Superteam Earn](https://superteam.fun/earn/category/development/). Look for small, scoped bounties with a defined deliverable: they are the fastest way to a first paid submission.
2. **Write about what you built** — [content bounties](https://superteam.fun/earn/category/content/) pay for technical writing. A walkthrough of the counter program you just deployed — what confused you, and how you worked through it — is exactly the material sponsors ask for.
3. **Have a bigger idea? Apply for a grant** — [Superteam grants](https://superteam.fun/earn/grants/) fund builders directly (avg $5.5k Brazil grants). A live Devnet program is the strongest artifact a first grant application can include.

Whatever you submit, link your deployed program. It is live on a public network — anyone can verify it, and that proof is worth more than any certificate screenshot.

## Keep Building

- **Install locally** — Set up the Anchor CLI on your machine for faster iteration
- **Solana Frontend Development** — Put a UI on your program: wallet integration, reading on-chain state, sending transactions from the browser

Your deployed program will continue to live on Devnet. You can interact with it anytime using the Solana CLI or any Anchor client.
