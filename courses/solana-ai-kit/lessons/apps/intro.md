# Build around it: frontend, mobile and backend engineers

Stamp has a program with three instructions and a test suite. Nobody at the till can use it yet. This lesson three agents build the parts people touch: **a web page for the customer's card, a phone app, and a service that gives the shop a dashboard**. The web part runs on Node alone, so everyone follows at the keyboard. Around twenty-five minutes.

## Why this matters

The program is a small part of a product, and the kit knows it: six of the fifteen agents work *around* the program. Three of them are in this lesson. **solana-frontend-engineer** builds React and Next.js on `@solana/kit`, with wallet connection through Wallet Standard and a fixed list of transaction UX rules. **mobile-engineer** builds React Native and Expo apps on Mobile Wallet Adapter. **rust-backend-engineer** builds the services that read the chain and keep a database honest under retries and forks. Each one's file carries *the mistakes older habits get wrong*, and those lists are the real value, because they are the bugs you would otherwise ship.

## Do this

1. **Scaffold the web app.** Type inside Claude Code:

```text
/scaffold stamp-web, frontend only, with a Kit template that has no program
```

The command runs `create-solana-dapp` with a Kit template, makes the folder a git repository and installs the kit into it. Open Claude Code there when it finishes.

2. **Ask for the customer's card page.** Type:

```text
Use the solana-frontend-engineer agent. Build the customer page for Stamp: connect a wallet, show the customer's card for one shop with the stamp count and how many stamps remain until a free coffee, and a redeem button. Read the card from the Stamp program's card PDA on devnet. Until the program is deployed, read from a fixture file so the page works today. Follow your transaction UX rules for the redeem flow and run /build-app and /test-ts before you hand back.
```

Expect it to read the frontend reference in the solana-dev pack first, then build on **`@solana/kit` 8 with the wallet and RPC plugins and `@solana/react`**, with the legacy wallet-adapter packages kept out. The redeem flow should follow the rules in its file: **simulate before asking for a signature**, show awaiting signature, then submitted with the signature and an explorer link, then **confirmed**, and release the UI at confirmed rather than finalized. A rejected signature is a normal outcome, not an error. It closes by running `/build-app` and `/test-ts` and reporting components changed, wallets tested and accessibility gaps left open, because its bar is **WCAG 2.2 AA**.

3. **Generate the client instead of hand-writing calls.** Once the program exists, type:

```text
/generate-idl-client
```

It builds the IDL and renders a typed TypeScript client with Codama, so the page calls `redeem` through a function with real types, and program errors decode into messages. *Hand-built instructions are the thing this command exists to replace.*

![The around-the-program lane: solana-frontend-engineer builds the web page on Kit with the transaction UX rules, mobile-engineer builds the Expo app on Mobile Wallet Adapter, and rust-backend-engineer builds the webhook indexer and dashboard API, all reading the same generated client and design.md.](assets/v01-app-lane.webp)

4. **Describe the phone app, build it later.** Type:

```text
Use the mobile-engineer agent. Plan the Stamp customer app in Expo: connect through Mobile Wallet Adapter, show the card, redeem. Tell me which skill pack you need, what works on iOS, and what needs a development build. Plan only for now.
```

Its answer will ask for the `solana-mobile` pack and tell you three things from its own file that save an afternoon: **Mobile Wallet Adapter is Android only**, so iOS signs through a wallet's deep-link protocol or SDK. It **needs a development build**, because Expo Go cannot load it. And the Web Crypto polyfill must load first or signing fails before the wallet opens, in a way that looks like a wallet bug.

5. **Give the shop a dashboard.** Type:

```text
Use the rust-backend-engineer agent. Design a small Axum service for Stamp: consume Helius webhooks for the program's stamp and redeem events, keep a Postgres table of stamps per customer per shop, and serve the shop a dashboard endpoint. Tell me how you keep the table correct when a webhook is redelivered. Design only.
```

The answer should contain the rules its file calls the hard part: **deliveries repeat**, so upsert on the signature plus the event's position in the transaction. **Failed transactions are on-chain too**, so check the error field before applying effects. **Confirmed is fine for display and finalized for anything irreversible.** Store the slot on every row and backfill through the same idempotent path. It needs the Helius key from `.env`, which `/setup-mcp` sets.

## Done when

- `stamp-web` builds and shows a card from the fixture with a wallet connected on devnet.
- You can repeat the three transaction stages the redeem button must show.
- You hold a written plan for the mobile app and the indexer, each naming the pack or key it needs.

## Watch out

- **Public env values ship in the bundle.** `NEXT_PUBLIC_*` and `EXPO_PUBLIC_*` are visible to every user, so a keyed RPC URL goes behind a server route.
- **A blockhash lasts around a minute.** Rebroadcasting the same signed transaction is safe until then. After that, rebuild and ask for a new signature.
- **Transaction v1.** The client plans v1 by default, and a wallet that does not support it needs a v0 client or an upgrade prompt. The frontend file has the check.
- **The blocking RPC client stalls a Tokio runtime.** The backend file names the non-blocking one. An older example does not.

## The takeaway

Three engineers build the parts people touch, each with **a list of the traps in its own file**: confirmed not finalized on the web, Android-only wallet adapter on mobile, idempotent upserts in the indexer. *The generated client is what keeps all three talking to the same program.*

## Next

Next lesson Stamp's stamps become a token, a customer pays with any coin, and the card turns into a game, which is the token engineer, the DeFi engineer and the two game agents. Mostly reading and prompts, no toolchain needed.
