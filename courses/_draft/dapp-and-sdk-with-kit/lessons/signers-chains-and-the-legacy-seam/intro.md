# Which Signer, and Why

A Wallet Standard wallet advertises what it can do as a list of **features**. Your job is to pick the right signer for the feature the wallet actually offers — and to know who broadcasts the transaction once it is signed, because that changes what your app is responsible for.

## The taxonomy

From `@solana/wallet-account-signer` (React equivalents in `@solana/react`):

| Advertised feature | Signer factory | Result | Who broadcasts |
| --- | --- | --- | --- |
| `solana:signTransaction` | `createTransactionSignerFromWalletAccount` | a `TransactionModifyingSigner` | **your app** — you get the signed bytes and send them |
| `solana:signAndSendTransaction` | `createTransactionSendingSignerFromWalletAccount` | a sending signer | **the wallet** — you never see the signed bytes |
| `solana:signMessage` | `createMessageSignerFromWalletAccount` | a message signer | nobody — you cannot send a transaction with only this |

`createSignerFromWalletAccount` inspects the features at call time and throws if neither transaction feature exists.

## The rule for *this* app

When a wallet advertises both transaction features, this app prefers `solana:signTransaction` — the one where **your app broadcasts**. That is not the universal default; it is the right call here because module 3 simulates the transaction, sizes its compute budget, and prices a priority fee, all of which have to happen to a transaction *you* still hold before it goes out. A sending signer hands broadcast to the wallet and takes that control away. Prefer the modifying signer; fall back to the sending signer only when the wallet offers nothing else.

## Chains are literal, and a wallet can say no

Chain ids are strings: `'solana:devnet'`, `'solana:mainnet'`. A connected account can **refuse** a chain — you ask it to sign for `solana:devnet` and it throws because it is a mainnet-only account. That is a real runtime error worth handling, not an edge case.

## The seam — read it, don't write it

The old stack has not gone away. `@solana/web3.js` v1 still holds npm's `latest` tag and out-downloads Kit (roughly 2.1M vs 1.65M weekly at the time of writing), and `@solana/wallet-adapter-react` is around 810k/week with a repo that is still maintained. "Canonical" and "what you will be paid to touch" have diverged: you will open codebases that use `useWallet()`, `publicKey` and `sendTransaction`, and you need to read them fluently — map `useWallet()` to Wallet Standard discovery, `publicKey` to the connected account's `address`, `sendTransaction` to a sending signer. Learn to read it. Do not learn to write it; this app does not.

## The exercise

`selectSigner` receives a pipe-joined list of the features a wallet advertises and returns `{ signer, broadcaster }`. Every branch is present in the starter but scrambled, with one decoy. Order them so this app's preference — a modifying signer when it can get one — comes out right.
