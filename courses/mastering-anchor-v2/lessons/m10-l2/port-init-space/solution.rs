/// Port a v1 account-space calculation to the Anchor V2 rule.
///
///   space = T::DISCRIMINATOR.len() + T::INIT_SPACE
///
/// DISCRIMINATOR.len() is 8 (sha256 default, unchanged in V2); INIT_SPACE is the
/// sum of the Pod field sizes and never includes the discriminator, so it is
/// added back exactly once here.
///
/// Field sizes (Pod, no padding, no borsh length prefixes):
///   Address field = 32 bytes, u64 field = 8 bytes, bool field = 1 byte.
pub fn account_len(address_fields: u64, u64_fields: u64, bool_fields: u64) -> u64 {
    const DISCRIMINATOR_LEN: u64 = 8;
    let init_space = address_fields * 32 + u64_fields * 8 + bool_fields;
    DISCRIMINATOR_LEN + init_space
}
