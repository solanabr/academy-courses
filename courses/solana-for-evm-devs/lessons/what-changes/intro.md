# What actually changed

You arrived able to build on the EVM. Nine lessons later, here is the whole
translation table in one place — and, more usefully, the parts that do not
translate.

## The table

| EVM | Solana |
| --- | --- |
| Contract storage | Separate accounts the program owns |
| `SSTORE`, paid once | Rent, a refundable deposit sized in bytes |
| `mapping(k => v)` | A PDA derived from seeds, computable off-chain |
| `msg.sender` | An account list with verified `is_signer` flags |
| `tx.origin` | No equivalent; signer status propagates explicitly |
| Gas | Compute units (work) plus priority fees (ordering), priced apart |
| Implicit storage access | An account list declared before execution |
| One ERC-20 contract per token | One shared token program; a mint is data |
| `approve` / `allowance` | A `delegate` field on the token account |
| External call | CPI, with signer propagation and explicit accounts |

## The three that are not translations

**Sizing is a design decision.** No EVM habit prepares you for declaring byte
counts up front. It surfaces early — usually as an account too small for what you
wanted to store — and it shapes schemas more than anything else in the model.

**Authority can come from derivation.** A PDA has no private key. A program signs
for it by producing seeds. There is no Solidity analogue, because in the EVM
"being the contract" is the only authority a contract has over its own storage.
Once this lands, a lot of Solana design stops looking arbitrary.

**The account list is a public interface.** Anyone can read what a transaction
touches without executing it. That is why parallel execution is possible, why
wallets can simulate meaningfully, and why some patterns you would reach for —
discovering state mid-execution — are simply unavailable.

## What did not change

Checked arithmetic still matters, and Rust's `checked_add` is your `SafeMath`
with better ergonomics. Access control is still the bug class that drains
protocols; it just moves from a missing `onlyOwner` to a missing "is this signer
the owner my state names". Upgrade authority is as sensitive as a proxy admin
key. Integrating unknown code is still where the risk lives.

The security instincts you already have are the ones worth keeping. It is the
storage instincts that need rebuilding.

## Where to go next

The natural sequel is a Rust course, then the account-validation layer Anchor
gives you — `#[derive(Accounts)]` is where every idea in module 1 becomes a
constraint the compiler checks for you. If you would rather stay on the client
side, the SDK path covers building transactions, deriving PDAs from TypeScript,
and simulating before you send.
