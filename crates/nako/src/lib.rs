//! Public Rust SDK entry point for Nako.
//!
//! This crate is intentionally a small facade over Nako's public protocol and
//! SDK crates. It is not the Nako server implementation.

pub use nako_addon_protocol as addon_protocol;

#[cfg(feature = "addon-client")]
pub use nako_addon_client as addon_client;

#[cfg(test)]
mod tests {
    use super::addon_protocol::ADDON_PROTOCOL_VERSION;

    #[test]
    fn exposes_current_addon_protocol_version() {
        assert_eq!(ADDON_PROTOCOL_VERSION, "0.1.0-alpha.1");
    }
}
