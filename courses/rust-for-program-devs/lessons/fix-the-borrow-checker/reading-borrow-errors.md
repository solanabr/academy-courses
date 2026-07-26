# Fix the Borrow Checker

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (crates.io latest stable) · `borsh` 1.5.7 declared / **1.8.0** resolved · edition 2021 · Agave ≥ 3.1.10 / rustc per the build server's pinned platform-tools. Every error message quoted below was produced by compiling the starter file as shipped.

You have read the rules twice. Reading them a third time will not help. What helps is reading the compiler.

The exercise below does not build. Four functions, four real errors, and every fix is already somewhere on the page — commented out among a set of candidate lines, some of which are wrong. Your job is to pick and order, not to invent.

**Build it before you change anything.** Then read the *first* error only. Rust reports every error it finds, and a borrow error early in a file usually produces two or three more downstream that vanish the moment you fix the first. Chasing them in parallel is how people conclude the borrow checker is arbitrary.

## The four you will meet

### `E0382` — use of moved value

```text
error[E0382]: use of moved value: `vault`
  |
  |     consume_and_log(vault);
  |                     ----- value moved here
  |     vault.balance
  |     ^^^^^^^^^^^^^ value used here after move
```

Read it bottom-up: the caret is where you used it, the dashes are where you lost it. The question to ask is never "how do I get it back" — it is **did that function need to own it at all?** Usually not, and the fix is a borrow. When it genuinely does need to own it, take what you need out first, or clone.

### `E0502` — cannot borrow as mutable because it is also borrowed as immutable

```text
error[E0502]: cannot borrow `*v` as mutable because it is also borrowed as immutable
  |
  |     let before = &v.balance;
  |                  ---------- immutable borrow occurs here
  |     credit(v, shortfall);
  |     ^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
  |     *before
  |     ------- immutable borrow later used here
```

Three lines named, and the third is the one that matters. The mutable borrow gets the caret, but it is not the problem — the problem is that the shared borrow is **used again afterwards**, which is what keeps it alive across the mutation. Delete that last use and the error disappears.

So the fix is almost never "make the mutable borrow later". It is: stop holding the shared borrow. If what you needed was a `u64`, take the `u64` by value; it is `Copy`, and a copied number cannot conflict with anything.

### `E0499` — cannot borrow as mutable more than once at a time

```text
error[E0499]: cannot borrow `*v` as mutable more than once at a time
  |
  |     let one = &mut *v;
  |               ------- first mutable borrow occurs here
  |     let two = &mut *v;
  |               ^^^^^^^ second mutable borrow occurs here
  |     credit(one, first);
  |            --- first borrow later used here
```

The honest reading: you asked for exclusive access twice and called it exclusive both times. The fix is nearly always to stop naming the borrows and just make the two calls in sequence — each call takes its borrow, uses it, and gives it back before the next line starts.

Note what this error is *not*. Two mutable borrows of **different fields** of the same struct are fine (`&mut v.balance` and `&mut v.bump` coexist happily) — the checker tracks each field as its own place. If you have talked yourself into a `RefCell` to work around this error, re-read it first; most of the time the borrows were disjoint and the code just needed reordering.

### `E0515` — cannot return reference to local variable

```text
error[E0515]: cannot return value referencing local variable `owned`
  |
  |     &owned[..8]
  |     ^-----^^^^^
  |     ||
  |     |`owned` is borrowed here
  |     returns a value referencing data owned by the current function
```

This one is not about the rules of borrowing. It is about **time**. `owned` is dropped when the function returns, so a reference into it would point at freed memory the instant the caller received it — the exact class of bug that makes C hard and that Rust exists to make impossible.

Every returned reference has to borrow from something that outlives the call. A parameter does. A local does not. The fix in bug 4 replaces two lines with one, and the one it keeps borrows from the parameter.

And that question — *which* input does the returned reference borrow from? — is the question lifetime annotations answer. That is the next lesson, and bug 4 is the reason it exists.

## Working method

1. Build. Read the first error. Ignore the rest.
2. Look at the candidate lines under that function. Decide which belong and in what order.
3. Comment out the lines marked `BUG`, uncomment your choice, build again.
4. Repeat. Four builds, four errors, in order.

If you get a fifth error you did not expect, you have chosen a decoy. That is a useful outcome — read what it says before you undo it.

One route is closed off deliberately. Four broken functions in a file that has to compile makes deleting them look like a fix; it is not, and the non-editable `mod verify` block at the bottom pins all five signatures, so a deletion fails to build with `E0425: cannot find value`. What that block cannot do is check a body. Bug 3 fixed in the wrong order still compiles, which is what quiz question 3 is about.
