# What You've Built — and Writing It Up

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · edition 2021 · on-chain IDL via the **Program Metadata Program** (`@solana-program/program-metadata`), not the removed `anchor idl init`. Every claim this lesson tells you to make about your own build is one that is true of the artifact you actually shipped.

You have a program id. It is live on a public network, anyone can look it up without asking you, and it did not exist eleven lessons ago.

## Name the pieces

Vague recaps are worth nothing, so here is the specific list. Every item is something you can point at.

| What you shipped                | Where it came from                                                                                                       |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| **The Course 2 vault core**     | `VaultState`, `VaultError`, `deposit`, `withdraw` — imported into module 1 unchanged, not retyped. The half of a program that has no runtime dependency |
| **The vault PDA**               | A per-user, program-owned account at `[b"vault", your_key]` — 49 bytes, space derived rather than guessed, canonical bump stored at init |
| **A System Program CPI**        | `transfer(CpiContext::new(System::id(), Transfer { … }), amount)` on deposit — the Anchor 1.0 shape, where `new` takes the program's ID rather than its `AccountInfo` |
| **The instruction with no CPI** | `withdraw`, which debits the vault directly because your program owns it — and which therefore has no `system_program` in its accounts struct at all |
| **A rent floor you enforced**   | `Rent::get()?.minimum_balance(data_len())`, checked before a lamport moves, so a too-large withdrawal returns your `InsufficientFunds` instead of the runtime's `InsufficientFundsForRent` |
| **Account constraints**         | `seeds` + `bump = vault.bump` binding the vault to its signer, `Account<'info, VaultState>` enforcing the discriminator, an ownership `constraint` raising `NotOwner`, and a decision about `dup` |
| **A tested-by-reading pre-flight** | The LiteSVM tier: what a test that actually runs `withdraw` asserts, and why compile-success cannot assert it            |
| **A program id**                | Yours, on devnet, under the BPF Loader Upgradeable, with your wallet as upgrade authority                                 |
| **An on-chain IDL**             | Published through the Program Metadata Program, reachable from the program id alone                                       |

The distinction worth keeping: **Anchor generated the boilerplate; you made every decision.** The discriminator, the deserialization, the create-account CPI, the error-code plumbing — macros wrote those. Which seeds. How much space. Where the bump lives. Which error names which failure. Which constraint stops which attack. Those were yours, and they are the part that transfers to the next program you write, in any framework.

## Two concrete next steps

### 1. Course 4 starts from what you just made

**`dapp-and-sdk-with-kit`** takes the program id and the IDL from the last lesson, runs Codama over the IDL to generate a typed client, and puts a real interface on your vault. That is why publishing the IDL was a step rather than a footnote: the next course is told nothing about your program except its address.

Bring the id. Nothing else is needed.

### 2. Turbin3, and paid work on Earn

**Turbin3's** admission process includes a Rust prerequisite task that requires executing a transaction on-chain. You have done exactly that, with a program you wrote, and you can point at the signature.

**Superteam Earn** lists bounties judged on submissions rather than résumés, and the shape of your artifact matches two categories of listing well:

- **Program work.** Chapter listings for Anchor vaults, escrows and security templates have run in the **$300–$4,000 USDG** range, at roughly **7–62 submissions** each. Your vault is a smaller version of exactly that brief. Start with the [development listings](https://superteam.fun/earn/category/development/) and prefer small, tightly-scoped deliverables for a first submission.
- **Writing about program work.** Educational modules and technical deep dives have paid **$1,500–$3,000** at roughly **10–50 competitors** — noticeably fewer people per dollar than the development side. That asymmetry is the single best-odds thing on the board for someone who has just built something and can explain it. It is also why the write-up below is a deliverable and not homework.

Treat those ranges as the historical shape of the board, not a forecast, and never read one day's submission count as a competition rate.

If you have a larger idea, [Superteam grants](https://superteam.fun/earn/grants/) fund builders directly. A live devnet program is the strongest artifact a first application can carry.

## The write-up

This is the second deliverable of the course, and the block below is where you draft it. It is never graded, awards no XP and gates nothing — it exists because the version of you that just finished this has information that disappears within a week.

A structure that works, and roughly what belongs in each part:

1. **What it is, in two sentences.** "A per-user SOL vault on Solana devnet: an Anchor program with `initialize_vault`, `deposit` and `withdraw`, where the vault is a program-owned PDA that the program debits directly, above an enforced rent floor." Then the program id, as a link with `?cluster=devnet`.
2. **One design decision, defended.** Not a tour — one choice. Why the canonical bump is stored in the account instead of re-derived, and what re-deriving costs. Or why `checked_sub` and a named error beat bare subtraction even with `overflow-checks = true`. One paragraph on a real decision beats five paragraphs of narration, and it is the paragraph a reviewer reads to decide whether you understood what you built.
3. **The thing that broke, and how you found it.** Everyone's write-up has the happy path. Almost none have this. If you hit the deliberate compile error in module 2, or a transaction that failed with a custom error code you had to look up, that is the most useful paragraph you can write — for a reader and for a sponsor assessing whether you can debug.
4. **What you would do differently.** Short, specific, no false modesty.

Two rules about claims, because they are the kind of thing that costs credibility:

- Say **"compiled against the real Anchor 1.1 toolchain."** Do not say "unit tested" — grading here is compile-only, and the LiteSVM tier is something you read rather than ran. The precise claim is more impressive than the loose one to anyone who knows the difference, and the loose one is checkable.
- Link the program id rather than describing it. It is on a public network; a reader can verify every claim you make about it in about ten seconds. That verifiability is worth more than any certificate screenshot, and it is the actual asset this course produced.

Your deployed program stays on devnet. Nothing expires, nothing is collected, and the id keeps resolving.
