# Derive the Vault PDA

> Version stamp — `@solana/kit` 7.0.0 · devnet · authored 2026-07-28.

The vault's address — `FY86s1fAwUiFQTjVFYprsiV6fwNH7e955MSUBo73FP4j` — was never generated from a private key. Nobody holds a secret for it, and nobody ever will. It is a **Program Derived Address**: computed, deterministically, from ingredients you already have.

## The recipe

A PDA is derived from three inputs:

1. **Seeds** — an ordered list of byte strings. For this vault: the UTF-8 bytes of `"vault"`, then the owner's 32-byte public key.
2. **The program's address** — `D7ZF…`. The same seeds under a different program give a completely different address, so programs can never collide on each other's accounts.
3. **A bump** — one byte, found by search (below).

The derivation is a SHA-256 hash over `seeds + bump + program address + a fixed marker string ("ProgramDerivedAddress")`. The marker keeps PDAs from ever colliding with hashes computed for any other purpose.

## Why the bump exists

Regular addresses are points **on** the ed25519 curve — that is what makes them signable by a private key. A PDA must be the opposite: an address **off** the curve, so that *no private key can ever exist for it*. Only the program itself (with the runtime's help) can authorize actions for it.

But a hash lands on the curve roughly half the time. So the derivation *searches*: try bump 255 — if the hash is on the curve, try 254, then 253… until the result falls off the curve. The first bump that works (searching downward) is the **canonical bump**.

Two facts follow, and both showed up in your decoder yesterday:

- The reference vault's stored bump is **255** — the very first candidate was already off-curve.
- The bump is stored in the account (byte 48) so the program never has to redo the search. Deriving is cheap for you; on-chain, re-searching every call would be wasted compute.

## In Kit

```ts
import {
  getProgramDerivedAddress,
  getUtf8Encoder,
  getAddressEncoder,
} from "@solana/kit";

const [address, bump] = await getProgramDerivedAddress({
  programAddress: VAULT_PROGRAM,
  seeds: [
    getUtf8Encoder().encode("vault"),
    getAddressEncoder().encode(owner),
  ],
});
```

Note it is `await` — the function is async — and it returns a pair: the address *and* the canonical bump it found.

**Seed order is part of the address.** `["vault", owner]` and `[owner, "vault"]` hash to different byte sequences, so they derive different addresses — same ingredients, different meal. The program defined the order once (you saw it in the IDL yesterday: constant `"vault"` first, then the user account); every client must reproduce it exactly.

## The exercise

The grader has no network and no crypto library, so `derive` is **injected**: a lookup over outputs recorded from real `getProgramDerivedAddress` runs on 2026-07-28 — the correct derivation for two different owners, plus the wrong-seed-order and wrong-program derivations, all real. Your job is the part that is yours in production too: assemble the seeds in the right order and hand the right program address to `derive`.

The parts bin holds five lines. Three belong. One flips the seed order — it will "work" and return a real address that is simply not the vault. One passes the owner as the *program* address — a category error the lookup refuses loudly. Choose and order the lines, then run the tests: one fixture owner is the reference wallet from lesson 1, the other is a different wallet whose vault you have never seen.
