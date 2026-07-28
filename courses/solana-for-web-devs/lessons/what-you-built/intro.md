# What You Built

Six lessons ago the vault was a URL you could not read. Inventory what exists now — each item is a real artifact, not a feeling:

- **A graded, working account inspector** — `classifyAccount` plus `inspect`, passing every fixture including the five failure paths. It reads any devnet address and answers with type, SOL, owner, rent-exemption, and — only when the owner and discriminator prove it — decoded vault fields. It lives in your completed lesson 7, and it is the through-line artifact of this whole track.
- **Your confirmed devnet transaction** — the deposit you assembled byte-by-byte and a wallet signed in lesson 6. Its explorer signature is on your profile via the lesson-6 reflection: a public, permanent record anyone can verify, showing your fee, your account order (vault first), and the vault's balance moving.
- **A funded devnet wallet** — connected through Wallet Standard, holding the SOL the faucet gave you minus one base fee. Keep it: **in Course 3 this exact wallet pays your own program's deploy** — rent for the program account and 200+ transactions of upload fees. (Course 2 spends nothing — it is all compiling.)
- **Your credential and XP** — the on-chain record that you completed these eight lessons, minted to the same wallet.

## The handoff contract, byte for byte

The one thing Course 2 picks up is not code — it is **knowledge of a layout**. Your decoder reads:

```
offset 0   8 bytes   discriminator [228, 196, 82, 165, 98, 210, 235, 152]
offset 8   32 bytes  owner
offset 40  8 bytes   balance, u64 little-endian
offset 48  1 byte    bump
```

In Course 2 you write the Rust struct that *produces* those bytes — `owner: Pubkey`, `balance: u64`, `bump: u8`, in that order. It must match your decoder **byte for byte**, because there is no translation layer between them: the struct's memory layout, serialized, IS the account data your `DataView` read. Get a field order wrong, or a width wrong, and your own inspector reports garbage against your own program — the mistake announces itself in a tool you built.

The seeds you ordered in lesson 3 — `["vault", owner]`, constant first — become the `seeds = [...]` constraint in Course 3's account validation, character for character. And the hand-written decoder itself? Course 4 replaces it with a generated client — which is the point: you will know exactly what the generator generates, because you wrote it once by hand.

Run the cumulative check below — it draws on every lesson — then the closing reflection.
