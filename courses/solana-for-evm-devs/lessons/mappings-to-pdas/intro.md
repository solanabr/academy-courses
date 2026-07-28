# A `mapping` becomes an address you can compute

`balances[user]` is a hash of the slot number and the key, resolved by the EVM at
read time inside the contract's storage. You never see the location; you just
index.

Solana has no mapping, because it has no per-program storage to index into. What
it has instead is stranger and, once it clicks, more useful: you can *derive an
address* from a set of seeds, deterministically, off-chain, before you send
anything.

That address is a **program-derived address** — a PDA.

```ts
const [vaultPda, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), owner.toBuffer()],
  programId
);
```

Given the same seeds and the same program id, everyone computes the same
address — your frontend, your tests, another protocol integrating with you. The
mapping key became part of the address itself.

## The two properties that matter

**No private key exists for a PDA.** These addresses are deliberately chosen to
fall *off* the ed25519 curve, so no keypair can produce a signature for them.
Nobody can sign as your vault by holding a secret — instead, the owning program
can sign for it, by supplying the seeds. Authority comes from the derivation, not
from a secret. There is no EVM equivalent, because in the EVM authority over a
contract's storage is simply "being that contract".

**The bump is the search.** Most seed combinations land *on* the curve, which
would be a valid keypair address and therefore unusable. So the derivation
appends a byte — the bump — and counts down from 255 until it finds an off-curve
result. `findProgramAddressSync` returns the first one it finds (the *canonical*
bump). Programs store that bump so they never repeat the search, and so an
attacker cannot supply a different, non-canonical bump to derive a second valid
address for the same logical key.

## What you gain, and what you give up

You gain addressability. A Solidity `mapping` entry has no address you can hand
to anyone; a PDA is a first-class account you can pass to another program, list
in a transaction, or read directly from a client with no view function and no
RPC call into your contract.

You give up enumeration. `balances` cannot be listed on the EVM either, but at
least the contract can maintain its own index. On Solana the accounts exist
independently of any registry, so if you need to enumerate holders you either
maintain that index yourself or lean on an indexer that scans accounts by owner.

## In the exercise

You will build the seed list itself — the small, boring, easy-to-get-wrong piece
that determines every address your program will ever use. Order matters, the
literal prefix matters, and both are part of your program's public interface: a
seed change is an address change, and an address change orphans every account
that existed before.
