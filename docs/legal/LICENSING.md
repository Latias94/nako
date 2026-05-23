# Nako Licensing Policy

## Project License

Nako server-side code is licensed under `AGPL-3.0-or-later` unless a crate or
file explicitly says otherwise.

This applies to the core server workspace, including:

- `nako-core`
- `nako-db`
- `nako-server`
- `nako-library`
- `nako-vfs`
- `nako-media-probe`
- `nako-metadata`
- `nako-nfo`
- `nako-search`
- `nako-streaming`
- `nako-transcode`
- `nako-events`
- `nako-automation`
- `nako-api`

The AGPL is intentional for the server because Nako is network-accessible
self-hosted infrastructure. Users should receive source access to modified
server builds that are offered over a network.

## Permissive Extension Surface

`nako`, `nako-addon-protocol`, and `nako-addon-client` are licensed under
`Apache-2.0 OR MIT`.

The addon protocol and public Rust SDK entry crates are kept permissive so
addon authors, SDKs, client tools, and integration services can adopt the
protocol without inheriting the server license for independent addon
implementations.

Protocol crates must stay independent from AGPL-only server crates. If a future
protocol or SDK crate needs shared IDs or DTOs, prefer duplicating protocol-level
wire types or moving neutral schema types into a permissive crate instead of
depending on AGPL server internals.

## Reference Code Policy

Reference repositories under `repo-ref/` are not part of Nako's source
distribution and must not be copied into Nako.

Allowed use:

- study architecture boundaries
- compare feature behavior
- inspect API shape and user workflows
- write original implementation notes
- write tests from observed behavior when the test data itself is original

Disallowed use:

- copying source files, functions, comments, SQL migrations, or tests
- translating GPL source line by line into Rust
- importing assets, schemas, or generated code unless their license is
  separately compatible and explicitly documented
- mixing reference repository files into Nako commits

When studying GPL projects such as Jellyfin, keep notes at the behavior and
architecture level. Nako implementations must be original work written against
Nako's own domain model and tests.

## License Files

- `LICENSE`: AGPL-3.0 license text
- `LICENSE-APACHE-2.0`: Apache-2.0 license text
- `LICENSE-MIT`: MIT license text
