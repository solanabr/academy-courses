/// Port a v1 account-space calculation to the Anchor V2 rule.
///
/// In Anchor v1 you sized an account by hand as `8 + <field bytes>`, where the
/// leading `8` was the account discriminator. V2 drops the magic number: the
/// idiom is `T::DISCRIMINATOR.len() + T::INIT_SPACE`, and INIT_SPACE is the sum
/// of the field sizes ONLY -- it never includes the discriminator.
///
/// The v1 helper below was already defensive: field counts reach it from a
/// caller, so every step was a `checked_*` and a bad count degraded to 0 rather
/// than wrapping into a plausible-looking length. Its last link used to be
/// `.checked_add(8)` for the discriminator. Then a migration swept the file for
/// the additive `8` of the v1 `space = 8 + ...` idiom and took that link with
/// it. The `* 8` sizing a u64 field below survived the same sweep, correctly --
/// it is a field width, not a magic number -- which is how a chain that still
/// looks careful ends up under-counting every account by exactly 8 bytes.
///
/// Name the tier, because the arithmetic below only holds for one of them: these
/// are the `#[account(borsh)]` sizes, fields written back to back with no
/// alignment padding and, since all three types are fixed-size, no length
/// prefixes either.
///   Address field = 32 bytes, u64 field = 8 bytes, bool field = 1 byte.
/// Under the Pod default these same three fields never reach a length at all --
/// see `error[E0080]` in the lesson text for why.
///
/// TODO: put the discriminator back. `account_len` must return the FULL on-chain
/// data length, and the 8 belongs INSIDE the chain, where the deleted link was.
/// It is deliberately not named `init_space`: INIT_SPACE is the half that
/// excludes the discriminator, and that name is how the bug got written.
fn account_len(address_fields: u64, u64_fields: u64, bool_fields: u64) -> u64 {
    address_fields
        .checked_mul(32)
        .and_then(|bytes| bytes.checked_add(u64_fields.checked_mul(8)?))
        .and_then(|bytes| bytes.checked_add(bool_fields))
        .unwrap_or(0)
}
