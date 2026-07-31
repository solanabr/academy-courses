// ---------------------------------------------------------------------------
// Lesson 8 — Lifetimes and Slices.
//
// Two functions. Signatures and spec below, no worked example. The `mod verify`
// block at the bottom is NOT EDITABLE — it calls your functions at compile time
// and asserts on the answers, so a `todo!()` body or a wrong length is a build
// failure, not a silent pass.
//
// Toolchain: anchor-lang 1.1.2 · borsh 1.5.7 declared / 1.8.0 resolved · edition 2021.
// ---------------------------------------------------------------------------

/// Every Anchor account begins with 8 discriminator bytes. Do not change this.
pub const DISCRIMINATOR_LEN: usize = 8;

// ===== 1. discriminator ====================================================
//
// SPEC
//   Return a borrowed view of the first `DISCRIMINATOR_LEN` bytes of `data`.
//   If `data` is shorter than that, return `None` — never panic, and never
//   return a shorter slice.
//
// ACCEPTANCE
//   discriminator(&[])            == None
//   discriminator(&[0u8; 7])      == None
//   discriminator(&[0u8; 8])      == Some(..) with len 8
//   discriminator(&[0u8; 49])     == Some(..) with len 8, the FIRST eight
//
// NOTES
//   - The signature below has no `'a` on it. That is legal: one input
//     reference means elision has exactly one answer, and it fills it in.
//   - `const fn` restricts you: no `?`, no iterators, no closures. `if`,
//     `match`, `while` and slice methods that are themselves `const` are in.
//   - Indexing with `[..8]` panics on a short slice. The spec says `None`, so
//     check the length before you take anything.
pub const fn discriminator(data: &[u8]) -> Option<&[u8]> {
    todo!()
}

// ===== 2. discriminator_if =================================================
//
// SPEC
//   Return the account's discriminator ONLY when it equals `expected`.
//   Otherwise return `None`. This is what Anchor does before it deserializes
//   an account: wrong discriminator, wrong account type, refuse.
//
// ACCEPTANCE
//   discriminator_if(&[1u8; 49], &[1u8; 8])  == Some(..) with len 8
//   discriminator_if(&[1u8; 49], &[2u8; 8])  == None
//   discriminator_if(&[0u8; 7],  &[0u8; 8])  == None   (too short)
//
// THE LIFETIME WORK — this is the lesson
//   As written, this does not compile: E0106, missing lifetime specifier. Two
//   input references, one output reference, no `self`, so elision has nothing
//   to go on and the compiler refuses to guess.
//
//   Annotate it yourself, and get the answer RIGHT rather than merely legal:
//     - The slice you return is bytes out of `data`, so it borrows from `data`.
//     - `expected` is a caller's constant. You read it and never hand it back,
//       so it needs its OWN lifetime, unconnected to the return type.
//   Two lifetime parameters, therefore. Putting one name on both compiles the
//   function but over-constrains every caller, and the harness rejects it —
//   read the `expected fn pointer` / `found fn item` lines if you try it.
//
// NOTES
//   - Reuse `discriminator`. Do not re-implement the length check.
//   - Comparing bytes in a `const fn` means a `while` loop and indexing. There
//     is no `==` on slices you can call here, and no iterator.
pub const fn discriminator_if(data: &[u8], expected: &[u8; DISCRIMINATOR_LEN]) -> Option<&[u8]> {
    todo!()
}

// ===========================================================================
// VERIFICATION HARNESS — DO NOT EDIT.
//
// These items run inside the compiler. The two `_SIG_*` constants pin each
// function's exact shape, including which input the returned reference borrows
// from. The `const _: () = assert!(…)` items call your code during compilation,
// so a wrong answer is reported as an `error[E0080]` const-evaluation failure
// naming the assertion that did not hold.
//
// This trick works here only because both functions are `const fn` over plain
// slices. It does not reach `deposit(&mut self)` in module 4.
// ===========================================================================
#[allow(dead_code)]
mod verify {
    const _SIG_DISCRIMINATOR: for<'a> fn(&'a [u8]) -> Option<&'a [u8]> = super::discriminator;

    const _SIG_DISCRIMINATOR_IF: for<'a, 'b> fn(&'a [u8], &'b [u8; 8]) -> Option<&'a [u8]> =
        super::discriminator_if;

    const _: () = assert!(super::DISCRIMINATOR_LEN == 8);

    const _: () = assert!(super::discriminator(&[]).is_none());
    const _: () = assert!(super::discriminator(&[0u8; 7]).is_none());
    const _: () = assert!(match super::discriminator(&[0u8; 8]) {
        Some(d) => d.len() == 8,
        None => false,
    });
    const _: () = assert!(match super::discriminator(&[7u8; 49]) {
        Some(d) => d.len() == 8 && d[0] == 7 && d[7] == 7,
        None => false,
    });

    const _: () = assert!(super::discriminator_if(&[1u8; 49], &[1u8; 8]).is_some());
    const _: () = assert!(super::discriminator_if(&[1u8; 49], &[2u8; 8]).is_none());
    const _: () = assert!(super::discriminator_if(&[0u8; 7], &[0u8; 8]).is_none());
}
