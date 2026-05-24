# nako-addon-client

Rust HTTP caller helper for Nako Addon Sidecar integrations.

This crate builds on `nako-addon-protocol` and provides a mockable transport
interface plus a `reqwest` transport implementation for bounded health checks
resource calls, and Nako runtime side-effect calls.

```rust
use nako_addon_client::ReqwestAddonTransport;

let _transport = ReqwestAddonTransport::default();
```

Use `nako-addon-protocol` directly if you only need manifest and envelope wire
types.

License: Apache-2.0 OR MIT.
