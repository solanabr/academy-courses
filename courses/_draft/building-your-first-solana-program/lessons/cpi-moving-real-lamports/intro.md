# CPI: Moving Real Lamports

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · edition 2021 · System Program behaviour read out of Agave `solana-system-program` 3.1.14 and 4.1.2. Every compiler message and every runtime log quoted below was produced by compiling this lesson's file, or read verbatim from that Agave source.

Your vault has a `balance` field. So far that field is a claim: it says how much the vault holds, and nothing enforces it. Nobody has moved a lamport.

This lesson is the one where the claim becomes true. You will read a finished `deposit` — not write it — and by the end you should be able to say exactly which program moved the money, who authorised it, and why the code looks the way it does rather than the way every tutorial written before Anchor 1.0 says it should.

## Your program cannot move lamports

Not the user's, anyway. An account's lamports may only be **debited by the program that owns the account**. The user's wallet is owned by the System Program, so the System Program is the only thing on the chain that can take lamports out of it.

That is not a limitation to work around. It is the whole security model: if any program could debit any account, a signature would mean nothing.

So `deposit` does not move lamports. It **asks** the System Program to. That request is a **cross-program invocation** — a CPI.

## What composability actually buys you

The reason this matters beyond your vault: almost nothing on Solana is self-contained. A program that wants to move SOL calls the System Program. One that wants to move tokens calls the Token program. One that wants to price a swap calls an AMM, which calls the Token program twice. Each of those programs was audited once and is now reused by everyone, which is why a 200-line program can do something a 20,000-line program would otherwise have to.

Your vault is the smallest honest example of that: about six lines of CPI, and it inherits every lamport-accounting guarantee the System Program has.

## The call stack, and how far down it goes

A CPI is a nested instruction. The runtime keeps a stack:

```
transaction
  └─ instruction 1: deposit          ← your program, stack depth 1
       └─ CPI: System Program transfer  ← stack depth 2
```

The ceiling is a constant in the runtime, `MAX_INSTRUCTION_STACK_DEPTH = 5`. Your top-level instruction occupies the first slot, so you get **four levels of nesting below you**. Deep call chains are a real constraint in composed DeFi; for a vault it is not close to a concern. Worth knowing the constant's name so you can check it yourself rather than trusting a blog post — the number has been proposed for a raise more than once.

## Signer propagation: nobody signs twice

Here is the part that surprises people. `transfer` needs the source account to have signed. The user signed the *transaction*. Does the signature reach the CPI?

Yes. Signatures established at the top of the transaction stay valid all the way down the stack. Your program does not re-sign, cannot re-sign, and does not need to. It hands the System Program a `from` account that is already marked as a signer, and the System Program sees a signer.

That single fact is why `deposit` is easy and why the next lesson is hard. Withdrawal needs *the vault* to authorise something, and the vault never signed anything — it has no private key to sign with.

## The Anchor 1.0 `CpiContext`, and the one line the internet gets wrong

A CPI in Anchor is two values: which accounts, and which program.

```rust
transfer(
    CpiContext::new(
        System::id(),
        Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        },
    ),
    amount,
)?;
```

Look at the first argument to `CpiContext::new`. It is a **program id** — a `Pubkey`. In `anchor-lang` 1.1.2 the signature is:

```rust
pub fn new(program_id: Pubkey, accounts: T) -> Self
```

Every CPI tutorial published before Anchor 1.0 — including the earlier version of this course — passes an `AccountInfo` instead:

```rust
// pre-1.0. Does not compile against anchor-lang 1.x.
CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts)
```

Paste that into this lesson's file and the build server says:

```
error[E0308]: mismatched types
   |
   |                 ctx.accounts.system_program.to_account_info(),
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Pubkey`, found `AccountInfo<'_>`
```

This is the single most common stale snippet in Solana material, and it is a *good* break: it fails loudly at compile time instead of quietly at runtime. When your copy-paste from a 2024 blog post fails this way, you now know why.

Two details that follow from the new shape:

- `System::id()` is a compile-time constant. Nothing is looked up, nothing is borrowed, and the context no longer depends on an account being present in order to be *built*.
- You still declare `pub system_program: Program<'info, System>` in the accounts struct. The runtime has to have the callee's program account loaded to execute the nested instruction, and that field is what makes the client include it. What changed in 1.0 is only how `CpiContext` is told which program to call — not whether the program account travels with the transaction.

## The direction rule: why deposit is a CPI and withdraw will not be

Read this from the Agave System Program, `transfer_verified`:

```rust
let mut from = instruction_context.try_borrow_instruction_account(from_account_index)?;
if !from.get_data().is_empty() {
    ic_msg!(invoke_context, "Transfer: `from` must not carry data");
    return Err(InstructionError::InvalidArgument);
}
```

The System Program refuses to send lamports **out of** any account that carries data. Not "out of a PDA" — out of *anything with a non-empty data field*.

Your vault carries 49 bytes. So:

| Direction | Source | Data on the source? | System `transfer` CPI |
| --- | --- | --- | --- |
| deposit | user's wallet | no — wallets hold no data | ✅ works, and is what you read below |
| withdraw | vault PDA | yes, 49 bytes of `VaultState` | ❌ `Transfer: 'from' must not carry data` |

Note where that failure lands: **at runtime, on devnet, after a green build.** The compiler has no opinion about it. Hold that thought until the next lesson.

## Where `new_with_signer` is genuinely required

You will meet `CpiContext::new_with_signer(program_id, accounts, signer_seeds)`. It exists so a PDA can authorise a call it has no key to sign for: the runtime accepts the signature if the seeds you pass re-derive the PDA under *your* program id. That is real, it is central, and it is used constantly — for a PDA that is the **authority** on a token account, or for a **dataless** PDA (allocated with zero bytes) used purely to hold SOL in escrow.

It is not what gets lamports out of a data-carrying vault. Anyone who tells you otherwise has written code that compiles.

## What you are reading in the exercise

The file below is complete and correct. Build it — do not edit it — and read for four things:

1. `Deposit<'info>` re-derives the vault from `bump = vault.bump` rather than re-searching for the canonical bump.
2. The CPI moves the lamports. It is the only thing in the file that touches a balance the program does not own.
3. `ctx.accounts.vault.deposit(amount)?` is your own Course 2 method, imported and called. The `checked_add` and the `ZeroAmount` guard are not retyped here; they are the same lines you already wrote and reasoned about.
4. The two are separate on purpose. Lamports are the runtime's bookkeeping; `vault.balance` is yours. Every real audit finding in a vault lives in the gap between those two numbers, and you cannot inspect a gap whose sides are the same statement.

One honest note about grading, true for every Rust exercise in this course: **the build server grades by compiling.** A green check means the Anchor 1.x toolchain accepted your program. It does not mean the program behaves. That is not a gap we are hiding — it is the reason Module 4 opens by walking you through the LiteSVM test that *would* run the finished vault, and showing exactly which of these mistakes compiling can never catch.
