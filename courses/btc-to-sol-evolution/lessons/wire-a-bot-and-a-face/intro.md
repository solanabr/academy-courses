# Sign it yourself: a bot for the vault, then a button

You closed the last module with a vault deployed and green under `anchor test`: a real PDA vault whose only caller, so far, has been your own test file. That was the point of the exercise, and it is also the problem. Your vault is deployed and it does nothing. The only thing that has ever called it is a test file, and test files don't ship. A real deposit needs something to build the transaction, sign it, and put it on the wire, and right now that something does not exist, so the vault just sits there holding zero.

So build the something. Everything happens inside the `toolkit/vault` workspace you already have, because that is where the compiler left your program's interface. Four short steps, and then you run a bot.

**One: give the workspace a Node side.** It has none yet; the Rust tests never needed one.

```bash
cd toolkit/vault
npm init -y && npm pkg set type=module
npm install @solana/kit
npm install -D tsx typescript @types/node codama @codama/nodes-from-anchor @codama/renderers-js@^1
```

Two pins in that line are load-bearing. `type=module` is what lets these files use top-level `await`, which every one of them does. And `@codama/renderers-js@^1` is deliberate: the 2.x line renders a whole publishable npm package instead of a plain folder, which moves every generated file down two directories and breaks the `./generated` import below. Pin the major or read a `Cannot find module` error for twenty minutes.

**Two: point TypeScript at the folder.** Save this as `tsconfig.json` next to `package.json`:

```json
{
  "compilerOptions": {
    "target": "es2022",
    "module": "preserve",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "types": ["node"]
  },
  "include": ["bot/**/*.ts"]
}
```

**Three: generate the client.** Save this as `codama.mjs` and run it; we take it apart in a minute.

```javascript
// codama.mjs - run once after every `anchor build`
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { createFromRoot } from "codama";
import { renderVisitor } from "@codama/renderers-js";
import { readFileSync } from "node:fs";

const idl = JSON.parse(readFileSync("./target/idl/vault.json", "utf-8"));
const codama = createFromRoot(rootNodeFromAnchor(idl));
codama.accept(renderVisitor("./bot/generated"));
```

```bash
node codama.mjs
```

**Four: give the bot a key and a cluster.** Your vault is deployed on devnet from last lesson, and the wallet that deployed it is already funded, so reuse it rather than minting a stranger:

```bash
mkdir -p state
cp "$(solana config get keypair | cut -d' ' -f3-)" state/sol.key
solana address -k state/sol.key      # same address you airdropped to last lesson
```

Now the bot itself. Two files. Save the first as `bot/init.ts`: your vault's record account has to exist before anything can deposit into it, and on devnet nothing has created it yet.

```typescript
// bot/init.ts - create this owner's vault record once, before any deposit.
import { readFileSync } from "node:fs";
import {
  createSolanaRpc, createSolanaRpcSubscriptions, sendAndConfirmTransactionFactory,
  createKeyPairSignerFromBytes, getSignatureFromTransaction, pipe, createTransactionMessage,
  setTransactionMessageFeePayerSigner, setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction, signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
} from "@solana/kit";
import { getInitializeInstructionAsync, findVaultStatePda } from "./generated";

const RPC_HTTP = process.env.SOLANA_RPC ?? "https://api.devnet.solana.com";
const RPC_WS = process.env.SOLANA_WS ?? "wss://api.devnet.solana.com";

const rpc = createSolanaRpc(RPC_HTTP);
const rpcSubscriptions = createSolanaRpcSubscriptions(RPC_WS);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

const secret = new Uint8Array(JSON.parse(readFileSync("state/sol.key", "utf8")));
const owner = await createKeyPairSignerFromBytes(secret);

const [vaultState] = await findVaultStatePda({ owner: owner.address });
const { value: existing } = await rpc.getAccountInfo(vaultState).send();
if (existing) {
  console.log("vault already initialized at", vaultState);
  process.exit(0);
}

const ix = await getInitializeInstructionAsync({ owner });
const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
const message = pipe(
  createTransactionMessage({ version: 0 }),
  (tx) => setTransactionMessageFeePayerSigner(owner, tx),
  (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
  (tx) => appendTransactionMessageInstruction(ix, tx),
);
const signed = await signTransactionMessageWithSigners(message);
assertIsTransactionWithBlockhashLifetime(signed);
await sendAndConfirm(signed, { commitment: "confirmed" });
console.log("initialized:", getSignatureFromTransaction(signed));
```

Save the second as `bot/deposit.ts`. Don't read it yet.

```typescript
import { readFileSync } from "node:fs";
import {
  createSolanaRpc, createSolanaRpcSubscriptions, sendAndConfirmTransactionFactory,
  createKeyPairSignerFromBytes, getSignatureFromTransaction, pipe, createTransactionMessage,
  setTransactionMessageFeePayerSigner, setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction, signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
} from "@solana/kit";
import { getDepositInstructionAsync, fetchVaultState, findVaultStatePda } from "./generated";

const RPC_HTTP = process.env.SOLANA_RPC ?? "https://api.devnet.solana.com";
const RPC_WS = process.env.SOLANA_WS ?? "wss://api.devnet.solana.com";

const rpc = createSolanaRpc(RPC_HTTP);
const rpcSubscriptions = createSolanaRpcSubscriptions(RPC_WS);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

const secret = new Uint8Array(JSON.parse(readFileSync("state/sol.key", "utf8")));
const owner = await createKeyPairSignerFromBytes(secret);

const [vaultState] = await findVaultStatePda({ owner: owner.address });
const before = await fetchVaultState(rpc, vaultState);
console.log("vault balance before:", before.data.balance);

const ix = await getDepositInstructionAsync({ owner, amount: 100_000_000n });

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
const message = pipe(
  createTransactionMessage({ version: 0 }),
  (tx) => setTransactionMessageFeePayerSigner(owner, tx),
  (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
  (tx) => appendTransactionMessageInstruction(ix, tx),
);
const signed = await signTransactionMessageWithSigners(message);
assertIsTransactionWithBlockhashLifetime(signed);
await sendAndConfirm(signed, { commitment: "confirmed" });
console.log("sent:", getSignatureFromTransaction(signed));

const after = await fetchVaultState(rpc, vaultState);
console.log("vault balance after:", after.data.balance);
```

Run them:

```bash
npx tsx bot/init.ts
npx tsx bot/deposit.ts
```

```
initialized: 5tioLDkv5Bc...eckwz8j (yours will differ)
vault balance before: 0n
sent: 5WXK1eRSKMX...osJudB (yours will differ)
vault balance after: 100000000n
```

(If devnet is slow or the faucet has left you short, every one of these commands takes a local validator instead: run `solana-test-validator` in another terminal, `solana -u localhost program deploy target/deploy/vault.so --program-id target/deploy/vault-keypair.json`, then prefix the bot with `SOLANA_RPC=http://127.0.0.1:8899 SOLANA_WS=ws://127.0.0.1:8900`. The two environment variables at the top of each file exist for exactly that.)

The last line is the vault's balance in lamports (the smallest unit of SOL, a billionth of one), and it went up by exactly the 0.1 SOL the bot deposited. The trailing `n` is JavaScript telling you it is a `BigInt`, not an ordinary number: lamport counts overflow a JavaScript float, so kit hands them to you in the one type that cannot silently round them. The line above it is your receipt from the network. Nothing about that receipt was faked: re-run `bot/deposit.ts` and the balance climbs again, from 100000000n to 200000000n, because a second, separate read confirmed it against the chain, not against a variable in memory. You just did the thing test files pretend to do, from a program that will still be running when the test harness is long gone.

![A headless script prints the vault balance before, a transaction receipt, and the balance after, which has risen by the deposited amount.](assets/v01-annotated-code.webp)

## Read the bot you just ran

You ran the thing before you read it. Now read it, top to bottom, because every line is a named idea you will reuse for the rest of this course.

**A 30-second sidebar for absolute beginners: how to read TypeScript.** You do not need to know TypeScript to follow this bot; read it the way you read a recipe. An `import` line at the top pulls a named tool in from another file, the way you would fetch a whisk from a drawer before you start cooking. A line like `const name = value` gives a value a name so you can call it back later. Any line that starts with `await` is a step that talks to the network, so the word just means "wait right here until the chain answers before running the next line." The bits with a colon are labels that tell your editor what shape a value should be, so it can underline a typo in red before you ever hit the network; they do nothing when the code actually runs. That is the entire vocabulary you need here. You are reading these lines, not inventing them: you pasted every one of them a page ago.

The client never wrote itself, and it never hand-wrote your program's shape either. It was generated, by the `codama.mjs` from step three. Last module `anchor build` emitted a file to `target/idl/vault.json`, the **IDL** (Interface Description Language: the generated contract Anchor writes down). It lists every instruction your program exposes, every argument each one takes, and every account each one touches. `rootNodeFromAnchor` parses that JSON into Codama's own description of a program, and `renderVisitor` walks that description and writes TypeScript out of it.

That is why `node codama.mjs` made a `bot/generated/` folder appear, holding a typed function for every instruction, a decoder for every account, and a helper for every PDA. `codama` reads the IDL and writes the client so you never do. Change the program, rebuild, regenerate, and the client rewrites itself. That is why you never hand-write an ABI here (the Application Binary Interface an Ethereum client has to maintain by hand, and keep in sync by hand, and get subtly wrong by hand). The build step is the source of truth, and the generated client is a reader of it, not a second author who can disagree.

![The Rust program compiles through anchor build into an IDL, which Codama renders into a generated client the bot imports, so the interface is generated, not hand-written.](assets/v02-flowchart.webp)

## One shape for every transaction

Scroll back to `bot/deposit.ts` and read it once more, because the shape it teaches is the shape of every Solana transaction you will ever send from a client. There is exactly one path: build a message, sign it, send it.

Start at the top. `createSolanaRpc` opens a plain HTTP connection to a node, the same JSON-RPC front door you built by hand back in module 2, now typed. `createSolanaRpcSubscriptions` opens the websocket twin of it, which the send helper uses to hear when your transaction confirms. `createKeyPairSignerFromBytes` loads a 64-byte keypair off disk and hands back a **signer**: an object that holds a key and knows how to sign. That signer, `owner`, is the pen for this whole script.

Now the instruction. `getDepositInstructionAsync` is one of the functions `codama` generated from your IDL, and the `Async` in the name is doing real work. You hand it only `{ owner, amount }`, and it derives the vault's two PDAs for you, off-chain, from the same seeds the program declared, then packs the discriminator, the `amount`, and the account list into a single instruction. You never spell out the vault address. The generated helper recomputes it, deterministically, exactly the way the on-chain program will.

Then the part that is identical for every transaction on Solana. `createTransactionMessage({ version: 0 })` starts an empty versioned message, and `pipe` threads it through three edits in order: name who pays the fee and signs (`setTransactionMessageFeePayerSigner`, handed the `owner` signer), stamp it with a recent blockhash so the network can date it (`setTransactionMessageLifetimeUsingBlockhash`, from `rpc.getLatestBlockhash()`), and drop your one instruction in (`appendTransactionMessageInstruction`). `pipe` is just left-to-right function application: each line takes the message so far and returns the next version of it, so you read the transaction being assembled top to bottom.

![An annotation of the kit deposit pipeline mapping each call to its job, grouped into build, sign, and send stages.](assets/v03-annotated-code.webp)

`signTransactionMessageWithSigners` is the payoff for attaching the signer to the message earlier. It walks the message, finds every account that must sign, and asks each attached signer to do it. Here that is just `owner`, so one signature goes on. This is the seam the whole lesson turns on, so mark it: the transaction does not care *which kind* of pen signed it. A key loaded off disk and a browser wallet both produce a signer, and both slot into this exact line. Swap the signer and every other line stays the same.

`assertIsTransactionWithBlockhashLifetime` is a one-line safety check that narrows the signed transaction to the blockhash-based kind the sender expects; without it the types stay loose and the next line won't accept the transaction. Then `sendAndConfirm`, the function you built from `sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })`, does the two jobs its name spells out. It sends the signed transaction, and then it blocks, listening on the websocket, and does not return until the cluster reports the transaction reached the commitment level you asked for.

## How sure is "sent"

That commitment level is `"confirmed"`, and it is worth a full paragraph because it is the dial between speed and certainty. Commitment is Solana's answer to how sure you want to be that a transaction is real, and it has three rungs. `"processed"` means a single validator has seen it and it might still be dropped. `"confirmed"` means validators representing a supermajority of stake, more than two-thirds, have voted on the block that holds your transaction, which is strong enough for almost every client. `"finalized"` means the block is buried dozens deep and is practically irreversible, at the cost of a few more seconds. The bot asks for `"confirmed"`, so when that line returns, you know the deposit landed, not merely that you fired it into the dark.

![A table of the three Solana commitment levels, processed, confirmed, and finalized, describing what each guarantees and how strong it is, with confirmed marked as the bot's choice.](assets/v04-table.webp)

One thing it does *not* do is hand you a receipt. `sendAndConfirm` returns `Promise<void>` — it tells you the transaction landed by returning at all, and it tells you it failed by throwing. That surprises people, because the receipt is what you want to print. Kit's answer is that you already have it: a signed transaction *contains* its own signature, so `getSignatureFromTransaction(signed)` reads it back out with no extra network call. That is the line printing `sent:` in the bot, and reaching for the return value of `sendAndConfirm` instead is how you end up logging `sent: undefined`.

That base-58 string is your **transaction signature** (the identifier that pins your exact transaction on-chain: your permanent receipt). Paste it into a block explorer, Solana Explorer or Solscan, and you can pull up the whole transaction: which program ran, which accounts changed and by how much, the compute it burned, the fee it paid. In the bot it is proof for you; in a frontend you turn it into a clickable link so a user can watch their own deposit settle.

That is the whole `bot/deposit.ts`: open an RPC, load a signer, build the instruction from the generated client, thread the message through `pipe`, sign, send. Nothing hidden, and no second path. There is no shortcut door and no verbose door, the way older Solana clients split into a one-line convenience call and a build-it-yourself call. Kit gives you one pipeline, and the only thing you ever vary is the signer.

## The pin that actually matters

Look back at the imports. Every primitive, the RPC, the signer, the message builders, the send helper, came from `@solana/kit`. Kit is the modern Solana SDK (formerly web3.js v2): tree-shakable, typed, built on native Web Crypto, and the one you reach for on anything new. Notice which package the install line pinned, though, because it is not that one.

Kit is unpinned here on purpose: `npm install @solana/kit` takes the current major and the rest of this stack agrees with it. The pin went on the *renderer*, `@codama/renderers-js@^1`, and it is the pin that saves your evening. Codama's 2.x renderer changed what it produces: instead of writing a plain folder of TypeScript, it writes a whole publishable npm package, complete with its own `package.json` and every source file moved down into `src/generated/`. The `import … from "./generated"` in your bot then resolves to that package's `main`, which points at a file the renderer does not create, and you get a module-resolution error with nothing obviously wrong on screen. The `^1` line still emits the flat folder these two bots expect.

Take the general rule with you, because it outlives every version number here: **pin to what your own dependencies say they need, and ask them rather than guessing.** `npm view @solana/react peerDependencies` will tell you, today, which kit major the React helpers require; `npm view @codama/renderers-js version` will tell you which renderer major `latest` currently means. Do that check per workspace, not once per career. Kit and the client generators have already been out of step once, in 2025, when generated clients required kit v6 while kit itself had shipped v7 and installing them together produced a flat peer-dependency error. That particular window has closed, but the way you detect the next one is the same two commands.

![A comparison showing the version fact that matters is the Codama renderer major: the 1.x renderer emits a flat generated folder the bot can import, while 2.x emits a package scaffold whose main entry does not resolve.](assets/v05-comparison.webp)

## The face: a button that signs

The bot proves the vault works. Nobody but you can drive it, because it needs your keypair on disk. A person with a browser wallet has to be able to walk up and deposit, and that means a frontend. The whole trick of the frontend is a single substitution: everywhere the bot loaded a signer off disk, the browser hands you a signer backed by the user's wallet instead. Every other line you already wrote stays.

This half needs a React app, which is outside what this lesson builds line by line; scaffold one however you normally would, copy `bot/generated/` into it, and install the browser-side packages beside kit:

```bash
npm install @solana/kit @solana/react @wallet-standard/react swr @tanstack/react-query
```

Point it at devnet, the same cluster your bot just deposited to, so the button and the bot are moving the same balance rather than two lookalikes on two networks.

Wallets announce themselves to a page through a browser standard called **wallet-standard**, and `@wallet-standard/react` gives you two hooks to reach them: `useWallets` lists every wallet the browser found, and `useConnect` opens one and returns the accounts the user approved. That is the entire connect flow, and you render it however you like:

```tsx
import { useState } from "react";
import { useWallets, useConnect } from "@wallet-standard/react";

function ConnectButton({ wallet, onAccount }) {
  const [isConnecting, connect] = useConnect(wallet);
  return (
    <button disabled={isConnecting} onClick={async () => {
      const accounts = await connect();
      if (accounts[0]) onAccount(accounts[0]);
    }}>
      Connect {wallet.name}
    </button>
  );
}

export function App() {
  const wallets = useWallets();
  const [account, setAccount] = useState(null);
  return (
    <div>
      {wallets.map((w) => <ConnectButton key={w.name} wallet={w} onAccount={setAccount} />)}
      {account && <DepositButton account={account} />}
    </div>
  );
}
```

Once the user picks a wallet and approves, you hold a `UiWalletAccount`, and that is the bridge to signing. `@solana/react` turns it into a kit signer with one hook, `useWalletAccountTransactionSendingSigner`, and from there the deposit is the bot's pipeline with `owner` swapped for the wallet's signer:

```tsx
import { useWalletAccountTransactionSendingSigner } from "@solana/react";
import {
  createSolanaRpc, pipe, createTransactionMessage,
  setTransactionMessageFeePayerSigner, setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction, signAndSendTransactionMessageWithSigners,
} from "@solana/kit";
import { getDepositInstructionAsync } from "./generated";

function DepositButton({ account }) {
  const signer = useWalletAccountTransactionSendingSigner(account, "solana:devnet");

  async function onDeposit() {
    const rpc = createSolanaRpc("https://api.devnet.solana.com");
    const ix = await getDepositInstructionAsync({ owner: signer, amount: 100_000_000n });
    const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
    const message = pipe(
      createTransactionMessage({ version: 0 }),
      (tx) => setTransactionMessageFeePayerSigner(signer, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) => appendTransactionMessageInstruction(ix, tx),
    );
    const signature = await signAndSendTransactionMessageWithSigners(message);
    console.log("sent", signature);
  }

  return <button onClick={onDeposit}>Deposit 0.1 SOL</button>;
}
```

Put the two files side by side and the point lands on its own. The `getDepositInstructionAsync` line is identical. The `pipe` block is identical. The one signer produced by `useWalletAccountTransactionSendingSigner` is passed to `setTransactionMessageFeePayerSigner` exactly where the bot passed `owner`, and it is the same signer the deposit instruction names as its `owner` account, because the connected wallet is now the vault's owner. The only real difference is the send call: the bot's local key signs and then a separate helper sends, while the wallet's signer signs and sends in one motion (`signAndSendTransactionMessageWithSigners`), because the wallet is the one holding the connection to the network. Same pipeline, a different pen.

![A single shared transaction pipeline fed by two signers, a disk keypair for the bot and a wallet signer for the button, showing the signer is the only difference.](assets/v06-diagram.webp)

## The bot grows an ear: listen instead of poll

The bot can push now. It cannot hear. It fires a deposit and forgets, blind to whether the balance actually moved or whether someone else touched the vault a second later. Back in module 2, the Bitcoin watcher fought the same blindness the only way Bitcoin allows: it sat in a loop, asked the node "anything new?" on a timer, and scanned each answer. That is polling. Ask, wait, ask again, forever, and your news is only ever as fresh as your last trip around the loop.

Kit lets you turn that inside out with the subscriptions connection you already opened. Instead of asking on a timer, you subscribe once and the node pushes the new bytes to you the instant the account changes:

```typescript
import { fetchVaultState, findVaultStatePda } from "./generated";

const [vaultStateAddr] = await findVaultStatePda({ owner: owner.address });

const notifications = await rpcSubscriptions
  .accountNotifications(vaultStateAddr, { commitment: "confirmed" })
  .subscribe({ abortSignal: AbortSignal.timeout(60_000) });

for await (const notification of notifications) {
  const state = await fetchVaultState(rpc, vaultStateAddr);
  console.log("vault moved, balance now:", state.data.balance);
}
```

`accountNotifications` takes the account you care about, here the vault's record PDA that `findVaultStatePda` derived, and `subscribe` hands back an async stream. `for await` then blocks on that stream, waking your code only when the chain has something to say. When it does, `fetchVaultState` decodes the account through the very generated codec the client has been using all along, turning raw bytes back into a typed object with a real `balance`, exactly the way the deposit read it, except this time you never asked. The chain volunteered it. That trailing `"confirmed"` is the same commitment rung from earlier, so you react to what is real, not to a maybe that might still get dropped, and the `abortSignal` is your off switch: when it fires the stream closes and the loop ends, instead of leaking an open socket for the life of the process.

![A two-column comparison of the Bitcoin polling watcher against kit's accountNotifications subscription, showing polling asks on a timer while a subscription is pushed the new bytes.](assets/v07-comparison.webp)

## Name the cost

Every tool in this course gets its bill read out loud, and kit's is two clauses, both real.

The first is verbosity. That `pipe` block is four lines to do what an older typed client could collapse into a single chained call that built, signed, and sent all at once. Kit made a deliberate trade: nothing is hidden, every step is a named function you can inspect, reorder, or swap, and in return you write the assembly out by hand every time. On a script that sends one instruction, that reads like extra typing. On a bot that needs to attach a compute-budget instruction, batch three deposits into one transaction, or sign with one key and pay fees with another, the explicit pipeline is the only thing that makes those possible, because there is a seam at every step to reach into. You pay a few lines on the simple case to keep the hard case reachable.

The second is the version fragmentation you already met. Kit is one package in a constellation — the generated client, the React helpers, the program clients — that ships on independent release trains, and any of them can move a major ahead of the others for a while. It happened in 2025 with kit v7 and the v6-era generated clients; it happened again with the Codama renderer's 2.x layout change that your `^1` pin is holding back. That is the toll of being early: the pieces are real, they work, and they do not all move in lockstep. The discipline is not a magic version number, it is a habit — read `peerDependencies` before you install, pin per workspace rather than per career, and write the date next to any version you hardcode in a document.

## Finish the bot, then the button

The bot deposits. The withdraw path is yours to close, and it hides one honest wrinkle worth meeting now. Open `bot/withdraw.ts` and reach for the generated builder, but notice it is `getWithdrawInstruction`, not `getWithdrawInstructionAsync`. Deposit got an `Async` builder that derived its PDAs for you because your `Deposit` accounts are seeded from the owner, which the client can recompute. Withdraw's record account is validated a different way in the program, so `codama` cannot derive it blind, and the sync builder asks you to pass the two accounts yourself.

```typescript
// bot/withdraw.ts - your completion
import { getWithdrawInstruction, findVaultStatePda, findVaultPda } from "./generated";

const [vaultState] = await findVaultStatePda({ owner: owner.address });
const [vault] = await findVaultPda(/* TODO(you): the seeds findVaultPda asks for */);

const ix = getWithdrawInstruction({ authority: owner, vaultState, vault, amount });
// then the same pipe -> signTransactionMessageWithSigners -> sendAndConfirm you already wrote
```

Derive the two accounts with the generated helpers, hand `getWithdrawInstruction` the `vaultState`, the `vault`, the `authority` signer, and the `amount`, then thread it through the same `pipe`, `sign`, `send` you already know. The lesson that wrinkle teaches is real: an async builder is a convenience the client can only offer when it can recompute every account, and when it can't, you supply what it cannot.

Then the harder one, unaided. Add a Withdraw button to the frontend, wired the same way the Deposit button is: the same `useWalletAccountTransactionSendingSigner`, the same `pipe`, `signAndSendTransactionMessageWithSigners`, pointed at your withdraw instruction instead of deposit, with the same before-and-after read so you can watch the balance fall.

You are done with this lesson when three things are true, and you prove each one by re-fetching the on-chain vault account and reading the changed balance, never by trusting a `console.log`. First: your headless bot deposits, returns a confirmed base-58 signature, and a follow-up `fetchVaultState` shows the vault balance risen. Second: your bot withdraws, returns a second confirmed signature, and the balance falls. Third: your browser button completes a deposit signed by a connected wallet, and a re-fetch of the vault account shows the effect. Balance up, balance down, and a wallet-signed deposit whose result you can read off the chain. That is the whole gate.

Say the answer to one question out loud before you move on, one sentence: the bot and the button send the same deposit; what single thing differs between them? A good answer lands on the signer. The bot loads a keypair off disk with `createKeyPairSignerFromBytes` and the button gets one from the wallet through `useWalletAccountTransactionSendingSigner`, and every other line, the generated instruction, the `pipe`, the signing, is identical. If your sentence names "who holds the pen," you have the one idea that runs under this entire lesson.

One caveat before you call the third gate passed: the browser wallet signs with *its own* key, not the one in `state/sol.key`, and your vault's seeds are derived from the owner. So the connected wallet gets its own vault, and it needs its own `initialize` before its first deposit — either run `bot/init.ts` with that key, or have the Deposit button send an initialize first when `fetchVaultState` comes back empty. Point the browser wallet at the same devnet keypair you gave the bot and both drive one balance; point it at a different wallet and you have two vaults from one program, which is the per-owner PDA design working exactly as specified.

This bot and this button both speak to the same deployed program on the same cluster your test file used to poke, and now the toolkit that started as a Bitcoin-RPC script drives a Solana program and listens to it too, one rung closer to the cross-chain ops bot. Your bot is no longer deaf: it pushes a transaction, and it hears the chain answer back when the vault moves. But it still lives on one side of a wall. The Bitcoin watcher from module 2 knows only Bitcoin, and this Solana bot knows only Solana, and neither has ever heard of the other. Next they finally report to a single brain, and the two lonely scripts become one cross-chain ops bot that watches both chains at once.
