# Rust Client SDK Foundation

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M29-M34 stabilized the Public Client API, OpenAPI inventory, and TypeScript SDK
package. Rust consumers now need a first-class SDK that reuses
`nako-client-protocol` DTOs instead of generating duplicate Rust wire types
from OpenAPI.

## Relevant Authority

- ADRs:
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
- Existing workstreams:
  - `docs/workstreams/public-client-api/`
  - `docs/workstreams/openapi-client-contract/`
  - `docs/workstreams/sdk-client-scaffold/`
  - `docs/workstreams/typescript-sdk-package/`
- Code boundaries:
  - `crates/nako-client-protocol`
  - `crates/nako-client`
  - `crates/nako-api/src/openapi.rs`

## Starting Audit

- `nako-client-protocol` is already `Apache-2.0` and dependency-light.
- Protocol DTOs cover the target JSON routes: health, libraries,
  catalog/search, source probe, playback decision, and playback sessions.
- The workspace default license is AGPL, so `nako-client` must override its
  crate license explicitly.
- `nako-api` owns the OpenAPI route inventory, but `nako-client` should not
  depend on `nako-api` because that would pull server/domain adapter crates
  across the public SDK boundary.

## Target State

- `crates/nako-client` exists as an `Apache-2.0` Rust SDK crate.
- The crate depends on `nako-client-protocol` for wire DTOs and does not depend
  on server/internal crates.
- The SDK exposes `NakoClient`, client configuration, pagination and playback
  query helpers, typed API errors, and an async transport boundary.
- Default runtime HTTP uses `reqwest`; tests use a mock transport without a
  real server.
- JSON routes cover health, libraries, catalog/search, source probe, playback
  decision, playback session inspection, and playback session cancellation.
- Streaming/raw byte APIs are intentionally deferred or represented only by a
  future-safe request/URL planning boundary.

## In Scope

- Add `crates/nako-client` with explicit permissive license metadata.
- Re-export or consume protocol DTOs without duplicating wire models.
- Add async client methods for core JSON public routes.
- Add auth/version/error/pagination tests through a mock transport.
- Add a local route inventory check that mirrors the OpenAPI public route set
  without depending on `nako-api`.
- Update HTTP API docs, goal map, roadmap, and workstream index.

## Out Of Scope

- crates.io publishing or release automation.
- Flutter/Dart SDK, Web UI, CLI product commands, or npm publishing.
- Full streaming body abstraction, download manager, or HLS player.
- OAuth/OIDC/RBAC/user sessions.
- Server public API expansion beyond small DTO hygiene.
- Any dependency from `nako-client` to server/internal crates.

## Architecture Direction

`nako-client` is the Rust SDK runtime crate. It should own HTTP mechanics:
base URL normalization, auth header insertion, API-version checking, error
envelope parsing, pagination serialization, and method-to-path mapping.

It should not own wire DTOs. Those remain in `nako-client-protocol`, preserving
a single Rust source of truth for public response shapes.

OpenAPI remains the contract inventory, but cross-checking should avoid making
`nako-client` depend on `nako-api`. The first M35 slice can keep a local
`PUBLIC_CLIENT_PATHS` inventory in `nako-client` and verify SDK coverage against
that list. A later follow-on can extract the inventory into the protocol crate
if duplication becomes costly.

## Closeout Condition

This lane can close when:

- `crates/nako-client` compiles and tests pass;
- dependency trees prove the SDK only depends on public/protocol/runtime
  crates, not server/internal crates;
- auth, version mismatch, error envelope, pagination, route paths, and leakage
  rejection are test-visible;
- docs record the Rust SDK boundary, examples, and validation commands;
- and publishing, full streaming, Rust CLI, Flutter/Dart, and UI work remain
  explicit follow-ons.
