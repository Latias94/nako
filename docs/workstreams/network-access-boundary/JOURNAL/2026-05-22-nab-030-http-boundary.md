# NAB-030 HTTP Boundary Enforcement — 2026-05-22

## Context

NAB-020 added network exposure policy and config preflight checks. NAB-030
needed to make that policy meaningful at request time without weakening the
existing bearer-auth boundary or opening a NAT traversal/runtime lane.

## Changes

- Added `nako-server::http::network` as the HTTP network boundary module.
- Preserved auth order by applying origin rejection only inside protected routes
after bearer auth remains authoritative for missing/invalid credentials.
- Added global annotation/preflight middleware for configured origins and
trusted forwarded external origin headers.
- Wired `ConnectInfo<SocketAddr>` into the served router so trusted proxy source
policy can inspect the remote peer.
- Trusted `X-Forwarded-Host` / `X-Forwarded-Proto` only when forwarded headers
are enabled and the remote source matches exact-IP or CIDR policy.
- Rejected malformed multi-hop or whitespace-bearing forwarded host/proto
values instead of reflecting them.
- Kept `/health` public and compatible with readiness checks.

## Verification

- `cargo nextest run -p nako-server network_boundary_ --no-fail-fast` — pass,
2 passed / 240 skipped.
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast` — pass,
21 passed / 220 skipped.
- `cargo nextest run -p nako-server config --no-fail-fast` — pass, 38 passed /
205 skipped.

## Next

NAB-040 should expose Admin-only network readiness diagnostics and typed Admin
web contract/client support. Keep Public Client API, tunnel runtime, identity,
protocol downloaders, AI writes, Addon runtime, and library mutation out of the
slice.
