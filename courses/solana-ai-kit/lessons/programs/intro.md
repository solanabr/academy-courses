# Build the program: anchor-engineer, pinocchio-engineer, solana-qa-engineer

Last lesson ended with `design.md`: a shop account, a card per customer, three instructions. This lesson an engineer builds them, a QA engineer tests them, and an audit reads the result. **Rust, Anchor and the Solana CLI enter here.** If they are not on your machine, follow on the screen and install them tonight. The Solana CLI command is in your `/doctor` report. The Anchor toolchain appears there once an Anchor workspace exists, so for tonight it is `rustup` for Rust, then `cargo install --git https://github.com/solana-foundation/anchor avm --force && avm install latest && avm use latest` for Anchor. Around twenty minutes.

## Why this matters

Program code is where a copied tutorial hurts most, because the Anchor API changed at 1.0 and most of the web still shows the old shape. The kit has **two implementers and one tester**. **anchor-engineer** is the default: Anchor 1.x, constraints instead of hand-written checks, generated IDL, Rust LiteSVM tests. **pinocchio-engineer** exists for the measured cases, *an instruction still over its compute budget after Anchor's own levers, compute fees at volume, or a binary-size limit*, and it writes every check by hand. **solana-qa-engineer** owns how the program is tested: which harness for which test, the coverage bar, compute baselines, and stripping the noise a model leaves in a diff. Three commands tie them together: `/build-program`, `/test-rust`, `/audit-solana`.

## Do this

1. **Hand the design to the engineer.** Type inside Claude Code:

```text
Use the anchor-engineer agent. Set up an Anchor 1.x workspace for Stamp with the LiteSVM test template and implement the three instructions in design.md: create_shop, stamp and redeem. One card account per customer as designed, checked arithmetic, stored bumps, and a Rust test per instruction. Run /build-program and /test-rust before you hand back.
```

Watch for the file it reads first: the Anchor reference inside the solana-dev pack. Then the shapes its own file insists on, the ones **older habits get wrong**: account space from `DISCRIMINATOR.len() + INIT_SPACE`, `CpiContext::new` taking a program key rather than an account info, stored PDAs validated with `bump = card.bump`, error codes starting at 6000. The house rules in `CLAUDE.md` add checked arithmetic and no `unwrap` outside tests. *Its closing rule is build, test, and an audit for code that moves value*, and it reports the instructions and accounts it changed with the test results.

2. **Read one test failure the kit's way.** If a test fails, the error arrives as hex. `/test-rust`'s file decodes it: **2000 to 2999** are constraint violations, **3000 to 3999** account errors, **4100** means the program was loaded at an address other than its `declare_id`, and **6000 and up** are your own error enum. Ask for the program logs before anyone changes code:

```text
Show me the program logs from the failing redeem test and tell me whether the test or the program is wrong, before you change anything.
```

![The program lane: design.md goes to anchor-engineer for instructions and LiteSVM tests, /build-program and /test-rust run before hand-back, solana-qa-engineer adds negative tests and strips slop, /audit-solana writes a findings report, and pinocchio-engineer is the branch taken for a measured compute problem.](assets/v01-program-lane.webp)

3. **Bring in the tester.** Type:

```text
Use the solana-qa-engineer agent. Review Stamp's tests against your coverage bar: every instruction, every error code hit by a negative test, and the boundaries, including a redeem with nine stamps and a stamp on a card at the maximum count. Add what is missing, run /test-rust, and strip any AI slop from the diff.
```

Its file gives it a **harness table**: Mollusk for one instruction and exact compute, LiteSVM for multi-instruction flows, Surfpool for real mainnet state, Trident for fuzzing before mainnet. For Stamp it will stay in LiteSVM. The part people do not expect is the **slop pass**: comments that restate the code, defensive checks around trusted internal calls, debug logging left in, verbose error strings. It removes them and tells you what it removed in a sentence or two.

4. **Audit it.** Type:

```text
/audit-solana
```

The command scopes every instruction, runs `cargo audit` and clippy with the arithmetic and unwrap lints, walks the auditor-skill checklists, and writes `docs/security-audit-<date>.md`: counts per severity, each finding with `file:line`, the exploit scenario and the fix, a separate list of unconfirmed suspicions, and a verdict. For Stamp expect it to look hard at **who can call stamp**, because a shop key that signs from a phone is the risk the architect already named, and at the redeem arithmetic.

5. **Know when the other engineer exists.** Stamp will never need Pinocchio, and that is the point of knowing the rule: the architect's file says *optimize one hot instruction in place first and port it only if it is still over budget*. When that day comes, `/profile-cu` measures compute per instruction, `/benchmark` diffs it against a baseline, and pinocchio-engineer closes with both. Its file is also the honest list of what manual validation costs: one-byte discriminators, zero-copy alignment, no generated IDL.

## Done when

- `/build-program` reports the program built, its size and id, and the IDL written.
- `/test-rust` passes with a negative test per error code.
- `docs/security-audit-<date>.md` exists and you have read its verdict line.

## Watch out

- **Rebuild before testing.** LiteSVM bakes the program binary in at compile time, so `cargo test` alone tests whatever was built last. The test command does this for you.
- **Two error enums collide.** Both start at 6000 unless one sets an offset, and the collision is silent. The engineer's file knows this. A copied tutorial does not.
- **An audit is not an external audit.** `/audit-solana` recommends one in its own output for programs that hold user funds.
- **No toolchain today is fine.** Read the transcript on the screen and run the install commands later. Lesson 4 is Node only again.

## The takeaway

One engineer builds from the design with the current Anchor API, one tester holds the coverage bar and strips the slop, and one command writes an audit report, **with build, test and audit as the fixed closing routine**. *Pinocchio is a measured decision, never a default.*

## Next

Next lesson the frontend engineer puts a customer's card on a web page with a wallet connected on devnet, which runs on Node alone. The mobile and backend engineers get their turn in the same lesson.
