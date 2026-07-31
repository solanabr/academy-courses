# Publish the Deep Dive

You have a paid tier, a metered endpoint and an agent with a spend cap. Nobody knows.

This lesson is not a writing-skills lesson. It is a lesson about a specific artifact that a specific buyer pays for, and about the two properties that decide whether it gets paid or ignored: **it is dated, and it is pinned.**

## The market evidence, plainly and dated

Superteam Canada ran two listings on the Subscriptions & Allowances program, both closing 2026-06-23 — three weeks after the program hit mainnet on 2026-06-02:

| Listing                                    | Reward       | Submissions at close |
| ------------------------------------------ | ------------ | -------------------- |
| Subscriptions & Allowances **code sample**  | $2,750 USDG  | 31                   |
| Technical **deep dive** on the same primitive | $2,750 USDG | 44                   |

Educational modules on Superteam Earn sit at **$2,500–$3,000** and routinely close at around **10 submissions**. Promotional content — threads, videos, "explain this protocol" posts — closes at **107 to 677**.

Those are final counts on closed listings, not snapshots. That distinction matters and you will see it again in the next lesson: a submission count read on any single day tells you nothing, because the count is a function of how close the deadline is, not of how hard the work is.

Read the table again. The write-up paid the same as the implementation, and the field with the lowest competition on the platform is the one where you explain something you actually built. **Writing about what you built is the best-odds path here.** You already did the expensive part.

## Why the pins are the product

Every competitor's material on this primitive rotted the same way. It was written against a pre-1.0 package, the package moved, and the post never said which version it was written against. Six months later the code in the post does not run and there is no way to tell whether the reader's environment or the author's assumption is wrong.

`@solana/subscriptions` is at `0.4.0`, published 2026-07-13. Its `beta` dist-tag points at `0.4.0-rc.2` — an *older* pre-release. `@x402/core`, `@x402/svm`, `@x402/express` and `@x402/fetch` are at `2.19.0`, published 2026-07-17, while the unscoped `x402`, `x402-express`, `x402-next` and `x402-fetch` packages are frozen at `1.2.0` from 2026-04-16 and speak an older protocol version. A reader who installs the wrong half of either split gets a failure that looks like their own bug.

Undated, unpinned content is how that happens. A dated version banner and an exact pin block are two paragraphs of work, and they are the difference between an artifact with a shelf life and a screenshot.

## The shape — worked exemplar, annotated

What follows is the full skeleton of the piece, with the job of each section called out. Write your own version of this against your own artifact. The order is load-bearing: a reader who bounces off section 1 never reaches section 4, and a reviewer who cannot reproduce section 4 does not pay.

---

### `> Version banner` — first thing on the page, above the fold

> Written against `@solana/subscriptions@0.4.0`, `@solana/kit@7.0.0`, `@x402/core@2.19.0`, Solana devnet. Program `De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44`. **Checked 2026-07-25.**

*Job:* tell the reader in one line whether this page is worth their next five minutes. It is the only section that costs you nothing and saves every future reader. It carries a **date you actually ran the code**, not the date you published.

### 1. The problem, in one paragraph, from the user's side

> A SaaS app wants to charge 5 USDC every month. A one-off transfer means the user signs every month and churns the first time they are busy. Handing the app a signed blank cheque means the app can drain the account. Neither is acceptable, and this is why the recurring delegation exists.

*Job:* establish that there is a real decision here. Do not open with "Solana is fast." Open with the thing that does not work.

### 2. The minimal reference implementation

Minimal means: no framework, no UI, no error handling beyond the one error that matters, and it runs top to bottom. Everything the reader does not need is a reason to stop reading.

```ts
import { createClient, signer } from '@solana/kit';
import {
  subscriptionsProgram,
  findSubscriptionAuthorityPda,
  fetchMaybeSubscriptionAuthority,
  initSubscriptionAuthority,
  createRecurringDelegation,
  transferRecurring,
} from '@solana/subscriptions';

const client = createClient()
  .use(signer(userSigner))
  .use(solanaDevnetRpc({ url: RPC_URL }))
  .use(subscriptionsProgram());

// One authority per (user, mint). Branch on .exists — init twice and it throws.
const [authority] = await findSubscriptionAuthorityPda({
  user: userSigner.address,
  tokenMint: USDC_DEVNET,
});
const maybe = await fetchMaybeSubscriptionAuthority(client, authority);
if (!maybe.exists) {
  await initSubscriptionAuthority({ user: userSigner, tokenMint: USDC_DEVNET })
    .sendTransaction(client);
}

// The paid tier: 5 USDC of headroom, refreshed every 30 days.
await createRecurringDelegation({
  user: userSigner,
  tokenMint: USDC_DEVNET,
  delegatee: merchant.address,
  amountPerPeriod: 5_000_000n,        // base units, bigint — 5 USDC at 6 decimals
  periodSeconds: 2_592_000n,
  expiryTs: EXPIRY,                    // hard stop, no grace period
}).sendTransaction(client);

// Collection. The MERCHANT signs this, not the user.
await transferRecurring({
  delegatee: merchantSigner,
  delegator: userSigner.address,
  tokenMint: USDC_DEVNET,
  amount: 5_000_000n,
}).sendTransaction(merchantClient);
```

*Job:* be copy-pasteable. Amounts are base units as `bigint` — a reader who writes `5` instead of `5_000_000n` charges five millionths of a dollar and will not notice for a week, so annotate it inline rather than in a footnote.

### 3. The semantics that bite — the section that earns the fee

Anyone can paste an API sequence. What a reviewer is buying is the three behaviours that are not visible in the call signatures and that everyone gets wrong the first time. State each one as a claim, then show the observation that proves it.

**(a) A second pull inside the same period fails.** Call `transferRecurring` twice before the period rolls over and the second call is rejected with `AMOUNT_EXCEEDS_PERIOD_LIMIT` ("Transfer amount exceeds period limit"). The cap is per period, not per call, and there is no partial-fill: a pull for more than the remaining period allowance fails outright rather than transferring what is left.

**(b) The cap resets on rollover — it does not accumulate.** When the period rolls, the allowance is restored to `amountPerPeriod`. It is set back to the cap, not increased by it.

**(c) Skipped periods are lost.** A merchant who forgets to collect in March cannot collect twice in April. Unclaimed headroom does not carry forward. This is a *user-protective* design — it bounds the delegatee's total draw to one period at a time — and it is the single most common wrong assumption about the primitive, which is exactly why it is worth writing down.

And the security point underneath all three: **the delegatee signs transfers; the user signs setup and revoke.** The user approves once and never signs again; the merchant cannot exceed the cap; the user can `revokeDelegation` at any time, which closes the PDA and returns the rent to the signer — or to the recorded `receiver` if a sponsor funded the account.

*Job:* this is the part a reader cannot get from the README, and it is the part that makes the piece worth $2,750 instead of a link.

### 4. A runnable repro, with exact pins

```jsonc
// package.json — copied out of the lockfile, not typed from memory
{
  "dependencies": {
    "@solana/kit": "7.0.0",
    "@solana/subscriptions": "0.4.0",
    "@solana-program/token": "<the exact version your lockfile resolved>"
  }
}
```

```bash
npm ci
node repro.mjs            # prints the settlement signature
node repro.mjs --again    # prints AMOUNT_EXCEEDS_PERIOD_LIMIT
```

*Job:* let a reviewer verify claim (a) in under two minutes without reading your prose. `0.4.0`, not `^0.4.0` — a caret on a pre-1.0 package admits every future minor, and pre-1.0 minors break. Copy the resolved versions out of your lockfile; the version you *meant* to install is not evidence.

### 5. What you would not use this for

> Fixed delegation caps a total and never refills — that is an allowance, and it is what an autonomous agent should hold. x402 meters a single stateless request and involves no delegation at all. If your requirement is a human paying once at a checkout, none of the three is the answer.

*Job:* a piece that says what the primitive is *not* for reads as written by someone who used it. A piece that claims one rail solves everything reads as marketing.

---

That is the whole shape: **banner → problem → minimal implementation → the semantics that bite → a runnable repro with exact pins → the boundary.** Five sections and a header line.

## Two additions that cost nothing

**Say what you actually ran into.** If a call failed and you spent an hour on it, that hour is the most valuable paragraph in the piece — it is the paragraph no documentation contains. The empty-402-body case from lesson 6, where the server ships no body and puts the base64 envelope in the `PAYMENT-REQUIRED` header, is exactly this kind of paragraph.

**Do not build the indexer.** If you want a sixth section, note that the program emits six event types as Anchor-compatible self-CPI inner instructions under the tag `e445a52e51cb9a1d`, which is how you would build a revenue dashboard on top of this. One paragraph. Naming the mechanism is the value; implementing it is a different bounty.

## Where to publish

Anywhere with a stable URL you control the content of, and where the code blocks survive: your own site, a GitHub repo with the repro in it, or the listing's stated venue. The URL is what you submit in the next lesson, so it has to still resolve in six months.

Now write it. The exemplar above is the scaffold — the artifact is yours.
