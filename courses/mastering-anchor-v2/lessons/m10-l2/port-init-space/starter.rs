/// Port a v1 account-space calculation to the Anchor V2 rule.
///
/// In Anchor v1 you sized an account by hand as `8 + <field bytes>`, where the
/// leading `8` was the account discriminator. V2 drops the magic number: the
/// idiom is `T::DISCRIMINATOR.len() + T::INIT_SPACE`, and INIT_SPACE is the
/// sum of the Pod field sizes ONLY -- it never includes the discriminator.
///
/// This half-finished port deleted the hardcoded `8` when it removed the magic
/// number, but forgot that INIT_SPACE already excludes the discriminator. The
/// result under-counts by 8 bytes, so every account it sizes is allocated too
/// small and the first write overruns the buffer.
///
/// Field sizes (Pod, no padding, no borsh length prefixes):
///   Address field = 32 bytes, u64 field = 8 bytes, bool field = 1 byte.
/// The account discriminator is 8 bytes (sha256 default, unchanged in V2).
///
/// Fix `account_len` so it returns the FULL on-chain data length. It is deliberately
/// NOT named init_space: INIT_SPACE is the half that excludes the discriminator.
pub fn account_len(address_fields: u64, u64_fields: u64, bool_fields: u64) -> u64 {
    let init_space = address_fields * 32 + u64_fields * 8 + bool_fields;
    // BUG: INIT_SPACE excludes the 8-byte discriminator -- it must be added back.
    init_space
}
