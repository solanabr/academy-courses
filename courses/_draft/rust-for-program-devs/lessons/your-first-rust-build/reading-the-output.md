## Reading the compiler output

The build log is not noise you scroll past on the way to a green tick. It is the primary interface of this course. Learn its shapes now and the next thirteen lessons get much shorter.

### 0. When you get a log at all

Start here, because it is the platform fact that shapes everything else.

**A failing build hands you the compiler's whole output. A passing build hands you one line and a build id.** That is what the runner does: on success it reports `Program compiled successfully!` and discards the log; on failure it prints the captured output verbatim.

So the log is the *failure* interface. Everything below is what you read when something is wrong — which is most of the time, and is when you need it.

Two consequences worth carrying:

- **Warnings on a green build are invisible to you here.** They exist, `rustc` emitted them, and this platform throws them away along with the rest of a successful log. When a lesson says a mistake "only produces a warning", read that as *nothing on this platform will tell you*. That is a real limitation and it is worth knowing rather than working around.
- **A green build is one bit of information.** Not zero — it means the file compiled against the real Anchor 1.x toolchain, which is more than a linter can say. But one bit.

### 1. `Compiling` and `Finished`

This is what the log looks like on the server, whether or not any of it reaches you:

```
   Compiling academy-program v0.1.0 (/build/program)
    Finished `release` profile [optimized] target(s) in 3.41s
```

`Finished` is the whole verdict. If the build reaches it, the file compiled and the lesson is graded as passed. Grading in this course is a **compilation check** — passing means it compiled. It does not mean it is correct. Lesson 4 is the first place where the file itself carries assertions the compiler must also satisfy, and that is a deliberate design, not a default.

A failing build reaches the `Compiling` line and never the `Finished` one. It ends with a summary instead:

```
   Compiling academy-program v0.1.0 (/build/program)
error[E0308]: mismatched types
   …
error: could not compile `academy-program` (lib) due to 1 previous error
```

So the pair you will actually read is `Compiling` at the top and `could not compile` at the bottom, with the errors in between. `Finished` is a line you infer from the success message rather than one you see.

One `Compiling` line per crate, so a failing build with a cold dependency cache prints the whole tree with each crate's resolved version — `Compiling borsh v1.8.0`, `Compiling anchor-lang v1.1.2`. That is the only place the pinned stack states itself out loud, and if you ever see it, note `borsh v1.8.0` against a manifest that asked for `1.5.7`. Lesson 13 explains why those differ.

What the log does **not** contain, at any point: a line naming the Rust compiler version or the target triple. `cargo-build-sbf` prints neither. The pinned `rustc` arrives with the platform-tools version the build server passes on the command line, which is not visible in the output — and that is why every lesson in this course carries a **version stamp** at the top of its prose. The stamp is the source of truth. There is no banner to check it against.

### 2. Version pinning as a ceiling

The compiler the build server runs is older than the newest Rust release, and that is normal — platform-tools lags upstream by design, because validators run what the toolchain produced.

Treat it as a **ceiling**. Any language feature stabilised after it does not exist for you, no matter what the Rust release notes say, and no matter that the copy of Rust on your laptop is newer. This is the single most common way a snippet from a blog post fails here for a reason that has nothing to do with the snippet: it was written against a newer compiler.

The same goes at the library level. This course pins `anchor-lang` **1.1.2**. Anchor 1.0 was a breaking major release, so a large amount of the Anchor material on the internet — most of it, by volume — was written against 0.2x or 0.3x. One example you will meet immediately if you go looking:

- `CpiContext::new(ctx.accounts.system_program.to_account_info(), …)` was the 0.x shape. On 1.x it is `CpiContext::new(System::id(), …)`, and the 0.x form does not compile here.

A second rule you will read as though it were a compile error, and it is not: **one `#[error_code]` enum per program.** Two enums compile perfectly well on 1.1.2 — we checked by compiling them. The problem is that both start numbering at 6000, so each one's first variant is the same code on the wire and no client can tell them apart. It is a convention enforced by consequences, not by `rustc`. Lesson 11 makes you predict that one.

When you copy Rust from anywhere, check what it was written against first. "Rust from 2024" is not a version.

### 3. `warning:` — free advice you will only see next to an error

```
warning: unused variable: `amount`
 --> src/lib.rs:12:16
   |
12 | pub fn credit(amount: u64) {
   |               ^^^^^^ help: if this is intentional, prefix it with an underscore: `_amount`
```

Warnings do not stop the build and do not fail the lesson. When they are shown, read them: Rust's warnings are unusually good at pointing straight at real mistakes — `unused variable`, `unreachable pattern`, `value assigned is never read` are each, in practice, a bug about a third of the time.

"When they are shown" is doing real work in that sentence. Per section 0, you see them only in a failing build's log, alongside the error that failed it. A file that compiles green with a `warning: unreachable pattern` in it looks, from where you are sitting, identical to a file with no warning at all.

You will also see warnings you cannot fix and should ignore, including a run of `unexpected cfg condition value` lines coming out of Anchor's own macros. Those are Anchor's, not yours.

### 4. `error[E….]` — the one you came for

```
error[E0308]: mismatched types
  --> src/lib.rs:14:5
   |
13 | pub fn bump_as_u64(bump: u8) -> u64 {
   |                                 --- expected `u64` because of return type
14 |     bump
   |     ^^^^ expected `u64`, found `u8`
   |
help: you can convert a `u8` to a `u64`
   |
14 |     bump.into()
   |         +++++++
```

Five things are in there, and you want all five:

1. **The code**, `E0308`. Every Rust error has a stable numbered code. It is searchable and it is documented — the Rust error index has a page per code, with a minimal example. Search the code, not the message.
2. **`--> src/lib.rs:14:5`** — file, line, column. Your submission is always compiled as `src/lib.rs`, so the line number is the line number in the editor.
3. **The excerpt with the carets**, pointing at the exact expression, plus a second span showing *why* — here, the return type on line 13 is what created the expectation.
4. **`expected … found …`**, in that order. Expected is what the surrounding code demands; found is what you gave it. Getting the direction the right way round is most of the skill.
5. **`help:`**, which frequently contains the literal fix. Not always right — lesson 9 has a case where the suggestion compiles and is wrong for on-chain code. Usually right.

You will also see `For more information about this error, try 'rustc --explain E0308'` and, at the very bottom, the `error: could not compile` summary from section 1. Neither adds anything the block above did not already tell you.

### Read the first error only

A file with one real mistake often produces a wall of errors, because everything downstream of the mistake is now being checked against a type the compiler had to guess at. Fix the **first** error, rebuild, and watch most of the rest disappear. Working bottom-up through a cascade is the most reliable way to waste an afternoon.

And unlike a JavaScript runtime, which stops dead at the first exception, `rustc` keeps going and reports every independent error in one pass. Four unrelated mistakes give you four errors in a single build. That is a feature: it means one round-trip per batch of work, not one per bug.

### What to carry into lesson 2

- You get the log when the build **fails**. A pass is one line, and warnings go in the bin with it.
- The pinned compiler is a ceiling on what you can use, and the lesson's version stamp is where you read it.
- `Finished` = compiled = passed. Compiled does not mean correct.
- Errors have numbered codes worth searching. Fix the first one, then rebuild.
