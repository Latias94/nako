# nako-addon-protocol

Wire types and validation helpers for the Nako Addon Protocol.

This crate is intended for Addon Sidecar authors and integration tools. It
does not depend on Nako server internals. It contains manifest, resource,
task, and runtime side-effect wire types.

```rust
use nako_addon_protocol::{ADDON_PROTOCOL_VERSION, AddonManifest};

let _version = ADDON_PROTOCOL_VERSION;
let _manifest_type = std::any::type_name::<AddonManifest>();
```

The current Addon Protocol Version is `0.1.0-alpha.1`.

License: Apache-2.0 OR MIT.
