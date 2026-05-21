# 0022: Keep Public Protocol Crates Permissive While Server Crates Remain AGPL

## Status

Accepted.

## Context

Taru is a network-accessible self-hosted server, so the server implementation
is intentionally licensed under `AGPL-3.0-or-later`. That choice fits the
product: modified server builds offered over a network should remain source
available to their users.

At the same time, Taru needs a clean reuse boundary for addon authors, client
applications, SDKs, and integration tools. Those consumers should be able to
depend on Taru wire contracts without inheriting the server implementation
license or depending on Taru's internal domain model.

`taru-addon-protocol` already establishes that pattern by using `Apache-2.0`.
`taru-addon-client` follows the same permissive boundary for the optional Rust
HTTP caller helper so the protocol crate can remain dependency-light. Future
public protocol or SDK crates will need the same treatment if they are meant
for independent reuse outside the server process.

## Decision

Taru server implementation crates remain `AGPL-3.0-or-later` unless a crate is
explicitly carved out as a public protocol or SDK boundary.

Crates intended for third-party reuse outside the server, especially protocol
and wire-contract crates, should use a permissive license. `Apache-2.0` is the
default choice for that boundary.

Public protocol crates must stay dependency-light and must not depend on AGPL
server crates or internal server domain models. If both sides need shared IDs,
DTOs, manifest types, or response envelopes, Taru should prefer one of these
approaches:

- move the neutral wire types into a permissive protocol crate; or
- duplicate the small wire types in the public boundary crate.

That boundary is for independent consumers, not for server convenience.
Example addons or fixtures that live inside the repository may remain AGPL if
they are not intended for external reuse.

Protocol crates should not absorb transport stacks just because Taru's server
or tests need a caller. If a permissive helper needs `reqwest`, async traits, or
runtime-specific networking behavior, it belongs in a separate permissive
client/helper crate that depends on the protocol crate.

## Consequences

- Addon authors can reuse Taru protocol crates without inheriting the server
  license.
- Future mobile, web, or CLI clients can target the same permissive wire
  contracts.
- The server stays strongly copyleft while the extension ecosystem stays open
  to independent implementations.
- Protocol crates require more discipline because they cannot casually import
  server internals.
- Optional helper crates such as `taru-addon-client` may carry heavier
  transport dependencies without making every wire-contract consumer inherit
  them.
- Some small wire types may be duplicated on purpose to keep the boundary
  clean.

## Alternatives Considered

- License everything AGPL: simpler mechanically, but it would make addon and
  client SDK reuse much less attractive.
- License public protocol crates AGPL as well: rejected for the same reason.
- Put shared DTOs in `taru-core`: rejected because it would couple external
  consumers to the server implementation boundary.
- Dual-license every crate: unnecessary complexity for the current project
  shape.

## Related Workstreams

- `docs/workstreams/addons-automation/`
- `docs/workstreams/server-foundation/`
