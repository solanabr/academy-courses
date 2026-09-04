/// Port a v1 account-space calculation to the Anchor V2 rule.
///
///   space = T::DISCRIMINATOR.len() + T::INIT_SPACE
///
/// DISCRIMINATOR.len() is 8 (sha256 default, unchanged in V2); INIT_SPACE is the
/// sum of the field sizes and never includes the discriminator, so it is added
/// back exactly once here.
///
/// The restored link goes back where the migration took it from -- inside the
/// chain, not bolted on after `unwrap_or`. Outside, an overflowed count returns
/// 8: a number that looks like an empty account instead of the refusal it is.
///
/// The field sizes below are the `#[account(borsh)]` tier's: back-to-back
/// fields, no alignment padding, no length prefixes on fixed-size types.
fn account_len(address_fields: u64, u64_fields: u64, bool_fields: u64) -> u64 {
    const DISCRIMINATOR_LEN: u64 = 8;
    address_fields
        .checked_mul(32)
        .and_then(|bytes| bytes.checked_add(u64_fields.checked_mul(8)?))
        .and_then(|bytes| bytes.checked_add(bool_fields))
        .and_then(|init_space| init_space.checked_add(DISCRIMINATOR_LEN))
        .unwrap_or(0)
}
