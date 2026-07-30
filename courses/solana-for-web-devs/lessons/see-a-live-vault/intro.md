# See a Live Vault

Before you write a line of code, you are going to read state off a real blockchain — a program deployed on Solana's devnet, frozen so it can never change, holding a real vault account you can inspect right now.

Nothing in this lesson is a simulation or a screenshot. Every number you see, you read live.

## Two kinds of account

Solana has exactly one container for everything: the **account**. An account has an address, a lamport balance, an owner, and a slab of data bytes. What people call "programs" and "data" are both accounts — the difference is one flag:

- A **program account** is marked `executable`. Its bytes are compiled code. Ours lives at [`D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd`](https://explorer.solana.com/address/D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd?cluster=devnet) — open it and the explorer says **Program**, and its **Upgradeable** field says **No**. That is the freeze: the deploy authority was burned, so this exact code is what runs, forever.
- A **data account** is not executable. Its bytes are state. The vault you will spend three courses with lives at [`FY86s1fAwUiFQTjVFYprsiV6fwNH7e955MSUBo73FP4j`](https://explorer.solana.com/address/FY86s1fAwUiFQTjVFYprsiV6fwNH7e955MSUBo73FP4j?cluster=devnet).

Open the vault link and read it like a table:

- **Balance** — the account's lamports. A lamport is a billionth of a SOL, and it is the native unit everything on Solana is denominated in. Read the number off the page; it is live, and later in this course you will change it.
- **Owner** — the program allowed to modify this account's data: `D7ZF…` — our vault program. On Solana *the owner of a data account is a program*, not a person. The person is recorded somewhere else, as you will see in the next lesson.
- **Allocated Data Size** — 49 bytes. Small enough that in the next lesson you will decode every single one by hand.
- **Executable** — No. State, not code.

One habit to build immediately: the `?cluster=devnet` at the end of those URLs. The explorer defaults to mainnet, where none of this exists — drop the parameter and it will tell you the account is not found, and you will waste an hour concluding the chain ate your work. It didn't. You were looking at the wrong chain.

## Why the balance is a moving target

This lesson will not tell you the vault's balance, on purpose. Other learners are depositing into this same account while you read this. Any number printed here would be stale by tonight — so the rule of this course is: **numbers come from the chain, not from the lesson**. Read it yourself; that is the skill.

## Rent: why accounts hold lamports at all

Every account must keep a minimum lamport balance proportional to its data size — called **rent-exemption**. It is a refundable deposit, not a fee: storage on every validator costs something, so the network requires accounts to lock roughly 0.00089 SOL plus a per-byte amount, and returns it when the account closes. An account below its minimum can be garbage-collected. In module 3 your inspector will compute this minimum for any account and report whether the account clears it.

## The explorer below

Under this text is the same program, embedded — its three instructions (`initialize_vault`, `deposit`, `withdraw`) read from the program's published interface, the **IDL**. Don't call anything yet; you have no wallet and no funds, and that is fine. Just open each instruction and read what it wants: which accounts it touches, who must sign, what arguments it takes. Notice `deposit` lists the vault account **first**, then the user, then the System Program — that exact order matters in module 2, when you build this call yourself.

Then take the quiz — it asks you to predict what the chain does, and every answer is checkable against the pages you just opened.
