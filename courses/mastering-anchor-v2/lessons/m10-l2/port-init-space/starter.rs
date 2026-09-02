/// Port a v1 account-space calculation to the Anchor V2 rule.
///
/// In Anchor v1 you sized an account by hand as `8 + <field bytes>`, where the
/// leading `8` was the account discriminator. V2 drops the magic number: the
/// idiom is `T::DISCRIMINATOR.len() + T::INIT_SPACE`, and INIT_SPACE is the
/// sum of the field sizes ONLY -- it never includes the discriminator.
///
/// This half-finished port deleted the hardcoded `8` when it removed the magic
/// number, but forgot that INIT_SPACE already excludes the discriminator. The
/// result under-counts by 8 bytes, so every account it sizes is allocated too
/// small and the first write overruns the buffer.
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
///
/// Fix `account_len` so it returns the FULL on-chain data length. It is deliberately
/// NOT named init_space: INIT_SPACE is the half that excludes the discriminator.
/// Leave the arithmetic checked -- an overflowed count must return 0, not wrap.
fn account_len(address_fields: u64, u64_fields: u64, bool_fields: u64) -> u64 {
    // BUG: INIT_SPACE excludes the 8-byte discriminator -- it must be added back.
    address_fields
        .checked_mul(32)
        .and_then(|bytes| bytes.checked_add(u64_fields.checked_mul(8)?))
        .and_then(|bytes| bytes.checked_add(bool_fields))
        .unwrap_or(0)
}
