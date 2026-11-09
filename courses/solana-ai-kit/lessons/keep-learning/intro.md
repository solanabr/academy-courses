# Keep learning: solana-guide, /explain-code and staying current

Last lesson you wrote your own chain of agents. Fourteen of the fifteen have been introduced, and the fifteenth is the one for the weeks after the workshop, when you are alone with a program you half understand. This lesson is **the guide agent, the command that explains code, the researcher as a habit, and how the kit updates itself**, then a quiz over the whole course. Around fifteen minutes.

## Why this matters

The kit will build faster than you learn, and that gap is the one I would watch. **solana-guide** exists for the gap. Its file says it teaches so that *the learner can solve the next problem without it*: it checks your level first and asks rather than assumes, gives a plain definition and the problem the concept solves, then a minimal example, then the mistake beginners make, and it checks that you can predict before it moves on. It teaches **current tooling** and points it out when the tutorial you brought predates it. Production code goes to the engineers, and that split is deliberate.

## Do this

1. **Ask the guide something you skipped.** Type inside Claude Code:

```text
Use the solana-guide agent. I built Stamp with the kit and I do not really understand why the card account is a PDA instead of a normal account. I know JavaScript and a little Rust.
```

Expect it to confirm your level, define a PDA in plain words with the problem it solves, draw a Mermaid diagram of shop, card and customer, show the derivation, name the beginner's mistake, and then ask you to predict something, *what happens if someone passes a different card account*, before it goes on. If it skips the question, ask for it. Over several sessions it can run a track from the incubator curriculum in the solana-new pack, concept, working code, exercise, line-by-line review, until you build unaided and it hands you to the engineers.

2. **Explain code you did not write.** Type:

```text
/explain-code programs/stamp/src/lib.rs
```

The command's output has a fixed shape: an overview in two or three sentences, a **Mermaid diagram** of instruction flow or account relationships, a walkthrough of the significant sections with the reasons behind the non-obvious choices, the Solana concepts used, **security notes** on the checks the code performs and the ones it seems to miss, and for learners two or three *what happens if* questions. Run it on the anchor-engineer's output from lesson 3 and you will understand your own program better than when it was written.

![The learning lane: solana-guide checks your level, defines, draws, shows the mistake and asks you to predict, /explain-code gives an overview, a diagram, a walkthrough and security notes, solana-researcher keeps versions and status current, and /update, memsearch and /dream keep the kit and its memory current.](assets/v01-learning-lane.webp)

3. **Make the researcher a habit.** Before any decision that depends on a version, a deployment status or a protocol's state, ask the researcher rather than your memory or Claude's:

```text
Use the solana-researcher agent. Before I build on <program>, check its upgrade authority on mainnet, whether the deployed binary is a verified build, and its audit history. Cite and date everything.
```

That is its own pre-recommendation check, and the three answers are what you want before trusting a program with a shop's money.

4. **Keep the kit current.** Type:

```text
/update
```

It pulls the newest kit release tag, refreshes the agents, commands, skills and scripts the kit ships, **keeps your `.env` and your `CLAUDE.md`**, writes the kit's new instruction file beside yours as `CLAUDE.md.upstream` when it changed, keeps the extensions you installed, and re-applies your firewall tier. Restart Claude Code afterwards. The agents you read in this course are kit files, so an update can change them, and `/add-skill` with no id lists every pack with whether it is installed and when to install it.

5. **Give it a memory.** Two commands and a restart:

```text
/plugin marketplace add zilliztech/memsearch
```

```text
/plugin install memsearch
```

It is a plugin, not an MCP server, with a local store and a local embedder, so no key and no account, at the cost of a model download of around 558 MB on first launch. After a few sessions, daily markdown files appear under `.memsearch/memory/`, and that is where the program id you deployed and the design decision you rejected stop being re-derived. `/dream` consolidates a different store, `MEMORY.md` and the Project Learnings section of `CLAUDE.md`, when they grow stale, and asks before it touches the instruction file.

6. **Know where the long answers live.** The kit's `docs/` folder is never copied into your project. `agents-and-commands.md` has all fifteen agents and all 54 commands, `firewall.md` the tiers and their limits, `skill-packs.md` the pinned packs, `configuration.md` the MCP servers and plugins with what each costs per session. The official skill packs are hosted as markdown at [kit.superteam.codes](https://kit.superteam.codes), where any agent can read them without the kit, and the kit's own files are served at `aikit.superteam.codes/<path>`.

## Done when

- The guide asked you to predict something before moving on.
- `/explain-code` gave you a diagram and a security note on your own program.
- memsearch is installed and Claude Code was restarted.

## Watch out

- **Restart after memsearch and after `/update`.** Plugins activate on restart, and permission rules are read at session start.
- **Facts in this course are dated.** Counts, versions and command names were read on 2026-11-09 at kit version 2.4.0. When the kit's README and this lesson disagree, *the README is right*.
- **The guide teaches, the engineers build.** Ask each one for what its own file promises.

## The takeaway

One agent teaches at your level and makes you predict, one command explains code with a diagram and security notes, and the researcher checks authority, verified build and audits before you trust a program. *The kit updates itself and keeps your files, and the memory plugin keeps it from re-deriving what you already decided.*

The quiz below runs over the whole course. After it, build something small with the kit this week, and tell the people who ran your workshop which agent surprised you and which one let you down.
