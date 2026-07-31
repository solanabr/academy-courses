# Kit, or What You'll Find in the Wild

> Version stamp — `@solana/kit` 7.0.0 · devnet · authored 2026-07-28.

This lesson adds no new machinery. It exists because the first real codebase you open will not look like this course — and you need to read it anyway.

## The seam, with dates

Two stacks coexist right now, and pretending otherwise would set you up to fail:

- **`@solana/kit`** — the current stack, what solana.com documents, what this course teaches. Weekly downloads around 1.65M (checked 2026-07-28).
- **`@solana/web3.js` v1** — officially labeled superseded, yet it still holds the npm **`latest`** tag at 1.98.x with roughly 2.1M weekly downloads. `npm i @solana/web3.js` installs v1 **today**. Its companion `@solana/wallet-adapter-*` is likewise superseded and likewise everywhere.

So: every bounty codebase, most Stack Overflow answers, and most tutorials you will search speak v1. "Canonical" and "what you will be paid to touch" have diverged. The skill is **read v1 fluently, write Kit only** — this course never asks you to write a line of v1, and neither should you.

One more honesty note: the ecosystem also contains `@solana/client`, `@solana/react-hooks`, and `gill` — packages whose own documentation pages disagree about their status. This course pins **one** stack, `@solana/kit` 7.0.0, and says so, so that every line you learn has one current answer.

## THE MAP

Left column: what you will read in the wild. Right column: what you write instead. The right-hand strings below are exactly the ones the drill grades against.

| Legacy (read it) | Kit (write it) |
| --- | --- |
| `new Connection(endpoint)` | `createSolanaRpc(endpoint)` |
| `clusterApiUrl('devnet')` | `an explicit endpoint string` |
| `new PublicKey(base58)` | `address(base58)` |
| `Keypair.generate()` | `generateKeyPairSigner()` |
| `SystemProgram.transfer({...})` | `getTransferSolInstruction({...})` |
| `sendAndConfirmTransaction(connection, tx, [payer])` | `sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })` |
| `connection.getAccountInfo(pubkey)` | `rpc.getAccountInfo(address).send()` |
| `PublicKey.findProgramAddressSync(seeds, programId)` | `await getProgramDerivedAddress({ programAddress, seeds })` |
| `useWallet() from @solana/wallet-adapter-react` | `@solana/kit-plugin-wallet (Wallet Standard)` |

The pattern behind the rows, so you can extend the map yourself:

- **Classes become functions.** v1 wraps everything in `new` — `Connection`, `PublicKey`, `Transaction`. Kit is plain functions returning plain values.
- **Magic endpoints become explicit.** `clusterApiUrl('devnet')` hid the URL; Kit asks you to write `"https://api.devnet.solana.com"` where everyone can see which chain you are on.
- **Sync PDA derivation becomes async.** `findProgramAddressSync` blocked while it searched for the bump; `getProgramDerivedAddress` is awaited — the single most common porting mistake is a forgotten `await` handing you a `Promise` where an address should be.
- **The confirm helper becomes a factory.** v1's `sendAndConfirmTransaction` took a connection each call. Kit's `sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })` binds its dependencies once and returns the function you actually call — you will use it for real in the next lesson.
- **Wallet plumbing becomes a protocol.** wallet-adapter needed a package per wallet; Wallet Standard is implemented by the wallets themselves, and `@solana/kit-plugin-wallet` just discovers them.

## How this lesson runs

First, a **cumulative check** — six questions reaching back through lessons 1–4, each framed against unfamiliar legacy code, because recall under new packaging is the thing interviews and codebases actually demand. Then the **translation drill**: you fill in THE MAP as a function and it is graded over legacy call sequences — a transfer flow you have seen, plus an account-read and a PDA flow you have not. Finally a short **spot-the-legacy** check on real-world tells.
