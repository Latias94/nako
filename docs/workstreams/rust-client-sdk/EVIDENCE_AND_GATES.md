# Rust Client SDK Foundation Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Starting Repro

- `nako-client-protocol` has public wire DTOs and is Apache-2.0.
- There is no Rust SDK crate for consumers.
- Rust callers would need to hand-roll HTTP paths, bearer auth, error parsing,
  API version checks, and pagination.

## Gate Set

### Focused SDK Gate

```bash
cargo check -p nako-client --tests
cargo nextest run -p nako-client --no-fail-fast
cargo tree -p nako-client
```

### Public Protocol Direction Gate

```bash
cargo tree -p nako-client-protocol
```

### Cross-SDK Regression Gate

```bash
npm run check --prefix sdk/typescript
```

### Workspace Closeout Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Anchors

- `crates/nako-client/Cargo.toml`
- `crates/nako-client/src/lib.rs`
- `crates/nako-client-protocol`
- `docs/api/HTTP_API.md`
- `docs/workstreams/rust-client-sdk/`

## Prompt-To-Artifact Checklist

- Establish Rust SDK crate boundary:
  `crates/nako-client` and workstream docs.
- Preserve permissive SDK license:
  explicit `license = "Apache-2.0"` in `crates/nako-client/Cargo.toml`.
- Reuse protocol DTOs:
  `nako-client` depends on `nako-client-protocol`.
- Avoid server/internal dependencies:
  `cargo tree -p nako-client` and forbidden term tests.
- Provide async JSON client:
  `NakoClient`, helpers, transport, errors, and route methods.
- Cover auth/version/error/pagination/paths:
  mock transport tests.
- Cover route inventory and leakage rejection:
  SDK tests.
- Preserve TypeScript SDK gate:
  `npm run check --prefix sdk/typescript`.
- Validate:
  final gate output recorded before closeout.

## Completion Audit

### Objective Requirements

- Add a Rust client SDK foundation after M29-M34:
  `crates/nako-client` exists and is included by the workspace `crates/*`
  membership.
- Do not generate duplicate Rust wire types from OpenAPI:
  `crates/nako-client/src/lib.rs` re-exports and consumes
  `nako-client-protocol` DTOs.
- Support future Rust consumers through a clean crate boundary:
  `crates/nako-client/Cargo.toml` names the SDK crate and records
  `license = "Apache-2.0"`.

### Scope Requirements

- New SDK crate:
  `crates/nako-client/Cargo.toml` and `crates/nako-client/src/lib.rs`.
- Permissive SDK boundary:
  explicit Apache-2.0 license in `crates/nako-client/Cargo.toml`.
- Protocol DTO source:
  `nako-client` depends on `nako-client-protocol`.
- No server/internal dependency:
  `cargo tree -p nako-client` contains no `nako-core`, `nako-api`,
  `nako-server`, `nako-streaming`, or `nako-transcode`.
- Minimal async client:
  `NakoClient`, `ReqwestTransport`, `ClientTransport`, `NakoClientError`,
  `PageQuery`, `SearchQuery`, and `PlaybackCapabilitiesQuery`.
- Bearer auth:
  mock transport test `client_adds_auth_and_pagination_query_to_protected_routes`.
- API version check:
  mock transport test `mismatched_api_version_is_rejected`.
- Public error envelope:
  mock transport test `api_error_uses_public_error_envelope`.
- Pagination helper:
  `PageQuery` and the same auth/pagination test.
- Core JSON routes:
  SDK methods cover health, libraries/list/detail/sources, items/list/detail,
  item credits/images, people/list/detail/items, tags/items, genres/items,
  search, source probe, playback decision, playback session inspect, and
  playback session cancel.
- Streaming/raw byte routes:
  explicitly deferred by `sdk_inventory_covers_foundation_public_routes_without_streaming_methods`
  and documented in `docs/api/HTTP_API.md`.
- Mockable transport:
  `ClientTransport` trait plus SDK tests using `MockTransport`.
- Public inventory/leakage:
  `PUBLIC_CLIENT_PATHS` plus
  `sdk_inventory_rejects_admin_internal_and_secret_surfaces`.
- Docs:
  `docs/api/HTTP_API.md`, `docs/GOALS.md`, `docs/ROADMAP.md`,
  `docs/README.md`, `docs/workstreams/README.md`, and this workstream.

### Non-Goals Preserved

- No crates.io publishing or release automation.
- No Flutter/Dart SDK, Web UI, Rust CLI product commands, or npm publishing.
- No full streaming body abstraction, download manager, HLS player,
  OAuth/OIDC/RBAC/user session.
- No server public API expansion.
- No SDK dependency on server/internal crates.

## Recorded Evidence

### RCS-010 Scope And Boundary Baseline

- Workstream docs define `crates/nako-client` as the SDK location, the
  Apache-2.0 license policy, dependency direction, route coverage, non-goals,
  task ledger, evidence anchors, and gate set.

### RCS-020 Crate Skeleton And Boundary Guard

- `crates/nako-client/Cargo.toml` defines the SDK crate with explicit
  `license = "Apache-2.0"`.
- The crate depends on `nako-client-protocol`, `reqwest`, `async-trait`,
  `serde`, `serde_json`, and `thiserror`.
- The crate does not depend on `nako-core`, `nako-api`, `nako-server`,
  `nako-streaming`, or `nako-transcode`.

### RCS-030 Async JSON Client Surface

- `crates/nako-client/src/lib.rs` exposes `NakoClient`, `ReqwestTransport`,
  `ClientTransport`, request/response transport records, `NakoClientError`,
  `PageQuery`, and playback/search query helpers.
- The SDK reuses `nako-client-protocol` DTOs for public JSON wire shapes.
- JSON methods cover health, libraries, catalog item browse/search, source
  probe, playback decision, playback session inspection, and playback session
  cancellation.
- Mock transport tests cover bearer auth, health without auth, API version
  mismatch, public error envelope parsing, pagination, URL joining, path
  encoding, playback capability query parameters, and cancellation method
  shape.

### RCS-040 Public Inventory And Leakage Checks

- `PUBLIC_CLIENT_PATHS` records the M35 JSON SDK route inventory.
- Tests assert the foundation inventory includes the supported JSON routes,
  excludes streaming/raw byte routes deferred from M35, and rejects
  admin/internal/secret/local-path terms.

### RCS-050 Docs And Closeout

- `docs/api/HTTP_API.md` documents both TypeScript and Rust client SDK
  boundaries.
- `docs/GOALS.md`, `docs/ROADMAP.md`, and `docs/workstreams/README.md` record
  M35 as completed and split streaming/publishing/client apps into follow-ons.
- Close-out validation on 2026-05-17:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -p nako-client --tests`: passed.
  - `cargo nextest run -p nako-client --no-fail-fast`: 7 tests passed.
  - `cargo tree -p nako-client`: passed; dependency tree includes
    `nako-client-protocol`, `reqwest`, `async-trait`, `serde`, `serde_json`,
    `thiserror`, and dev `tokio`, with no server/internal Nako crates.
  - `cargo tree -p nako-client-protocol`: passed; protocol dependency tree
    remains limited to `serde` and dev `serde_json`.
  - `npm run check --prefix sdk/typescript`: passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: 271 tests passed.
  - `git diff --check`: passed with Git CRLF normalization warnings only.
