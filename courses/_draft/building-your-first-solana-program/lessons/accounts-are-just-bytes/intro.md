# Accounts Are Just Bytes

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26) · `cargo-build-sbf` with platform-tools v1.54 (rustc 1.89) · `borsh` resolves to **1.8.0** · edition 2021. The three rent constants below — `ACCOUNT_STORAGE_OVERHEAD = 128`, `DEFAULT_LAMPORTS_PER_BYTE_YEAR = 3480`, `DEFAULT_EXEMPTION_THRESHOLD = 2.0` — were read out of `solana-rent`. The byte sizes were verified as `const` assertions compiled on the toolchain above.

Your program so far has no memory. Every instruction logs a line and exits. Call it a thousand times
and nothing about the chain is different afterwards.

To fix that you need an **account** — and an account is not an object, a row, or a document. It is a
byte buffer whose length you choose once, at creation, and can never change.

That single sentence is why this lesson comes before you write any code. The size decision is
irreversible, it costs the payer money for as long as the account exists, and Anchor will not compute
it for you unless you ask.

## What the runtime actually stores

Every account on Solana carries exactly four things the runtime cares about:

| Field    | What it is                                                    |
| -------- | ------------------------------------------------------------- |
| address  | 32 bytes. How anything finds the account                      |
| owner    | 32 bytes. The **program id** allowed to write the data        |
| lamports | The balance — and the rent-exemption deposit                  |
| data     | A flat `[u8]` of fixed length. This is the part you design    |

Note what is missing: no type, no schema, no column names. `data` is bytes. The only reason
`[0, 0, 0, 0, 0, 0, 0, 0, 82, 12, …]` means anything at all is that a program agrees to read it a
particular way.

"Owner" is the word people trip on. The **owner is a program**, not a person. Your vault's owner will
be your program id — that is what makes it program-controlled state rather than data sitting under
somebody's wallet. Your own `VaultState` has a field called `owner` too, and that one holds the
*user's* wallet. Two different meanings, one word, both correct, and you will read code that uses
each. The runtime's owner is enforced by the runtime. Yours is enforced only by the constraints you
write in module 3.

## The eight bytes you did not write

Anchor puts an 8-byte **discriminator** at the front of every `#[account]` buffer. It is derived from
the account's type name, so `VaultState`'s eight bytes are not `Config`'s eight bytes.

```
byte 0               8                                                          49
  │  discriminator   │  your fields, Borsh-serialized in declaration order      │
  └──────────────────┴──────────────────────────────────────────────────────────┘
```

Every deserialize checks those eight bytes first. Hand your program some other program's account — or
your own `Config` where a `VaultState` is expected — and it fails on the discriminator instead of
reading foreign bytes as if they were your struct. This is the cheapest security property in Anchor,
and you get it by typing `#[account]`.

You never read or write those eight bytes yourself. You do have to **pay for them**.

## Deriving the vault's size, out loud

Here is the struct you wrote in Course 2, unchanged:

```rust
#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub owner: Pubkey,   // 32
    pub balance: u64,    //  8
    pub bump: u8,        //  1
}
```

Borsh writes fields in declaration order, back to back, with no padding and no type information. So:

```
  8   discriminator   (Anchor, on every #[account])
 32   owner: Pubkey   (32 raw bytes, no length prefix)
  8   balance: u64    (little-endian, 8 bytes, fixed)
  1   bump: u8        (one byte, values 0..=255)
───
 49   bytes total
```

**49.** Not "about 48", not "round it to 64". Forty-nine. You will type that as
`space = 8 + 32 + 8 + 1` in the next two lessons — written as a sum rather than as `49`, so the reason
for every term stays visible in the code.

The sizes you need for anything else:

| Type               | Bytes                                   |
| ------------------ | --------------------------------------- |
| `bool`, `u8`, `i8` | 1                                       |
| `u16` / `i16`      | 2                                       |
| `u32` / `i32`      | 4                                       |
| `u64` / `i64`      | 8                                       |
| `u128` / `i128`    | 16                                      |
| `Pubkey`           | 32                                      |
| `Option<T>`        | 1 + size of `T`                         |
| `String`           | 4 (length prefix) + bytes of content    |
| `Vec<T>`           | 4 (length prefix) + count × size of `T` |

The last two are where fixed-size accounts and variable-length data collide: you must allocate for the
**maximum** length you will ever accept, and then enforce that maximum in code. A `String` field with
no length cap is an account you cannot size.

## Two ways to get 49 wrong

**`std::mem::size_of::<VaultState>()` is not the answer.** Rust aligns struct fields for CPU access;
Borsh does not. `size_of::<VaultState>()` is **48** — 41 bytes of fields rounded up to the struct's
8-byte alignment — while Borsh writes **41**. Size with `size_of` and you allocate 56 bytes for a
49-byte account, and overpay rent on every vault for as long as it exists, for no benefit. Rust's
memory layout and Borsh's wire layout are different problems that happen to be about the same struct.

**Counting the discriminator twice, or not at all,** is the other one. `8 + 32 + 8 + 1` counts it
once. If you ever see a `space` that equals exactly the sum of the field sizes, that account is 8
bytes short, and the first write past the field boundary fails.

The maintainable form is the one your Course 2 file already set up:

```rust
space = 8 + VaultState::INIT_SPACE   // 8 + 41 = 49
```

`#[derive(InitSpace)]` generates `INIT_SPACE` as the sum of the field sizes — **without** the
discriminator, which is why the `8 +` is still yours to write. Add a field later and the constant
follows; a literal sum does not. Ship `INIT_SPACE`. We write the sum in this course because you are
learning what it is made of.

## Why the size is fixed, and what it costs

The runtime allocates `space` bytes at creation, and that is the account's size for life. There is a
separate `realloc` mechanism for growing an account later — a deliberate operation with its own payer
and its own constraints, not something that happens because your struct grew.

The account also has to be **rent-exempt** to survive, which means holding a lamport balance above a
minimum that scales with size:

```
minimum = (128 + space) × 3480 × 2 lamports
```

128 bytes is the per-account storage overhead the runtime charges regardless of your data, 3,480
lamports is the per-byte-year price, and 2 is the exemption threshold in years. For the vault:

```
(128 + 49) × 3480 × 2  =  177 × 6960  =  1,231,920 lamports  ≈  0.00123 SOL
```

That deposit is not a fee. It sits in the account and comes back if the account is ever closed. But it
is the payer's SOL, locked, for as long as the account exists — which is why over-allocating has a
price tag and "just make it 1024 bytes to be safe" is a decision, not a precaution. Those three
numbers are protocol parameters, so the way to read the current value rather than trust a number in a
tutorial is the `getMinimumBalanceForRentExemption` RPC method.

One thing that is **not** a cost: ongoing rent collection. It is gone. An account's `rent_epoch` field
is vestigial — if you find prose describing it as "when rent is next due", that prose is describing a
mechanism that no longer runs.

## What Anchor stopped asking for

Search for account-creation examples and you will find this shape:

```rust
#[account(init, payer = user, space = 49, rent = rent)]
```

`rent = rent` was **removed in Anchor 0.31**. There is no `rent` constraint, and no
`rent: Sysvar<'info, Rent>` field to add alongside it. Anchor reads the rent parameters itself and
computes the deposit from `space`. Any tutorial still passing it predates 0.31 by definition — which
is a useful dating signal for everything else on that page.

What you do still supply, every time, is `space`. And now you know what each term in it pays for.
