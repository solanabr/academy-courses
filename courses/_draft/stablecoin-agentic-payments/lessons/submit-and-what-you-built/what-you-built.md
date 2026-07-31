# Submit It — and What You Built

One artifact. Five courses. Name every layer of it.

## The through-line

Nothing you built restarted. Each course added a layer to the same object.

**Course 1 — you signed something.** Your own transaction, built and sent from your own key, confirmed on devnet, and an account inspector that reads state back off the chain rather than off a screenshot. The first time the chain answered you specifically.

**Course 2 — the vault got a core.** Rust that models the vault's state and its rules, compiled green against the real toolchain on the build server. No local install, no version roulette.

**Course 3 — your program went live.** Deposit and withdraw, hardened, deployed to devnet. You own a program id, and a credential NFT that says you do.

**Course 4 — it became something other people can use.** A typed client generated from your own IDL, wrapped in an API you would actually import, published to npm with a semver policy you wrote down — and a deployed app at a public URL that installs that package from the registry and signs a real deposit through it.

**Course 5 — it started earning.** A Subscription Authority and a Recurring Delegation gave the app a paid tier that charges once per period and correctly refuses to charge twice. An x402 route returns a real 402 challenge naming the network, the asset and the payTo address. An agent calls that route on a loop, pays for each call out of a Fixed Delegation, and halts cleanly when the allowance says no.

Say the sentence out loud, because it is true and you will need it in a proposal:

> You started as a web2 JavaScript developer, and you now own a monetized on-chain product.

If you entered here — at C5 lesson 1, on the reference app, because you wanted payments and not Anchor — the same sentence applies to the payments layer you built. The credential still requires your own artifact, not the reference one.

## What is in scope, and what is not

The rails you learned are the ones open to you. From 2026-10-01, BCB Resolution 561 forbids a regulated eFX provider from taking reais, converting to a stablecoin, and settling the obligation abroad on-chain. That is a live constraint on a business model, not on you personally: individuals may still buy, sell, hold and transfer through authorized VASPs.

What this course built sits entirely inside what the resolution leaves open: **domestic merchant acceptance, contractor payouts, subscriptions, and agent budgets.** Every one of those is a real product surface, and every one of them is something an Earn listing has asked for. When you write a proposal, name the use case in those terms — a reviewer in Brazil reads the difference immediately, and it is a signal that you understand the market and not just the SDK.

## Pick a rail before you pick a package

The decision you now make without looking it up:

- **Fixed delegation** — a capped total that never refills. An allowance. This is what an autonomous agent holds, because the on-chain cap is what makes the autonomy safe.
- **Recurring delegation** — a cap that resets each period. Payroll, contractors, subscriptions. Skipped periods are lost.
- **Subscription plan** — a merchant publishes terms and approved pullers collect against them.
- **Solana Pay** — a human paying once, at a checkout. A decision, not an install: `@solana/pay@1.0.23` peer-depends on `@solana/kit ^6.9.0`, which does not admit the `@solana/kit@7.0.0` this course pins everywhere else.
- **x402** — stateless per-request metering. No delegation, no relationship, one call.

That list is the most portable thing you learned. Packages will move; the shape of the requirement will not.

## Submit one listing

Now the part most people skip.

**The calibration, honestly.** What this path realistically opens is **your first $500 to $5,000** of paid on-chain work. It is not a job offer, and anyone telling you a course makes you hireable as a Solana developer is selling something. The $5,000 frontend listing drew 731 submissions. The odds live somewhere else.

**Where they live.** Two termini this course specifically unlocks:

1. **Payments technical demos** — the two Superteam Canada listings paid $2,750 each and drew 44 submissions (the code sample) and 31 (the deep dive): **$62.5 and $88.7 of reward per competitor**, second only to Rust listings on that measure. You have a metered endpoint and a paid tier already running.
2. **Educational modules** — $2,500–$3,000, typically closing near 10 submissions. Your deep dive from the last lesson is most of one.

**How to read a listing, which is a skill in itself.** Competition is a function of **time-to-deadline**, not of category. The Zeroclaw listing sat at twelve submissions or fewer for most of its run and closed at 52 as the deadline approached. A submission count you read today tells you where that listing is in its life, not how contested it is. So: **never quote a single-day count as a competition rate.** Filter for **newly-opened development listings** and compare medians across listings that have actually closed.

**Then submit.** One listing, this week, with a real artifact attached. Not a plan to submit. The bar is lower than it looks — most listings are decided among a handful of complete, reproducible submissions, and you now have the two things most submissions lack: something that runs, and a write-up that says which versions it ran against.

## One more door

Turbin3 runs cohort-based Solana builder programs, and its prerequisites are the kind of thing C1–C4 already satisfy: your own deployed program, a published client, a deployed app. If you want structure and a cohort after this, that is where to look. Bring the program id and the package URL.

That is the course. Submit the listing.
