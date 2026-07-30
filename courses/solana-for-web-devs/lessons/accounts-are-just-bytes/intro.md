# Accounts Are Just Bytes

> Version stamp — `@solana/kit` 7.0.0 · devnet · authored 2026-07-28.

Yesterday you read the vault in an explorer. Today you become the explorer: fetch the account over RPC and turn its 49 raw bytes into `{ owner, balance, bump }` yourself.

## Fetching the bytes

In Kit, an RPC client is one function call, and a read is a request you build and then `.send()`:

```ts
import { createSolanaRpc, address } from "@solana/kit";

const rpc = createSolanaRpc("https://api.devnet.solana.com");
const vault = address("FY86s1fAwUiFQTjVFYprsiV6fwNH7e955MSUBo73FP4j");

const { value } = await rpc
  .getAccountInfo(vault, { encoding: "base64" })
  .send();
// value.data[0] is the base64 string; decode it and you hold 49 bytes.
```

That is the whole read path. What comes back is not JSON, not an object, not a struct — a base64 string that decodes to 49 bytes. The chain stores bytes; **meaning is a layout you have to know**.

## The layout

This vault program documents its account layout (and in Course 2 you will write the Rust struct that produces it):

| offset | length | field | encoding |
|---|---|---|---|
| 0 | 8 | discriminator | fixed bytes `[228, 196, 82, 165, 98, 210, 235, 152]` |
| 8 | 32 | owner | raw public key, base58-encoded for display |
| 40 | 8 | balance | unsigned 64-bit integer, **little-endian** |
| 48 | 1 | bump | single byte |

- The **discriminator** is an 8-byte type tag stamped at offset 0 of every account this program family creates — it answers "are these bytes actually a VaultState?" before you read another field. In module 3 your inspector checks it before decoding anything.
- The **owner** field is the human answer lesson 1 promised: the wallet that controls this vault, stored as 32 raw bytes.
- The **balance** is the vault's own accounting of what its owner deposited — distinct from the account's lamports, which also carry the rent deposit.
- The **bump** you will understand in the next lesson. For today: one byte at the end.

## Little-endian, felt once

`balance` is a u64 stored least-significant byte first. A `DataView` reads it in one call — the `true` is "little-endian", and forgetting it is the classic bug:

```ts
const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
const balance = view.getBigUint64(40, true); // true = little-endian
```

Note the type: **`BigInt`, not `Number`**. Lamport amounts are u64, and `Number` silently loses integer precision above 2^53 — real mainnet balances cross that line. Every lamport value in this course is a `bigint`.

## The production form

Decoding by hand teaches you what the bytes are. In production code you declare the layout once with Kit's codecs and get the whole struct back:

```ts
import {
  getStructDecoder,
  getU64Decoder,
  getU8Decoder,
  getAddressDecoder,
  getBytesDecoder,
  fixDecoderSize,
} from "@solana/kit";

const vaultDecoder = getStructDecoder([
  ["discriminator", fixDecoderSize(getBytesDecoder(), 8)],
  ["owner", getAddressDecoder()],
  ["balance", getU64Decoder()],
  ["bump", getU8Decoder()],
]);

const state = vaultDecoder.decode(data); // { discriminator, owner, balance, bump }
```

Same 49 bytes, same offsets, same little-endian u64 — the codec just says it declaratively. Use codecs in real code; decode by hand today so the codec is never magic.

## The exercise

The grader has no network, so the harness hands your function **bytes recorded from the real devnet account** — the exact 49 bytes you can re-fetch yourself with the snippet above. Write `decodeVault(data)` in four labeled subgoals; a real base58 helper is provided for the owner field. One of the test fixtures is a *different* vault with a different owner, balance, and bump — so decode the bytes in front of you, don't transcribe the explorer.
