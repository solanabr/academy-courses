// Anchor derives every 8-byte discriminator by hashing a NAMESPACED preimage:
// sha256("<namespace>:<Name>")[..8]. Instruction handlers hash from the "global:"
// namespace, not "instruction:", which is the one mapping that trips people up.
fn discriminator_preimage(item_kind: &str, name: &str) -> String {
    let namespace = match item_kind {
        "account" => "account:",
        "instruction" => "global:",
        "event" => "event:",
        _ => "",
    };
    format!("{namespace}{name}")
}
