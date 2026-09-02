/// Port a v1 account-space calculation to the Anchor V2 rule.
///
///   space = T::DISCRIMINATOR.len() + T::INIT_SPACE
///
/// DISCRIMINATOR.len() is 8 (sha256 default, unchanged in V2); INIT_SPACE is the
/// sum of the field sizes and never includes the discriminator, so it is added
/// back exactly once here.
///
/// Name the tier, because the arithmetic below only holds for one of them.
/// These are the sizes under `#[account(borsh)]`, where fields are written back
/// to back with no alignment padding and every type here is fixed-size, so
/// nothing carries a length prefix:
///   Address field = 32 bytes, u64 field = 8 bytes, bool field = 1 byte.
/// The same three fields cannot be the Pod default: the u64 forces 8-byte
/// alignment, the struct would need tail padding, and V2 rejects a padded Pod
/// account at compile time with `error[E0080]: account struct has padding
/// bytes`. That rejection is exactly why a shape like this is declared
/// `#[account(borsh)]` in the first place.
fn account_len(address_fields: u64, u64_fields: u64, bool_fields: u64) -> u64 {
    const DISCRIMINATOR_LEN: u64 = 8;
    // Checked throughout: field counts arrive from a caller, and a wrapped
    // length would look plausible while sizing the account far too small.
    address_fields
        .checked_mul(32)
        .and_then(|bytes| bytes.checked_add(u64_fields.checked_mul(8)?))
        .and_then(|bytes| bytes.checked_add(bool_fields))
        .and_then(|init_space| init_space.checked_add(DISCRIMINATOR_LEN))
        .unwrap_or(0)
}
