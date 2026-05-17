# Client SDK Contract Inventory And Streaming Builders Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Starting Repro

- `taru-api` owns `public_client_paths()` locally in its AGPL OpenAPI module.
- `taru-client` owns a second `PUBLIC_CLIENT_PATHS` list.
- Rust SDK streaming/raw byte routes are public API routes but M35 only
  documents them as deferred.

## Gate Set

### Protocol Boundary Gate

```bash
cargo check -p taru-client-protocol --tests
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo tree -p taru-client-protocol
```

### API And TypeScript SDK Gate

```bash
cargo check -p taru-api --tests
cargo nextest run -p taru-api --no-fail-fast
npm run check --prefix sdk/typescript
```

### Rust SDK Gate

```bash
cargo check -p taru-client --tests
cargo nextest run -p taru-client --no-fail-fast
cargo tree -p taru-client
```

### Workspace Closeout Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Anchors

- `crates/taru-client-protocol/src/lib.rs`
- `crates/taru-api/src/openapi.rs`
- `crates/taru-api/src/sdk.rs`
- `crates/taru-client/src/lib.rs`
- `docs/api/HTTP_API.md`
- `docs/workstreams/client-sdk-contract/`

## Prompt-To-Artifact Checklist

- Move public route inventory into permissive protocol crate.
- Keep `taru-client-protocol` Apache-2.0 and dependency-light.
- Make `taru-api` consume shared inventory without moving OpenAPI rendering out
  of the AGPL adapter crate.
- Make TypeScript generator consume the shared inventory through `taru-api`.
- Make `taru-client` consume shared inventory.
- Add streaming request builders for direct stream, HEAD preflight, remux, HLS
  playlist, and HLS segment.
- Preserve non-goals: no streaming body abstraction, no downloader, no player,
  no publishing, no Flutter/Dart/Rust CLI product work.
- Validate route drift, leakage, license/dependency direction, and docs.

## Recorded Evidence

### CSC-010 Scope And License Baseline

- Workstream docs define `taru-client-protocol` as the shared inventory owner,
  `taru-client` as the Rust SDK boundary, and `taru-api` as the AGPL
  OpenAPI/SDK generation adapter.

### CSC-020 Shared Public Route Inventory

- `crates/taru-client-protocol/src/lib.rs` defines `PublicClientRoute`,
  `PublicClientHttpMethod`, `PublicClientRouteKind`,
  `PublicClientRustSdkExposure`, `PUBLIC_CLIENT_ROUTES`,
  `public_client_paths`, `public_client_json_routes`, and
  `public_client_streaming_routes`.
- `taru-client-protocol` tests prove the inventory has 24 public paths, 20
  Rust JSON-method routes, 4 Rust streaming-builder routes, GET/HEAD coverage
  for direct stream, and no admin/internal/secret/local-path terms.
- `crates/taru-api/src/openapi.rs` no longer owns a local public path list for
  OpenAPI route membership tests.
- `crates/taru-api/src/sdk.rs` consumes `taru_client_protocol::public_client_paths`
  for TypeScript SDK path generation and tests.
- Validation so far:
  - `cargo check -p taru-client-protocol --tests`: passed.
  - `cargo check -p taru-api --tests`: passed.
  - `cargo nextest run -p taru-client-protocol --no-fail-fast`: 7 tests passed.
  - `cargo nextest run -p taru-api --no-fail-fast`: 11 tests passed.

### CSC-030 Rust SDK Inventory And Streaming Builders

- `crates/taru-client/src/lib.rs` consumes
  `taru_client_protocol::public_client_paths`,
  `public_client_json_routes`, `public_client_streaming_routes`,
  `PUBLIC_CLIENT_ROUTES`, and `PublicClientRustSdkExposure`.
- Removed the local `taru-client` `PUBLIC_CLIENT_PATHS` duplication.
- Added Rust SDK request builders:
  - `stream_source_request`;
  - `head_stream_source_request`;
  - `remux_stream_source_request`;
  - `hls_playlist_request`;
  - `hls_segment_request`.
- Added `RemuxPlaybackQuery` for remux capability and output-container query
  serialization.
- Builder tests cover method, path encoding, query serialization, bearer auth,
  and `Range` header behavior without executing streaming response bodies.
- Validation:
  - `cargo check -p taru-client --tests`: passed.
  - `cargo nextest run -p taru-client --no-fail-fast`: 8 tests passed.

### CSC-040 Cross-SDK Drift And Leakage Gates

- `cargo nextest run -p taru-api --no-fail-fast`: 11 tests passed.
- `cargo nextest run -p taru-client --no-fail-fast`: 8 tests passed.
- `npm run check --prefix sdk/typescript`: passed.
- `cargo tree -p taru-client-protocol`: passed; the protocol crate remains
  dependency-light with `serde` plus dev `serde_json`.
- `cargo tree -p taru-client`: passed; the SDK depends on protocol/runtime
  crates and does not depend on `taru-api` or server/internal Taru crates.

### CSC-050 Docs And Closeout

- `docs/api/HTTP_API.md` documents protocol-owned route inventory and Rust SDK
  streaming request builders.
- `docs/GOALS.md`, `docs/ROADMAP.md`, and `docs/workstreams/README.md` record
  M36 as completed and split full streaming bodies, package publishing, Rust
  CLI, Dart/Flutter SDK, and concrete clients into follow-ons.
- Closeout validation on 2026-05-17:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -p taru-client-protocol --tests`: passed.
  - `cargo check -p taru-api --tests`: passed.
  - `cargo check -p taru-client --tests`: passed.
  - `cargo nextest run -p taru-client-protocol --no-fail-fast`: 7 tests
    passed.
  - `cargo nextest run -p taru-api --no-fail-fast`: 11 tests passed.
  - `cargo nextest run -p taru-client --no-fail-fast`: 8 tests passed.
  - `cargo nextest run -p taru-server http::tests::playback --no-fail-fast`:
    16 tests passed after widening the remux fixture start wait.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: 274 tests passed.
  - `cargo tree -p taru-client-protocol`: passed; protocol dependency tree
    remains `serde` plus dev `serde_json`.
  - `cargo tree -p taru-client`: passed; no `taru-api` or server/internal
    Taru crate dependency.
  - `npm run check --prefix sdk/typescript`: passed.
  - `git diff --check`: passed with Git CRLF normalization warnings only.

## Completion Audit

### Objective Requirements

- Eliminate route inventory duplication:
  `taru-api` and `taru-client` now consume protocol-owned inventory instead of
  owning local public path lists.
- Move route inventory into permissive protocol crate:
  `taru-client-protocol` owns `PUBLIC_CLIENT_ROUTES` and stays Apache-2.0.
- Keep OpenAPI and TypeScript SDK aligned:
  `taru-api` OpenAPI tests and `typescript_sdk()` use
  `taru_client_protocol::public_client_paths`.
- Add Rust SDK streaming request builders:
  `taru-client` exposes builders for direct stream GET, direct stream HEAD,
  remux stream GET, HLS playlist GET, and HLS segment GET.
- Preserve no-body-streaming boundary:
  SDK builders return `ClientRequest`; they do not send requests or own
  response bodies.
- Preserve license boundary:
  client-reusable inventory lives in `taru-client-protocol`, not only in AGPL
  `taru-api`; dependency trees confirm permissive crates do not depend on
  server/internal crates.

### Named Files And Artifacts

- `docs/workstreams/client-sdk-contract/`: workstream docs and closeout.
- `crates/taru-client-protocol/src/lib.rs`: route inventory source of truth.
- `crates/taru-api/src/openapi.rs`: consumes protocol inventory for OpenAPI
  path checks.
- `crates/taru-api/src/sdk.rs`: consumes protocol inventory for TypeScript SDK
  generation and tests.
- `crates/taru-client/src/lib.rs`: consumes protocol inventory and implements
  streaming request builders.
- `docs/api/HTTP_API.md`: documents shared inventory and builder behavior.
- `docs/GOALS.md`, `docs/ROADMAP.md`, `docs/workstreams/README.md`: record M36
  completion and follow-ons.

### Non-Goals Preserved

- No crates.io or npm publishing.
- No full streaming response abstraction, download manager, HLS player, or
  playback UI.
- No Flutter/Dart SDK, Rust CLI product commands, Web/mobile UI.
- No server public API behavior expansion.
- No dependency from permissive protocol/SDK crates to AGPL server/internal
  crates.
