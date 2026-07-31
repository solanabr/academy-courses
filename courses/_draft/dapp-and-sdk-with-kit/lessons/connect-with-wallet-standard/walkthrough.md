# Annotated Walkthrough: Connect / Disconnect

Read each step and the note under it. You will build this for real when you deploy in lesson 7; here the goal is to recognize every piece.

## 1. Publish the client to the tree

```tsx
import { ClientProvider } from "@solana/react";
import { client } from "./client";

export function App() {
  return (
    <ClientProvider client={client}>
      <WalletBar />
    </ClientProvider>
  );
}
```

`ClientProvider` puts the one module-scope client on the context. Every component below reads the same instance — you never build a second one per render.

## 2. Rebuild only when the endpoint changes

```tsx
import { useMemo } from "react";
import { createClient } from "@solana/kit";
import { solanaDevnetRpc } from "@solana/kit-plugin-rpc";

function useVaultClient(endpoint: string) {
  // Keyed on the endpoint: switching devnet -> mainnet rebuilds the client,
  // because a Kit client is bound to one chain and cannot be re-pointed.
  return useMemo(
    () => createClient().use(solanaDevnetRpc(endpoint)).use(walletSigner()).use(vaultProgram()),
    [endpoint]
  );
}
```

The `useMemo` dependency is the whole point: nothing rebuilds on a normal render, but a cluster switch does.

## 3. Discover, connect, disconnect

```tsx
import { useWallets, useConnect, useConnectedWallet, useDisconnect } from "@solana/kit-plugin-wallet/react";

function WalletBar() {
  const wallets = useWallets();          // whatever the visitor actually has installed
  const connect = useConnect();
  const connected = useConnectedWallet(); // undefined until one is connected
  const disconnect = useDisconnect();

  if (connected) {
    return (
      <button onClick={() => disconnect()}>
        Disconnect {connected.accounts[0]?.address.slice(0, 4)}…
      </button>
    );
  }

  if (wallets.length === 0) {
    // The EMPTY state — no wallet installed. Do not render a dead button.
    return <p>No Solana wallet found. Install one to continue.</p>;
  }

  return (
    <>
      {wallets.map((w) => (
        <button key={w.name} onClick={() => connect(w)}>Connect {w.name}</button>
      ))}
    </>
  );
}
```

`useWallets()` returns whatever Wallet Standard wallets the browser advertises — no adapter package per wallet, no hardcoded Phantom. The three states from `react-patterns` survive as concepts: **connected** (show identity + disconnect), **empty** (no wallet — a real state, not an error), and the **error** state you add when a connect attempt is rejected.

## What is deliberately absent

No `ConnectionProvider`. No `WalletProvider`. No `PhantomWalletAdapter`. No `useWallet()`. If you paste any of them in from a tutorial, you are on the old stack, and it will not compose with the Kit client you built.
