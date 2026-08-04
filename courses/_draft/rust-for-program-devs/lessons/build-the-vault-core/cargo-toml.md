## The `Cargo.toml` you never see

Your editor shows one file. The build server compiles a crate, and a crate is a manifest plus source. You do not edit the manifest, and it decides what your code is allowed to say — so read it once.

```toml
[package]
name = "academy-program"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
path = "src/lib.rs"

[profile.release]
overflow-checks = true
lto = true
codegen-units = 1
opt-level = "z"

[dependencies]
anchor-lang = { version = "1.1.2", features = ["init-if-needed"] }
borsh = "1.5.7"
```

Six lines in it are load-bearing for what you are about to write.

**`path = "src/lib.rs"`** — the file in the editor *is* `src/lib.rs`. One starter, one file, one crate. This is the reason the module is inline.

**`crate-type = ["cdylib", "lib"]`** — two outputs from one source. `cdylib` is the shared object the chain loads and runs. `lib` is an ordinary Rust library, which is what makes the crate importable by something else — a test harness, for instance. Nothing imports it today. It is the hook a `cargo test --lib` grading step would hang on, and it costs nothing to leave in.

**`edition = "2021"`, not 2024** — editions are per-crate, and a crate cannot be on an edition the Cargo compiling it has not stabilised. The build server's `cargo-build-sbf` ships its own pinned Rust, and that is the constraint. Practical effect: a snippet from a recent blog post that needs an edition-2024 feature will not compile here. Nothing in this course needs one.

**`overflow-checks = true` under `[profile.release]`** — worth understanding because it is so easily mistaken for a safety net. A release build normally *wraps* on integer overflow: `0u64 - 1` silently becomes `u64::MAX`. This line turns the panic back on, so it wraps no more. It is still not a substitute for `checked_sub`:

| What you write               | Release behaviour with this profile | What the caller of your instruction sees      |
| ---------------------------- | ----------------------------------- | --------------------------------------------- |
| `balance - amount`           | panic                               | aborted instruction, no error code, no message |
| `balance.checked_sub(amount)`| `None`                              | `InsufficientFunds` — a code and a `#[msg]` string you wrote |

A panic tells the caller that something went wrong. `checked_sub` + `ok_or` tells them what. That is lesson 4's argument, restated at the manifest level.

**`anchor-lang = { version = "1.1.2", … }`** — Anchor's Rust crate, at the version this course is written against. It re-exports the Solana crates it needs, which is why one `use anchor_lang::prelude::*;` is enough to get `Pubkey`, `Result`, `require!` and `msg!` without naming a single `solana-*` dependency yourself.

**`borsh = "1.5.7"`** — a range, not a pin. On a `1.x` crate, `"1.5.7"` means `>=1.5.7, <2.0.0`, so the version that actually compiles today is **1.8.0**, published 2026-07-16. To pin exactly you write `=1.5.7`. Internalise the habit now, because it is the same habit that keeps a client library from moving under you: the number in a manifest is usually a range, and the number that shipped is in the lockfile.

What is *not* here is also informative, and it is a different file. There is no `Anchor.toml` in play at all — the build server invokes `cargo-build-sbf` directly on this manifest, so the workspace file you will see in every Anchor tutorial has no part in grading. And even where there is one, the `[registry]` section and `anchor publish` it used to carry are gone from the toolchain: the on-chain IDL is now written by the **Program Metadata Program**. You will meet both in Course 3, at deploy time, when your program finally has an id to attach metadata to.
