# From IDL to a Typed Client

You finished Course 3 holding two things: a program id on devnet, and an Anchor 1.x IDL — a JSON description of every instruction, account and error your vault exposes. Those two artifacts are the entire input to this course. Nobody who installs your client will read your Rust; they will read the IDL you already have, through a generated TypeScript client.

> **Version stamp — kit 7.0.0, @codama/renderers-js 2.3.0, checked 2026-07-27.** This is the most version-fragile course in the catalog: it sits on sub-1.0 Kit plugins. Every version below is pinned at authoring and verified against the live registry. A kit-8 release (an `8.0.0-canary` was published 2026-07-24) would invalidate these stamps first — check the date before trusting them.

## What Codama does

[Codama](https://github.com/codama-idl/codama) reads an IDL and generates a client: typed instruction builders, account decoders, PDA helpers and a decoded error map. Its config lives at your repo root:

```json
{
  "idl": "program/idl.json",
  "scripts": { "js": { "from": "@codama/renderers-js", "args": ["src/generated/vault"] } }
}
```

`npx codama run js` writes a tree like this:

```
src/generated/vault/
  accounts/       fetchVault, fetchMaybeVault, getVaultDecoder
  instructions/   getDepositInstruction, getWithdrawInstruction
  types/          the argument and account types
  errors/         a decoded map of your VaultError codes
  programs/       a Kit program plugin
```

The types are **Kit types**, not the old web3.js ones: an account address is an `Address`, not a `PublicKey`, and a lamport amount is a `bigint`, not a `number`. That difference runs through the whole course.

## Why you read the output here instead of generating it now

The documented `renderVisitor` writes files to disk, and this lesson's runner is a sandbox with no filesystem — so codegen does not run here. It runs later, in your CI workflow (lesson 4), which is also where it belongs: the client is regenerated from the IDL on every publish, never edited by hand. In this lesson you read the shape of what it produces and call a builder the way the generated code does, so the structure is familiar before you depend on it.

Anchor 1.x can run this same generation itself — `anchor codama generate -l js -p clients` — if you would rather not add a Codama config. Either way the output is the same shape.

## Two things this course will never do

- It will **never** teach `new Program(idl, provider)`. That is the old Anchor TypeScript client; the Kit path is a generated client plus your own wrapper.
- It will **never** fetch the IDL from the chain with the old `anchor idl` instructions. Your IDL is the build artifact from Course 3 — a file you already have.

## The worked example

Below is a hand-written stand-in for one generated instruction builder — `buildDepositInstruction` — assembled exactly the way `getDepositInstruction` assembles it: a program address, an ordered account list with roles, and a data payload carrying the instruction discriminator and the argument. Run it, read the returned object, and notice three things: the accounts are ordered and role-tagged, the amount is a `bigint`, and nothing in the shape mentions your Rust. That object is what every later lesson sends.
