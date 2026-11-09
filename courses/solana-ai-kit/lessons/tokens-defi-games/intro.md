# Tokens, DeFi and games: four more specialists

Last lesson three engineers built the card page, planned the phone app and designed the shop's indexer. This lesson asks three questions a real shop would ask next: **can the stamps be a token in the customer's wallet, can the customer pay in any coin, and can the card be a game?** Each question has an agent, and each agent's file carries the rule that stops the obvious mistake. Prompts and reading, no toolchain. Around fifteen minutes.

## Why this matters

These four agents cover the parts of the ecosystem that change often, and where a model's memory is often out of date: token extensions that were new last year, a swap API that moved to a new version, a game SDK with platform limits. **token-engineer** builds mints, Token-2022 extensions, transfer-hook programs, NFT collections and launch plans. **defi-engineer** integrates Jupiter, Kamino, Marginfi, Raydium, Orca, Meteora and the oracles, through SDK, API or CPI. **game-architect** and **unity-engineer** design and build Solana games. All four read protocol skills that the kit pins and installs on demand, and the DeFi engineer's file says it outright: *build from the protocol's skill, not from memory*.

## Do this

1. **Make the stamps a token.** Type inside Claude Code:

```text
Use the token-engineer agent. Stamp's stamps should appear in the customer's wallet as a token they cannot sell. Propose the mint for devnet: token program, the extension set, who holds each authority, and what a wallet will show. Fewest extensions that do the job.
```

The phrase *fewest extensions* is the agent's own first rule, quoted back. Its file explains why: **mint extensions are fixed at creation, and every extension narrows where the token can trade**, because pools, wallets and aggregators reject hooks, permanent delegates and default-frozen accounts. For stamps, expect a Token-2022 mint with the **non-transferable** extension and the **metadata extension**, the shop as mint authority behind a two-step transfer, and a note that a free coffee is a burn. Expect the gotcha too: the associated token account derives from the Token-2022 program id, and the classic id gives the wrong address.

If you ever launch a token people can trade, the same agent carries the **pre-launch checks**: allocations documented, team tokens vesting on-chain, every remaining authority on a Squads multisig or revoked, and *revoking is permanent, so it asks you first*. `/inspect-mint` decodes any mint's extensions and live authorities from raw bytes, which is how you check someone else's token before integrating it.

2. **Let the customer pay in any token.** Type:

```text
Use the defi-engineer agent. A customer wants to pay for coffee with whatever token they hold, and the shop wants USDC. Design the swap step with Jupiter: which API, how the shop is protected from a bad rate, and how you would test it. Design only.
```

Its file names the current API, **swap v2 with an API key**, and three checks that a quick integration skips: **enforce a minimum output on-chain**, then reload the destination account after the swap and check the balance delta rather than the quote. **Price from an oracle** with a feed id and a maximum age, never from a pool's spot reserves that a flash loan can move. And **test on a Surfpool mainnet fork**, so the real protocol accounts load. It will ask for the `jupiter` pack, and for `sendai` if lending or oracles come up.

![The token, DeFi and game lane: token-engineer builds the stamp mint with the fewest extensions, defi-engineer adds a Jupiter swap with an on-chain minimum output and oracle pricing tested on a Surfpool fork, and game-architect writes concept.md and plan.md for unity-engineer with payouts verified on-chain.](assets/v01-token-defi-game-lane.webp)

3. **Turn the card into a game.** Type:

```text
Use the game-architect agent. Stamp wants a small game: after each coffee the customer plays a ten-second round in the shop's app and a good score earns a bonus stamp. Decide what lives on-chain and what does not, pick the platform, and write concept.md and plan.md. Ask me what you need to know first.
```

The agent's file makes it ask about gameplay, platform and what must be on-chain before it writes anything, and then gives it the split: **on-chain for what is valuable, tradeable or trusted by other players**, off-chain for per-frame gameplay and session state, writes at checkpoints rather than per action. The sentence to look for in its answer is this one: **client-reported scores are attacker-controlled**, so the bonus stamp is paid only against a check the client cannot forge. It writes `concept.md` with numbers rather than adjectives and `plan.md` with numbered steps, and waits for your approval. **unity-engineer** then builds from those two files in small steps, closing each with `/build-unity` and `/test-dotnet`, and its file carries the Unity traps: main-thread-only objects, no C# threads on WebGL, textures that never get collected.

4. **Pick your variant.** The token, the swap and the game are three versions of Stamp you could build next. Write the one you would build under `design.md`, with the agent that owns it.

## Done when

- You hold a proposed extension set for the stamp token with an owner for each authority.
- You can say which three checks protect the shop in the swap design.
- `concept.md` and `plan.md` exist for the mini-game, with the on-chain split written down.

## Watch out

- **Each of these agents asks for a pack.** `metaplex`, `jupiter`, `sendai`, `solana-game`. The hub prints the install command, `/add-skill <id>` runs it, and `/update` keeps it.
- **Value-moving actions stop.** The token and DeFi files both say the security plugin gates mainnet, value-moving and authority actions, and that an ask or deny is your policy, reported rather than retried another way.
- **Transfer fees reduce what the recipient gets.** Credit the balance delta, not the amount argument. The token file says it, the DeFi file says it for swaps.
- **Mobile and PSG1 only when you ask.** The game architect defaults to desktop first and WebGL second.

## The takeaway

Four agents for four fast-moving parts of the ecosystem, and each file carries **the one rule that stops the obvious mistake**: fewest extensions, minimum output on-chain, oracle not spot, verify what pays out. *They build from pinned protocol skills and ask for the pack they need.*

## Next

Next lesson Stamp ships: a devnet deploy with the approval stop, a CI pipeline, docs that state the program id and every account layout, and a product review by someone who meets the app for the first time. The firewall shows up where it matters, at the deploy.
