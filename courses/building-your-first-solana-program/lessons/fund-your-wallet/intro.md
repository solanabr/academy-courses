# Fund Your Wallet

> **Version stamp — checked 2026-07-26.** Rent constants from `solana-rent`; account-size constants from `solana-loader-v3-interface` (`size_of_program() = 36`, `size_of_programdata_metadata() = 45`, `size_of_buffer_metadata() = 37`). The 234,040-byte binary is this course's finished vault, compiled with `cargo-build-sbf --tools-version v1.54`. Faucet limits are policy, not protocol, and move without notice — treat the SOL figures as exact and the faucet rules as approximate.

Before deploying your program, you need devnet SOL to pay for the on-chain storage (called "rent").

## You may already have some

If you came through Course 1 you funded this same wallet there, and it is very likely still holding SOL — devnet balances do not expire, and nothing in Courses 2 or 3 spent any. So check the balance in the widget below **before** you airdrop: if you are already at 4-5 SOL, you are done and the next lesson is waiting.

If Course 3 is your entry point, or the balance has drained, read on.

## How Much SOL Do You Need?

Deploying an Anchor program requires approximately **2 SOL** on devnet:
- **~1.63 SOL** for the buffer account (temporary storage during upload)
- **~1.63 SOL** for the **programdata** account, which holds your bytecode for the life of the program
- **~0.00114 SOL** for the program account itself (it is only 36 bytes — a pointer, not the code)
- **~0.3 SOL** for transaction fees (200+ transactions)

Those figures are for a compiled Anchor program of roughly **229 KB**, which is what your vault actually comes out to. Most of that size is the framework's account-validation and error machinery, so it barely moves with the number of instructions you wrote — three instructions and thirty cost close to the same.

Read that list again, because the two big numbers do not both stay spent. The buffer's deposit is **refunded automatically**: the deploy instruction drains the buffer back to your wallet in the same instruction that funds programdata. So you need ~2 SOL *on hand* to deploy, but you end up down roughly **1.63 SOL plus fees** — and that 1.63 stays locked as long as the program exists. Getting it back means `solana program close`, which permanently retires the program id.

## The Devnet Faucet

Solana provides a free faucet on devnet. The web faucet at [faucet.solana.com](https://faucet.solana.com) is the reliable path — airdrops through the public RPC (`solana airdrop`) are throttled more aggressively. The web faucet is rate-limited by **request frequency** (a maximum of 2 requests every 8 hours, as of mid-2026); signing in with GitHub raises that limit.

Use the widget below to fund your wallet. You'll need at least 4-5 SOL to comfortably deploy — because of the request-frequency limit, that may take more than one faucet visit, which is expected, not a failure.

> **Tip**: The faucet can be slow during peak hours. If you see "faucet is busy," just wait 30-60 seconds and try again. This is normal — devnet is a shared resource.

## The same rent, at a different scale

This is the rent-exempt deposit you already sized for the vault account in module 2 — `(128 + space) × 3480 × 2` — applied to something four thousand times larger. Nothing about the rule changed; only `space` did.

Run it on the deploy accounts and the numbers above fall out:

```
programdata: (128 + 45 + 234,040) × 6,960  =  1,630,122,480 lamports  ≈  1.63 SOL
program:     (128 +          36) × 6,960  =      1,141,440 lamports  ≈  0.00114 SOL
```

The 45 and the 36 are the loader's own metadata headers, and 234,040 is your compiled `.so` in bytes. Note which account is expensive: the program account is a 36-byte pointer, and the bytecode lives in programdata.

One correction worth carrying forward from module 2: rent is a **deposit, not a meter**, and the buffer's deposit comes back to you automatically as part of deploying. The programdata deposit does not — that one is the real price of having a program on-chain.
