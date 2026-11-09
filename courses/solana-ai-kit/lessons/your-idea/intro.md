# Your own idea: three threads to adapt

Stamp went from three lines to a devnet deploy through thirteen of the fifteen agents. This lesson shows **the same route on three other ideas**, each as a chain of agents with its first prompt, and ends with a template for yours. Reading and one piece of writing. Around fifteen minutes.

## Why this matters

The order Stamp followed is the kit's shape for any idea: research, design, plan, build, test, audit, the parts people touch, deploy, docs, review. What changes between ideas is **which agents get a turn and which pack they ask for**. Once you can write that chain for an idea in ten lines, you can start building the next morning, and the agents will tell you when you picked the wrong one, because every file carries a Handoffs section.

## The method, in one block

```text
1. idea.md, three lines: a person, what they cannot do today, what your thing does
2. solana-researcher: what exists, cited, dated, labelled
3. solana-architect (game-architect for a game): accounts, instructions, invariants, authority, risks → design.md
4. /plan-feature: phases, risks, tests, open questions → answer them in design.md
5. anchor-engineer: build from design.md, closes with /build-program, /test-rust, /audit-solana
6. solana-qa-engineer: coverage bar, negative tests, slop pass
7. the parts people touch: solana-frontend-engineer, mobile-engineer, rust-backend-engineer
8. the specialists an idea needs: token-engineer, defi-engineer, unity-engineer
9. /deploy devnet, with the approval stop
10. devops-engineer for CI, tech-docs-writer for the README, /product-review for a cold read
```

## Thread one: Vaquinha, a group gift pool

Friends pool SOL for a gift. The pool pays the organizer when the target is reached, or refunds everyone at a deadline.

- **solana-researcher**: what exists for group pools and escrow on Solana, and how each one handles the refund path. Expect Low labels on half of it.
- **solana-architect**: a pool PDA per gift holding target, deadline and organizer, a contribution PDA per contributor, three instructions, contribute, release, refund. The risk list starts with *who can call release and when*, and the check I would watch is the **deadline**, which the audit command wants read from the clock sysvar, never from an argument.
- **anchor-engineer, then solana-qa-engineer**: the boundary tests are the deadline minus one second and the target minus one lamport.
- **solana-frontend-engineer**: a page per pool with a progress bar and a contribute button, simulate first, confirmed not finalized.
- **defi-engineer, as a variant**: contributions in any token, converted to USDC through a Jupiter swap with the minimum output enforced on-chain.
- What you hold at the end: a pool on devnet and a page a friend can open.

First prompt to paste:

```text
Use the solana-researcher agent. What exists on Solana for group payment pools or crowdfunding escrow, how does each handle refunds after a deadline, and which ones shut down? Cite, date and label every finding.
```

## Thread two: Tool library, deposits on a neighbourhood shed

A shed lends drills and ladders. A borrower locks a deposit, and it comes back when the tool does.

- **solana-researcher**: deposit-and-return patterns, and who holds the deposit while the tool is out.
- **solana-architect**: an item PDA per tool, a loan PDA per active loan, instructions to lend, return and claim a deposit after a grace period. The question it will put to you is *who confirms the return*, and the honest design has the shed volunteer sign it.
- **token-engineer, as a variant**: a membership that cannot be sold, a Token-2022 mint with the non-transferable extension, checked before a loan is allowed.
- **mobile-engineer**: the volunteer scans a code at the shed door, so the app that matters is on a phone, Android first because that is where Mobile Wallet Adapter runs.
- **rust-backend-engineer**: a small indexer so the shed sees which tools are out and since when, idempotent on redelivery.
- What you hold at the end: lend and return working on devnet, and a members list.

First prompt to paste:

```text
Use the solana-architect agent. Design a tool-lending program: items, loans with a locked deposit, return confirmed by a volunteer, and a deposit claim after a grace period. Every account with owner, seeds, size and rent payer, every instruction with its signers, the invariants, the authority model, the open risks. Anchor unless measured otherwise. Write design.md.
```

## Thread three: Door list, tickets that cannot be scalped

A small event sells a fixed number of entries that cannot be resold above face value.

- **solana-researcher**: ticketing on Solana, and which projects tried to cap resale and what happened to them.
- **token-engineer**: this idea is its job. A ticket is a token, the rule lives in a **transfer-hook program** that checks each transfer, and the choice is made before the mint exists because extensions are fixed at creation. Its file warns that hooked mints are rejected by many venues, *which for a ticket that should not trade is a feature*.
- **token-engineer, again**: the hook program itself is in its lane too, with the hook interface discriminator its file names, and anchor-engineer builds the rest of the program around it.
- **solana-frontend-engineer**: the door page, scanning a wallet and showing a valid ticket, with the wallet connect deferred until the value is visible.
- **devops-engineer**: the one thread where a mainnet date is real, so the upgrade authority plan and the CI key rules from lesson 6 apply.
- What you hold at the end: a hooked mint on devnet and a door page that says yes or no.

First prompt to paste:

```text
Use the token-engineer agent. Tickets for a 200-person event as a Token-2022 mint that cannot be resold above face value. Propose the extension set, the transfer-hook rule, who holds each authority, and what a wallet shows the holder. Fewest extensions that do the job, and say which venues will reject the mint.
```

![Three idea threads as agent chains: Vaquinha through researcher, architect, anchor, QA, frontend and a DeFi variant, Tool library through researcher, architect, token, mobile and backend, Door list through researcher, token-engineer with a transfer hook, anchor, frontend and devops, all ending at /deploy devnet.](assets/v01-three-threads.webp)

## Write yours

Copy the block and fill it. Keep the three idea lines honest, and put a name on every step, because *an agent asked by name does the job its file describes, and an unnamed request goes wherever the description match lands*.

```text
<name>
<a person>, <what they cannot do today>, <what your thing does about it>

1. solana-researcher: <the one question you need answered first>
2. <architect>: <the two or three accounts you expect, in words>
3. anchor-engineer, solana-qa-engineer: <the boundary case a test must hit>
4. <the people-facing agent>: <the one screen that must exist>
5. <the specialist, if any>: <the pack it will ask for>
6. /deploy devnet, then tech-docs-writer and /product-review
```

When a backlog is real and you have a day to watch it, the kit also chains the steps for you: `/intake` reads issues into a work list, `/research`, `/decide` and `/plan-work` settle the plan, `/implement`, `/review-round` and `/fix-wave` build and check it, and **`/crew` runs those seven and stops at your gates**, always at the plan and always before shipping. `/ship` is the one step of that chain `/crew` never runs for you. I would start with the chain above by hand, and reach for `/crew` on the second project.

## Done when

- Your thread has three idea lines, a named agent per step and one pasted first prompt.
- You can say which agent owns a resale cap, a group refund and a volunteer-confirmed return.

## Watch out

- **A game starts with game-architect**, not solana-architect. The system architect's description says so and hands off.
- **Every specialist asks for a pack.** Budget the install into the first hour.
- **Deadlines come from the clock sysvar.** The audit checklist reads sysvars through the runtime, so a design that takes the time as an argument has a flaw it will find.

## The takeaway

One route, ten steps, **a named agent at each one**, and the only thing that changes between ideas is which specialists get a turn. *Write the chain before you write the prompt.*

## Next

The final lesson is about staying good at this after the workshop: the guide agent that teaches at your level, the command that explains code you did not write, and how the kit updates itself. It ends with a quiz over the whole course.
