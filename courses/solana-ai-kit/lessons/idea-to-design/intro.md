# From an idea to a design

Last lesson the researcher answered one question by name. This lesson two agents take Stamp from **three lines of idea to a design an engineer can build from**, and you learn what each of them owes you when it finishes. Around twenty minutes, and nothing here needs a toolchain.

## Why this matters

Most of the hackathon projects I have watched fail lost their month in the first week, with the wrong account model and a copied idea. The kit puts two specialists at that spot. **solana-researcher** answers ecosystem questions with cited, dated sources and a confidence label on every finding, because the ecosystem moves faster than any model's memory. **solana-architect** settles accounts, PDAs, program boundaries, signers and the trust model *before* code exists, so the engineers never reopen them. Games have their own architect, **game-architect**, which writes a concept file and a plan file and decides what lives on-chain. The output of this lesson is a file the rest of the course reads.

## Do this

1. **Write the idea in three lines.** Create `idea.md` in the project with a person, the thing they cannot do today, and what Stamp does about it. Ours:

```text
Stamp
Rita runs a coffee shop and gives a free coffee every tenth purchase, tracked on paper cards that customers lose.
Stamp keeps the card on Solana: the shop stamps a customer's wallet at the till, ten stamps redeem one coffee, and the customer sees their card in any wallet app.
```

2. **Ask the researcher what exists.** Type inside Claude Code:

```text
Use the solana-researcher agent. What exists on Solana today for small-business loyalty cards or punch cards, including projects that shut down and why? Cite a source for each finding with a date and a confidence label, and end with what you could not verify.
```

Its file fixes the shape of the answer: a **two or three sentence answer first**, then findings each with a link and a label, **High** when several authoritative sources agree, **Medium** for one, **Low** for community sources only, **Speculative** for an inference, then a recommendation and a list of what it could not verify. Read the Low items as leads, not facts. *The agent is allowed to say it found nothing solid, and that answer is worth having before you build.*

3. **Hand the idea to the architect.** Type:

```text
Use the solana-architect agent to design Stamp from idea.md. I want every account with its owner, seeds, size, rent payer and who can change or close it, every instruction with its signers, the invariants, the authority model and the open risks. Default to Anchor unless you can show a measured reason for Pinocchio. Write the result to design.md.
```

That list is the architect's own **Output** section, quoted back at it, which is a trick worth keeping: *an agent asked for exactly what its file promises rarely leaves anything out.* Expect something like this for Stamp:

- A **shop account**, a PDA from a fixed prefix and the shop's wallet, holding the stamps-per-reward number and the shop authority.
- A **card account per customer**, a PDA from the prefix, the shop and the customer, holding the stamp count and the redeemed count. One account per customer, because the architect's file says shared writable accounts serialize every transaction that touches them, and two customers at the till at once should not wait on each other.
- Three instructions: **create shop** (the shop signs), **stamp** (the shop signs, the customer's card is written), **redeem** (the customer signs, ten stamps leave the card). The architect will ask who pays rent for a new card, and *the honest answer for a coffee shop is the shop.*
- Invariants such as stamps never go negative and a redeem always costs exactly the configured number, an authority model with a two-step transfer of the shop authority, and a risk list that starts with the shop's key being on a phone.

![The design lane: three lines of idea go to solana-researcher for cited findings with confidence labels, then to solana-architect for accounts, instructions, invariants and risks in design.md, then to /plan-feature for a phased plan, with game-architect taking the design step for games.](assets/v01-design-lane.webp)

4. **Turn the design into phases.** Type:

```text
/plan-feature Stamp, from design.md
```

The command writes no code. It reads the design, asks about decisions that change it, and returns a plan with accounts, instructions, integrations, clients, phases, risks, tests and rollout, **ending in open questions for you**. Answer them in `design.md` so the next agents read one file.

5. **If you are going to a hackathon, add the startup pack.** `/add-skill startup-builder` installs the whole pack of ten skills, idea-sprint, positioning, pricing, pitch-deck, the hackathon skill, build-status, fundraising, launch, incident-comms and ecosystem-bd, because they link to each other's files. Its idea-sprint writes `.claude/context/idea.md`, and `/plan-feature` starts from that file when it exists. The core **colosseum** skill does prior-art research on past submissions, after one sign-in per machine with `npx @colosseum-org/copilot-connect login`.

## Three idea seeds to try instead of Stamp

The same two prompts work on any idea. Replace the three lines and keep the rest:

- **Vaquinha**: friends pool SOL for a gift and the pool pays out when the target is reached, or refunds everyone at a deadline. First research question: what exists for group pools and escrow, and how do they handle the refund path?
- **Tool library**: a neighbourhood shed lends drills and ladders against a deposit that comes back on return. First research question: deposit-and-return patterns, and who holds the deposit.
- **Door list**: a small event sells a fixed number of entries that cannot be resold above face value. First research question: ticketing on Solana, and which projects tried to cap resale.

## Done when

- `idea.md` holds three lines and `design.md` holds accounts, instructions, invariants, authority and risks.
- Every finding in the research answer carries a link and a label.
- The plan's open questions are answered in `design.md`.

## Watch out

- **The architect designs and does not code.** If it starts writing Rust, stop it and ask for the design file.
- **Pinocchio needs a measured reason.** The architect's file says so: default to Anchor, and port one hot instruction only after Anchor's own levers failed.
- **A Low label is a lead.** Do not put a Low number in a deck.
- **Games go to game-architect.** It asks about gameplay, platform and what must be on-chain, then writes `concept.md` and `plan.md` and waits for your approval before anyone implements.

## The takeaway

Research comes back **cited and labelled**, and a design comes back as **accounts, instructions, invariants, authority and risks** in a file. *Ask each agent for the output its own file promises, and keep the file, because the next agents read it.*

## Next

Next lesson the anchor-engineer builds Stamp's three instructions from `design.md`, the QA engineer tests them, and you see the three commands that close every program task. Rust, Anchor and the Solana CLI enter here, so install them before the session if you want to follow along at the keyboard. `/doctor` printed the Solana CLI command already, and the engineer's scaffold step brings the Anchor toolchain into view.
