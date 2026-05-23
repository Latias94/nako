# nako

Public Rust SDK entry point for Nako protocols and addon integration.

This crate is intentionally small. It is a facade over the public protocol and
SDK crates that third-party Addon Sidecars and integration tools may use. It
is not the Nako server implementation.

## Addon Protocol

```rust
use nako::addon_protocol::{ADDON_PROTOCOL_VERSION, AddonManifest};

let _version = ADDON_PROTOCOL_VERSION;
let _manifest_type = std::any::type_name::<AddonManifest>();
```

## Optional Addon Client

Enable the `addon-client` feature to use Nako's Rust HTTP caller helper:

```toml
[dependencies]
nako = { version = "0.1.0-alpha.1", features = ["addon-client"] }
```

```rust
use nako::addon_client::ReqwestAddonTransport;
```

## Scope

- `nako::addon_protocol` re-exports `nako-addon-protocol`.
- `nako::addon_client` re-exports `nako-addon-client` behind the
  `addon-client` feature.
- Server internals such as persistence, catalog, playback, metadata, VFS, and
  Admin API implementation details are not exposed by this crate.

The Nako server remains AGPL-3.0-or-later. This public SDK facade is licensed
as Apache-2.0 OR MIT for addon and integration authors.
