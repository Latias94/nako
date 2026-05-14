# Taru Licensing Policy

## Project License

Taru server-side code is licensed under `AGPL-3.0-or-later` unless a crate or
file explicitly says otherwise.

This applies to the core server workspace, including:

- `taru-core`
- `taru-db`
- `taru-server`
- `taru-library`
- `taru-vfs`
- `taru-media-probe`
- `taru-metadata`
- `taru-nfo`
- `taru-search`
- `taru-streaming`
- `taru-transcode`
- `taru-events`
- `taru-automation`
- `taru-api`

The AGPL is intentional for the server because Taru is network-accessible
self-hosted infrastructure. Users should receive source access to modified
server builds that are offered over a network.

## Permissive Extension Surface

`taru-addon-protocol` is licensed under `Apache-2.0`.

The addon protocol is kept permissive so addon authors, SDKs, client tools, and
integration services can adopt the protocol without inheriting the server
license for independent addon implementations.

Protocol crates must stay independent from AGPL-only server crates. If a future
protocol or SDK crate needs shared IDs or DTOs, prefer duplicating protocol-level
wire types or moving neutral schema types into a permissive crate instead of
depending on AGPL server internals.

## Reference Code Policy

Reference repositories under `repo-ref/` are not part of Taru's source
distribution and must not be copied into Taru.

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
- mixing reference repository files into Taru commits

When studying GPL projects such as Jellyfin, keep notes at the behavior and
architecture level. Taru implementations must be original work written against
Taru's own domain model and tests.

## License Files

- `LICENSE`: AGPL-3.0 license text
- `LICENSE-APACHE-2.0`: Apache-2.0 license text
