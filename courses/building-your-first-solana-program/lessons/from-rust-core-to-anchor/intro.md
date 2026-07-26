# From Rust Core to Anchor

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · `borsh` resolves to **1.8.0** · edition 2021. This lesson's starter and solution were both compiled against that toolchain; the build is green with no edits.

You have a file that compiles. It is called `vault_core.rs`, you wrote it at the end of Course 2, and it holds a `VaultState` struct, a `VaultError` enum, and two methods that do checked arithmetic and return named errors instead of panicking.

It is not a Solana program. It has no `#[program]` module and no `#[derive(Accounts)]` struct — **that is what this course adds.**

That distinction is the shape of all fifteen lessons in this course, so be precise about it. Nothing in Course 2 was a program: `#[account]` and `#[error_code]` are Anchor attributes and you used both, but neither one makes a crate into something the runtime can call. A program needs an entrypoint and a dispatcher. You have not written one yet. You are about to compile one.

## The two halves, side by side

The starter below is your Course 2 file with the wrapper added underneath it, separated by a labeled line. Here is the same split as a table.

| Your Course 2 file                                   | What Course 3 adds                                                             |
| ---------------------------------------------------- | ------------------------------------------------------------------------------ |
| `pub mod vault { … }` — an ordinary Rust module       | `#[program] pub mod vault_program { … }` — instruction handlers                 |
| `#[account] pub struct VaultState` — a *shape*        | `#[derive(Accounts)] pub struct DryRun` — an *account list*                     |
| `#[error_code] pub enum VaultError` — named failures  | nothing: one per program is the practice, and you already have yours            |
| `impl VaultState { deposit, withdraw }` — the logic   | handler bodies that **call** `deposit` and `withdraw`                           |
| Callable only from other Rust                        | Callable from a transaction, by address                                        |

Two attributes doing two different jobs:

**`#[program]`** expands the module into a dispatcher and a program entrypoint. Every `pub fn` inside becomes a callable instruction, selected by an 8-byte discriminator derived from its name. This is the line that turns a library into a program.

**`#[derive(Accounts)]`** generates the deserialization and validation code for one instruction's account list. `DryRun` is empty, so `dry_run` may touch no accounts at all — which is exactly why nothing it does survives the transaction. Read the handler and notice what is absent: there is no account, so `VaultState` is a value on the stack, `deposit` mutates it, and it is gone when the instruction returns. Module 2 is where the vault stops being a local variable.

What the body *does* show is the division of labour for the rest of the course. It calls `vault.deposit(1_000)?` — your method, imported, not retyped. Handlers validate accounts and delegate; the arithmetic stays in the core you already reasoned through line by line.

## What happens when you press Build

Four stages, none of them on your machine:

1. **Rust source** — the single file in the editor. It becomes `src/lib.rs` of a fixed crate, the one whose `Cargo.toml` you read at the end of Course 2.
2. **`cargo-build-sbf`** — the Solana compiler driver. It compiles the crate to Solana Bytecode Format, a register-based instruction set derived from eBPF and executed by the runtime's VM: sandboxed, with no host filesystem and no sockets, and deterministic, because the same inputs must produce the same result on every validator.
3. **A `.so` binary** — an ELF shared object. The build response carries its identifier; in module 4 one of these gets uploaded to devnet.
4. **A green check, or compiler output** — back in the browser tab.

The toolchain the build server pins, which is the version stamp for every code block in this course:

| Layer          | Version                                             |
| -------------- | --------------------------------------------------- |
| `anchor-lang`  | **1.1.2**                                           |
| platform-tools | **v1.54** — rustc 1.89, Anchor 1.x's minimum        |
| Rust edition   | 2021                                                |
| `borsh`        | `1.5.7`, a range — so 1.8.0 is what actually resolves |

Two consequences worth internalising now. **Warnings are not failures:** this starter returns six of them, every one from inside the `#[program]` expansion, and the build is green. And **the sandbox is real:** the server rejects a submission containing `std::process`, `std::fs::`, `std::net::`, `std::env::`, `Command::new`, `include_bytes!`, `include_str!`, `env!(`, `option_env!` or `proc_macro` before the compiler ever sees it. None of those belongs in a program anyway.

Be exact about what a green check proves: **your code compiled against the real Anchor 1.x toolchain.** No test ran. Nothing executed. A `withdraw` that calls `checked_add` compiles perfectly, and nothing in this course will catch it — module 4 shows you the LiteSVM test that would, and why no compiler can.

## The seam: two things called "Anchor"

Search for Anchor documentation today and you will land on material for a different version of a differently-named package. Three facts save you an afternoon.

- **The Rust crate is `anchor-lang`, at `1.1`.** Anchor 1.0 was a breaking release and the internet has not caught up. When a snippet does not compile here, suspect its age before you suspect yourself; the specific breakages are called out in the lessons where they bite.
- **The TypeScript package is `@anchor-lang/core`.** The old one, `@coral-xyz/anchor`, is frozen at 0.32.1 and still pulls roughly 659k weekly downloads against `@anchor-lang/core`'s 16k. Popularity is not currency — that ratio measures how much stale tutorial code is in circulation. You need the TS side in Course 4, not here, but you will meet the wrong package name long before then.
- **The version manager installs from `otter-sec/anchor`,** not `coral-xyz/anchor`. Maintenance moved. A `cargo install --git …/coral-xyz/anchor avm` line dates a blog post on its own.

None of this is your problem in this lesson, because you are installing nothing. That is what the build server is for: no toolchain, no version manager, no `PATH`. Browser builds are not new — Solana Playground has offered them for years and the Foundation's own quickstart uses one. What is different here is that this path is **graded, sequenced and credentialed**, and the program you deploy in module 4 is an artifact that three later things consume.

## Your job

Press **Build**. Change nothing. Read the output.
