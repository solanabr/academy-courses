# Declaring accounts up front is what buys parallelism

An Ethereum transaction says: call this address with this calldata. What storage
gets touched is discovered while executing, so the EVM cannot know in advance
whether two transactions conflict. It executes them one at a time. That is the
whole reason the EVM is single-threaded.

A Solana transaction says something stronger: here is every account I will touch,
and whether I need to write to it.

Because that list is complete and declared before execution, the scheduler can
look at two transactions and decide immediately whether they overlap. Two
transactions writing to different accounts run **in parallel**. Two writing the
same account are serialized. This is Sealevel, and it is the direct payoff for
the thing that annoys you most when you start: having to pass every account
explicitly.

## What this changes about your code

**Instructions cannot discover state mid-flight.** In Solidity you can read one
mapping, and based on the result, read another contract. On Solana, if the
account was not in the list, you cannot touch it — full stop. Dynamic access
patterns must become explicit lists computed by the client before sending.

**Contention is per-account, not global.** A popular NFT mint contends on that
mint's accounts. It does not slow down a lending protocol that touches entirely
different accounts. There is no shared global queue for unrelated work.

**Write locks are the scarce resource.** Declaring an account writable when you
only read it is a real cost: it blocks every other transaction that wants to
write it, for no benefit. Mark reads as reads.

## Reading a transaction

Because the account list is explicit, you can read any Solana transaction and
answer "what does this touch?" without executing it or having the source. An
Ethereum transaction hides that behind an opaque calldata blob and whatever the
contract chooses to do. This is why Solana wallets can show you a meaningful
simulation, and it is a genuine security property, not just tooling polish.

## The tradeoff, stated honestly

You are trading ergonomics for throughput. Building a transaction means knowing
your accounts in advance, which means deriving PDAs client-side (lesson 3) and
sometimes making an extra RPC call to discover what to include. Anchor's IDL and
its generated clients hide much of it, but the constraint is real and it is the
first thing EVM developers describe as friction.

It buys you a chain where unrelated work does not queue behind unrelated work.
Whether that is a good trade depends on what you are building — but it is a
trade, not an accident.
