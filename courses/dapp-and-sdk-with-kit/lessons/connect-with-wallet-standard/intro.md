# Connect a Wallet Without wallet-adapter

You have a published package. Now build the app that uses it — starting with the one thing every dApp needs and most get wrong: connecting a wallet.

> **Why there is no graded exercise here.** This lesson's runtime is a sandbox with no DOM and no module resolution, so a React component and live Wallet Standard discovery cannot be graded. You get an annotated walkthrough instead. The graded rung of this module is the next lesson, where signer selection is pure logic. The lesson after that grades the transaction you assemble.

## One client, built from plugins

Kit builds a client by composing plugins. You build it **once, at module scope**, and share it through a provider:

```ts
// client.ts — built once
import { createClient } from "@solana/kit";
import { solanaDevnetRpc } from "@solana/kit-plugin-rpc";
import { walletSigner } from "@solana/kit-plugin-wallet";
import { vaultProgram } from "@your-scope/vault-client";

export const client = createClient()
  .use(solanaDevnetRpc())
  .use(walletSigner())
  .use(vaultProgram());
```

Install list (verify at authoring — this course pins kit 7.0.0, checked 2026-07-27):

```
@solana/kit @solana/kit-plugin-rpc @solana/kit-plugin-wallet @solana/react @solana-program/system swr @tanstack/react-query
```

The last two are not optional decoration: `@solana/react` peer-depends on `swr` and `@tanstack/react-query`, and omitting them produces missing-module errors on the first install — the single most common setup failure.

## The stale trap this lesson exists to kill

The old stack — `ConnectionProvider`, `WalletProvider`, `WalletModalProvider`, `PhantomWalletAdapter`, `useWallet()` — is what most tutorials and most of your search results still teach. This app uses **none of it**. Wallet Standard is a browser protocol wallets implement directly, so discovery needs no per-wallet adapter package. If a code sample in this course reaches for `WalletProvider`, it is wrong.

## The surprise: a client is bound to one chain

A Kit client is bound to exactly one chain and one endpoint at build time. Switching clusters — devnet to mainnet — is not a config change you flip; it means **rebuilding the client**, which in React means a `useMemo` keyed on the endpoint. This is the first thing that surprises people, so the walkthrough builds it in from the start.

Read the annotated walkthrough next: it publishes the client with `ClientProvider`, reads it with `useClient()`, and renders a connect/disconnect control driven by the Wallet Standard hooks — keeping the loading / empty / error states that age well from the old `react-patterns` material, and rewriting every line of its code.
