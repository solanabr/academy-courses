# Connect, Fund, and Send It

> Version stamp — `@solana/kit` 7.0.0 · devnet · authored 2026-07-28.

Everything so far was reading and rehearsal. Today: a real wallet, real devnet SOL, and a real deposit into the vault you have been decoding since lesson 1 — confirmed on-chain, with a signature you can hand anyone as proof.

## Connecting: Wallet Standard, not wallet-adapter

Modern wallets (Phantom, Solflare, Backpack, …) implement **Wallet Standard** — a browser protocol in which the wallet announces itself to the page. Your app does not install a package per wallet; it *discovers* whatever the visitor already has:

```ts
import { createClient } from "@solana/kit";
import { solanaDevnetRpc } from "@solana/kit-plugin-rpc";
import { walletSigner } from "@solana/kit-plugin-wallet";

const client = createClient()
  .use(solanaDevnetRpc())
  .use(walletSigner()); // Wallet Standard discovery — no per-wallet adapters
```

If a tutorial hands you `ConnectionProvider`, `WalletProvider`, or `PhantomWalletAdapter`, you are reading the legacy stack from lesson 5 — read it, don't copy it.

When the wallet connects, your app gets an **account and a signer** — the wallet holds the private key and never gives it up. Your code's job is to *assemble* the transaction; the wallet's job is to *sign* it. That division is the whole security model, and it is why the exercise below builds everything **except** the signature.

## Funding

The button below fetches devnet SOL to your connected wallet. Devnet SOL is free and worthless by design — it exists so you can do exactly this. The faucet rations amounts and can run dry at busy times; if it declines, wait a little and retry. What lands in your wallet funds today's deposit *and* stays with you: in Course 3, this same wallet pays your own program's deploy.

## The deposit, precisely

You already know every ingredient:

- **Which instruction**: the vault program's `deposit`, from lesson 1's IDL.
- **Which accounts, in which order**: the IDL fixes it — **the vault PDA first**, then your wallet (writable, signer), then the System Program. The order is part of the program's interface: the program reads accounts by position, so a swapped order is not style, it is a different (failing) call.
- **Which vault**: *your* wallet's vault, derived exactly as in lesson 3 — seeds `["vault", yourAddress]`. First deposit? The program creates it — which is why the System Program is in the account list.
- **What data**: 8 bytes of instruction discriminator — `[242, 35, 198, 137, 82, 225, 242, 182]`, the `deposit` tag from the IDL — followed by the amount as a **u64 little-endian**, the exact write-side of the read you did in lesson 2.
- **What it costs**: you predicted it in lesson 4 — 5,000 lamports for your one signature, plus any priority fee on the *requested* limit.

Amounts are **integer lamports as bigint**, always: `0.001 SOL` is `1_000_000n` lamports. No floats anywhere near money.

The graded exercise assembles this plan as a pure function in four labeled subgoals — the same shape Kit's `pipe` built in lesson 4, with the instruction now built byte by byte. The grader checks the account order the way the program would: vault first, or it fails.

After it passes, read **Send, Watch, and Read the Failure** below, then send the real thing from the embedded runner: connect, fund, deposit. When the confirmation lands, open your transaction in the explorer (`?cluster=devnet`!) and look at the vault's balance move.
