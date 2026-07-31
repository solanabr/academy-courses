# Send, Watch, and Read the Failure

Assembling was the pure part. Sending is where the network answers back — and the answer is not a boolean.

## Confirmation is a ladder, not a light switch

In Kit you send-and-confirm with a factory bound once to your RPC and its websocket side:

```ts
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  sendAndConfirmTransactionFactory,
  getSignatureFromTransaction,
} from "@solana/kit";

const rpc = createSolanaRpc("https://api.devnet.solana.com");
const rpcSubscriptions = createSolanaRpcSubscriptions(
  "wss://api.devnet.solana.com"
);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

await sendAndConfirm(signedTransaction, { commitment: "confirmed" });
const signature = getSignatureFromTransaction(signedTransaction);
```

What you are waiting for is a **commitment level**, and each rung means something different:

- **`processed`** — one validator has executed it. Fast, and still reversible.
- **`confirmed`** — a supermajority of the cluster has voted on its block. The practical default: what "it went through" honestly means for a UI.
- **`finalized`** — locked in beyond rollback. What you wait for before acting on money you cannot claw back.

An honest UI shows the rung it has actually reached — "sending" is not "confirmed", and "confirmed" is not a guess made at the moment you hit send.

## The failure taxonomy

Five ways this deposit can not-happen, and they are not the same event:

1. **You reject in the wallet.** Nothing reached the network. Not a failed transaction — the honest UI state is back-to-resting, not an error banner.
2. **The faucet declines.** Also pre-network; your wallet simply is not funded yet. Retry after a wait.
3. **The blockhash expires.** The transaction aged out (~a minute) before landing — lesson 4's lifetime rule. It can never land now, which is precisely what makes the fix safe: rebuild with a fresh blockhash, re-sign, resend.
4. **Simulation rejects it pre-flight.** The RPC ran it before broadcast and it failed — insufficient lamports, a program error. Nothing was sent; read the message, fix the cause.
5. **The program says no, on-chain.** The transaction landed and *failed*, fee paid. This program's refusals are named in the IDL you read in lesson 1 — depositing `0` fails with error `6002 ZeroAmount`. A landed failure has a signature and logs; that is where you look.

The through-line: **"failed" is only honest for 3, 4 and 5.** Rejecting a wallet prompt did not fail — nothing happened, and an app that shows a red X for it is lying to its user.

## Your receipt

Once confirmed, your signature is a permanent public record. `https://explorer.solana.com/tx/<signature>?cluster=devnet` shows what you paid (compare it to your lesson-4 prediction — for real this time), the accounts your instruction touched — vault first — and the vault's balance change. That link is the first artifact of this course anyone else can verify. Keep it; the reflection below asks for it, and your profile keeps it after that.
