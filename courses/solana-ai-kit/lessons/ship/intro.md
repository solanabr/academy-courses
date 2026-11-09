# Ship it: deploy, devops-engineer, tech-docs-writer

Stamp has a program, tests, an audit report, a web page, and from last lesson a token proposal, a swap design and a game plan. This lesson puts it on devnet with the kit's approval stop, gives it a pipeline, writes the docs a stranger needs, and lets a reviewer meet the app cold. Around twenty minutes, and the deploy needs the Solana CLI from lesson 3.

## Why this matters

Shipping is where money and keys appear, so it is where the kit's firewall matters most. **The firewall is a set of permission and sandbox rules generated from one tier**, named in `.claude/security.json`, plus guard hooks that run before a command, plus the safe-ai-skill plugin. The default tier, **Relaxed**, lets the toolchain work and **stops every on-chain write for your approval**, naming the cluster, except on a local validator. Irreversible actions such as a final deploy or a program close are refused outright, at every tier, and the agent hands you the command to run yourself. Two agents work in this lane: **devops-engineer** for CI, toolchain pinning, RPC infrastructure and monitoring, and **tech-docs-writer** for the documentation that takes every fact from the code rather than from memory.

## Do this

1. **Deploy to devnet.** Type inside Claude Code:

```text
/deploy devnet
```

The command's own steps: confirm the RPC and the deployer with `solana config get`, run `anchor keys sync` for a never-deployed program so `declare_id` matches the keypair, fund the deployer with an airdrop or the web faucet, then deploy **with the cluster in the command**, so the approval prompt shows which cluster it resolved. That prompt is the firewall. Say yes, and the command publishes the IDL to Program Metadata, checks the program with `solana program show`, and records the id in `.program-id-devnet` and your app env. *Mainnet is a separate step*: the command starts it only when you ask, and first reports what is missing, an audit with no open Critical or High findings, tests and fuzzing passing, a devnet run of the real flows, an upgrade-authority plan, and an external audit for programs that hold user funds.

2. **See the tier you are on.** Type:

```text
/firewall
```

It reports the declared and enforced tier and the rule count. **Medium** is for reading code you did not write, **High** denies mainnet writes outright and is the only tier that stops the agent editing the kit's own configuration, and a switch applies in the next session. Ask Claude to read `.env` once to see the plugin refuse it. The kit's way to know which keys are set is `bash .claude/bin/env-keys.sh`, which prints names and `set` or `empty`, never a value.

![The ship lane: /deploy devnet stops at the firewall prompt that names the cluster, devops-engineer pins the toolchain and writes CI with devnet-only keys, tech-docs-writer states program id, authority, seeds, layouts and errors from the code, and /product-review meets the app as its next user.](assets/v01-ship-lane.webp)

3. **Give it a pipeline.** Type:

```text
Use the devops-engineer agent. Set up CI for Stamp with /setup-ci-cd: build, tests and the audit lints on every pull request, a verifiable build for release, and a devnet deploy on tags. Tell me where the keys live and which versions you pinned.
```

Its file carries the Solana CI facts a generic pipeline gets wrong: **pin the toolchain** (Anchor 1.1.x with Solana CLI 3.1.10, host Rust 1.89 or newer, GLIBC 2.39 or newer), gate merges on build plus tests, build the release artifact with `solana-verify build` so the on-chain program can be verified, install Surfpool because `anchor test` runs on it, and **only devnet keys belong in CI**. The mainnet upgrade authority stays on a hardware wallet or a Squads multisig, and CI may write the buffer and hand it to the multisig, which executes the upgrade. It closes with `/audit-infra`, the infrastructure audit, and with `/doctor` when the toolchain changed, and reports pipelines, secret names without values, pinned versions and the alert list.

4. **Write the docs a stranger needs.** Type:

```text
Use the tech-docs-writer agent to write Stamp's README for frontend and indexer developers, from the code, the IDL and the tests.
```

Its file says what program docs must state: the **program id per cluster and the upgrade authority**, checked with `solana program show`, each instruction's accounts with signer and writable flags, **PDA seeds as exact bytes** and whether the bump is stored, account layouts with offsets because indexers filter on them, compute units per instruction, and **errors by name, decimal and hex**, so `0x1770` is findable. It takes examples from the tests or the generated client and runs them before publishing.

5. **Let a stranger meet the app.** Type:

```text
/product-review
```

It asks for the product, the target user, the one core action and the stage, then walks the app as that user and scores eight dimensions with evidence: onboarding, core loop, empty states, error states, performance, trust signals, mobile, docs. A score comes with the line that earned it, and the roadmap is bucketed by effort. `--harsh` turns it into a stress test with ten weighted dimensions and exactly three fixes. *For Stamp the checkpoint to watch is whether value is visible before the wallet connect*, and for a loyalty card the answer should be yes.

6. **Commit and describe the change.** `/quick-commit` formats, lints and commits on a dated branch with a conventional message and no attribution trailer. `/pr-description` writes a pull request body a reviewer can verify, from the diff, the log and test output that actually ran, *and leaves out any number nobody ran*.

## Done when

- `.program-id-devnet` holds Stamp's id and `solana program show` on it names you as the upgrade authority.
- A CI workflow exists with devnet keys only and pinned versions.
- The README states the program id, seeds as bytes and error codes.
- You have a scorecard with a line of evidence per dimension.

## Watch out

- **For mainnet, the precondition report comes before the approval prompt.** The prompt is the second check.
- **A failed deploy leaves SOL in a buffer.** The command knows how to resume or reclaim it. Never print the keypair or seed phrase it asks about.
- **Public RPC keys ship in the bundle.** The devops file routes them through a Worker or server route.
- **The firewall stops the expensive mistake and asks.** Whether Stamp should be on mainnet at all is still your call.

## The takeaway

Deploys **stop for your approval and name the cluster**, mainnet starts with a precondition report, CI holds devnet keys only, and the docs state the id, the authority, the seeds and the error codes from the code. *The firewall sits at the deploy because that is where the money is.*

## Next

Next lesson is yours: three idea threads laid out as agent chains, from the first research prompt to a weekend's output, and a template to write your own. Bring an idea, even a bad one.
