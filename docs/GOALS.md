# Taru Goal Map

This file is the top-level tracker for current and upcoming engineering goals.
Workstream TODO files track tasks; this file tracks goal boundaries,
non-goals, exit criteria, and evidence.

## Goal Format

Each implementation goal should define:

- Objective: the user-visible or architecture-visible outcome.
- Deliverables: concrete files, APIs, crates, or documents expected to change.
- Non-goals: adjacent work intentionally left out.
- Exit criteria: observable behavior that must be true.
- Evidence: commands, tests, docs, or commit IDs proving completion.

Use one goal per meaningful milestone. A goal should be large enough to produce
a coherent commit, but small enough that validation remains clear.

## Numbering Policy

Goal numbers are historical identifiers, not dense release numbers. Do not
reuse earlier gaps such as M10-M12 or M17 for new work. New implementation
goals should use the next number after the highest documented completed or
proposed milestone.

## Current Goal

No active implementation goal is recorded in this file after M37 closeout.

Next recommended implementation goal:

- M38 should choose the next client-runtime risk: full Rust SDK streaming body
  abstraction, TypeScript SDK streaming/package parity, or Flutter/Dart SDK
  foundation.

## Completed Goals

### M37: Apache-2.0 Rust Client CLI Entrypoint

Status: completed.

Objective:

- Add the first concrete Rust client entrypoint after M35/M36 validated the
  SDK and shared public route inventory.
- Prove an external program can consume `taru-client` without depending on
  AGPL server/internal crates or reimplementing HTTP DTOs.
- Keep the new CLI Apache-2.0 and narrowly scoped to Public Client API usage.

Evidence:

- [client-cli workstream](workstreams/client-cli/README.md)
- `crates/taru-client-cli` is an Apache-2.0 CLI crate.
- The CLI uses `taru-client` as its Taru API entrypoint and does not depend on
  `taru-api`, `taru-server`, `taru-core`, `taru-streaming`, or
  `taru-transcode`.
- Commands cover health, libraries, items, search, source probe, playback
  decision, playback session get/cancel, and streaming request construction.
- Streaming commands print method, URL, and safe headers with bearer token
  values redacted; they do not execute streaming bodies or implement
  downloads/playback.
- Tests cover mocked SDK transport requests, query/path behavior, unauthenticated
  health preflight, authenticated public routes, token redaction, and manifest
  dependency boundaries.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  taru-client-cli --tests`, `cargo nextest run -p taru-client-cli
  --no-fail-fast` with 5 tests passed, `cargo tree -p taru-client-cli`,
  `cargo check --workspace --tests`, `cargo nextest run --workspace
  --no-fail-fast` with 279 tests passed, and `git diff --check`.
- Non-goals preserved: no crates.io publishing, installer, release automation,
  shell completions, TUI, player, HLS playback, download manager, cache,
  background sync, server-admin/internal CLI commands, Flutter/Dart SDK, Web UI,
  or mobile client.

### M36: Client SDK Contract Inventory and Streaming Request Builders

Status: completed.

Objective:

- Remove public client route inventory duplication between `taru-api`,
  TypeScript SDK generation, and `taru-client`.
- Move neutral public route facts into permissive `taru-client-protocol`
  without making clients depend on the AGPL `taru-api` crate.
- Add future-safe Rust SDK request builders for public streaming/raw byte
  routes without implementing body streaming, download management, or player
  behavior.

Evidence:

- [client-sdk-contract workstream](workstreams/client-sdk-contract/README.md)
- `taru-client-protocol` remains `Apache-2.0`, dependency-light, and owns
  `PUBLIC_CLIENT_ROUTES`, `PublicClientRoute`, `PublicClientHttpMethod`,
  `PublicClientRouteKind`, `PublicClientRustSdkExposure`,
  `public_client_paths`, `public_client_json_routes`, and
  `public_client_streaming_routes`.
- `taru-api` OpenAPI tests and TypeScript SDK generation consume the shared
  protocol inventory instead of a local path list.
- `taru-client` consumes the shared inventory and exposes request builders for
  direct stream GET, direct stream HEAD preflight, remux stream GET, HLS
  playlist GET, and HLS segment GET.
- Rust SDK builder tests cover method, path encoding, query serialization,
  bearer auth, and `Range` header behavior.
- Close-out validation: `cargo fmt --all -- --check`, focused check/nextest
  gates for `taru-client-protocol`, `taru-api`, and `taru-client`, `cargo
  nextest run -p taru-server http::tests::playback --no-fail-fast` with 16
  tests passed, `cargo check --workspace --tests`, `cargo nextest run
  --workspace --no-fail-fast` with 274 tests passed, `cargo tree -p
  taru-client-protocol`, `cargo tree -p taru-client`, `npm run check --prefix
  sdk/typescript`, and `git diff --check`.
- Non-goals preserved: no crates.io/npm publishing, no streaming body
  abstraction, no download manager, no HLS player, no Flutter/Dart SDK, no Rust
  CLI product command, and no server API behavior expansion.

### M35: Rust Client SDK Foundation

Status: completed.

Objective:

- Add the first Rust client SDK foundation after M29-M34 stabilized the Public
  Client API, OpenAPI contract, and TypeScript SDK package.
- Reuse permissive `taru-client-protocol` DTOs instead of duplicating Rust wire
  types from OpenAPI.
- Give future Rust CLI, integration tests, third-party tools, and automation
  clients a clean crate boundary for calling Taru public client APIs.

Evidence:

- [rust-client-sdk workstream](workstreams/rust-client-sdk/README.md)
- `crates/taru-client` is an Apache-2.0 SDK crate with explicit license
  metadata.
- `taru-client` depends on `taru-client-protocol` for public DTOs and does not
  depend on `taru-core`, `taru-api`, `taru-server`, `taru-streaming`, or
  `taru-transcode`.
- The SDK exposes `TaruClient`, `ReqwestTransport`, mockable
  `ClientTransport`, `TaruClientError`, pagination helpers, search/playback
  query helpers, bearer-token injection, API-version checking, and
  `ErrorResponse` parsing.
- JSON route methods cover health, libraries, catalog items/search, source
  probe, playback decision, playback session inspection, and playback session
  cancellation.
- Tests cover auth, health without auth, API-version mismatch, public error
  envelope parsing, pagination, URL/path behavior, playback query parameters,
  route inventory, streaming-route deferral, and internal/admin leakage
  rejection.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  taru-client --tests`, `cargo nextest run -p taru-client --no-fail-fast`
  with 7 tests passed, `cargo tree -p taru-client`, `cargo tree -p
  taru-client-protocol`, `npm run check --prefix sdk/typescript`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  271 tests passed, and `git diff --check`.
- Streaming/raw byte methods, crates.io publishing, Rust CLI commands,
  Flutter/Dart SDK, npm publishing, and concrete UI clients remain follow-ons.

### M34: TypeScript SDK Package Hardening and Contract Compile Check

Status: completed.

Objective:

- Turn the M33 TypeScript SDK scaffold generator into a minimal package with a
  repeatable generation command and a real TypeScript compile contract.
- Prove the generated SDK can be consumed as a future Web/CLI client API
  surface instead of only passing Rust-side string checks.
- Keep package hardening separate from npm publishing, concrete UI clients,
  Flutter/Dart SDK, and Rust SDK implementation.

Evidence:

- [typescript-sdk-package workstream](workstreams/typescript-sdk-package/README.md)
- `sdk/typescript` is a private TypeScript SDK package with local TypeScript
  tooling, strict `tsconfig.json`, committed generated `src/index.ts`, and
  package README.
- `npm run generate --prefix sdk/typescript` refreshes `src/index.ts` through
  `cargo run -q --manifest-path ../../Cargo.toml -p taru-api --example
  emit-typescript-sdk -- --output src/index.ts`.
- `npm run check --prefix sdk/typescript` runs `tsc --noEmit` against the
  generated SDK with strict settings.
- `taru-api` has a sync test that compares the committed package entry with
  `sdk::typescript_sdk()`.
- Close-out validation: `npm run generate --prefix sdk/typescript`, `npm run
  check --prefix sdk/typescript`, `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo check -p taru-api --examples`, `cargo nextest
  run -p taru-api --no-fail-fast` with 11 tests passed, `cargo nextest run
  --workspace --no-fail-fast` with 264 tests passed, `cargo tree -p
  taru-client-protocol`, and `git diff --check`.

### M33: SDK Generation and Client Integration Scaffold

Status: completed.

Objective:

- Establish a repeatable SDK/client integration scaffold after M32 OpenAPI v1.
- Prove future web, CLI, and Flutter work can start from the same public API
  contract instead of scattered handwritten HTTP calls.
- Produce a dependency-light TypeScript/Web/CLI SDK scaffold with bearer auth,
  API-version inspection, error envelope parsing, pagination helpers, and core
  public route methods.

Evidence:

- [sdk-client-scaffold workstream](workstreams/sdk-client-scaffold/README.md)
- `taru-api` owns `sdk::typescript_sdk()` and the
  `emit-typescript-sdk` example for generating a dependency-free
  TypeScript/Web/CLI client scaffold from the OpenAPI v1 contract.
- Generated scaffold covers API version constants, public path inventory,
  OpenAPI-derived wire interfaces, `TaruClient`, `TaruApiError`, bearer-token
  injection, `x-taru-api-version` inspection, error envelope parsing,
  pagination helpers, and core library/catalog/playback/session route calls.
- SDK generator tests cover route inventory, auth/version/error/pagination
  runtime details, and rejection of admin/internal/secret/local-path terms.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo check -p taru-api --examples`, `cargo
  nextest run -p taru-api --no-fail-fast` with 10 tests passed, `cargo
  nextest run --workspace --no-fail-fast` with 263 tests passed, `cargo tree
  -p taru-client-protocol`, and `git diff --check`.

### M32: OpenAPI and Public Client SDK Contract Foundation

Status: completed.

Objective:

- Establish a machine-readable Public Client API schema after the M29 public
  protocol, M30 version/error contract, and M31 bearer-auth boundary.
- Keep `taru-client-protocol` as the permissive public wire-type owner,
  `taru-api` as the AGPL adapter/schema aggregation layer, and `taru-server`
  as route wiring and behavior evidence.
- Produce the first verifiable OpenAPI v1 artifact for core future
  Flutter/web/CLI/SDK surfaces: health, library, catalog browse/search, source
  probe, playback decision, direct/remux/HLS playback, playback sessions, and
  M30/M31 error/auth envelopes.

Evidence:

- [openapi-client-contract workstream](workstreams/openapi-client-contract/README.md)
- [ADR 0025](adr/0025-openapi-public-client-sdk-contract.md)
- `taru-client-protocol` owns protocol DTOs for library detail and playback
  session responses.
- Public playback session responses no longer expose server-local output paths.
- `taru-api` owns `openapi::public_openapi_v1_json()` and the
  `emit-openapi` example for generating the OpenAPI JSON artifact.
- OpenAPI checker tests cover public route inventory, bearer auth,
  `x-taru-api-version`, shared `ErrorResponse`, pagination, and internal/admin
  leakage rejection.
- `taru-server` exposes and tests `GET /libraries/{library_id}` for the
  public library detail surface.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  260 tests passed, `cargo check -p taru-api --examples`, `cargo tree -p
  taru-client-protocol`, and `git diff --check`.

### M31: Access Boundary and Token Authentication Foundation

Status: completed.

Objective:

- Establish an inbound HTTP access boundary so future Flutter, web, CLI,
  remote access, and tunnel work does not depend on unauthenticated server
  APIs.
- Define the difference between Public Client API, Server Admin/Internal API,
  and outbound addon/provider/webhook integration auth.
- Implement the first bearer-token authentication foundation with safe
  defaults, local-development ergonomics, route-level tests, and no token
  leakage.

Evidence:

- [access-boundary-auth workstream](workstreams/access-boundary-auth/README.md)
- [ADR 0024](adr/0024-inbound-token-authentication-boundary.md)
- `taru-client-protocol` owns public `unauthorized` and `forbidden` error
  codes.
- `taru-server` config exposes `[auth]` with auth enabled by default and
  `TARU_ADMIN_TOKEN` as the default token environment reference.
- `taru-server` HTTP middleware protects every non-health route when auth is
  enabled, while `GET /health` remains public.
- Auth failures return M30-compatible `401 unauthorized` error envelopes with
  `WWW-Authenticate: Bearer` and no token leakage.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  256 tests passed, `cargo tree -p taru-client-protocol`, and `git diff
  --check`.

### M30: Public API Versioning and Error Envelope Hardening

Status: completed.

Objective:

- Stabilize the HTTP API version, error response, pagination/response
  envelope, and compatibility rules that future Flutter, web, CLI, and SDK
  clients will depend on.
- Clarify Public Client API vs Server Admin/Internal API boundaries for error
  codes, HTTP status mapping, version evolution, and deprecation policy.
- Make catalog/library/playback/system public route success and failure
  behavior test-visible and documentable.

Evidence:

- [public-api-contract workstream](workstreams/public-api-contract/README.md)
- [ADR 0023](adr/0023-public-api-versioning-and-error-envelope-contract.md)
- `taru-client-protocol` owns `ClientErrorCode`, `API_VERSION_HEADER`, and
  the compatible `ErrorResponse` envelope constructor.
- `taru-server` emits `x-taru-api-version: v1` and maps `TaruError` through
  protocol-owned public error codes.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  254 tests passed, `cargo tree -p taru-client-protocol`, and
  `git diff --check`.

### M29: Public Client API Contract and Catalog Browse Surface

Status: completed.

Objective:

- Expand `taru-client-protocol` into the first useful public client contract
  for library/catalog browse, search, list/detail, probe, and playback
  decision responses while keeping `taru-api` as the server adapter over
  internal models.

Evidence:

- [public-client-api workstream](workstreams/public-client-api/README.md)
- `taru-client-protocol` owns protocol DTOs with string wire IDs and public
  protocol enums.
- `taru-api` owns explicit mapping functions from `taru-core`,
  `taru-streaming`, and `taru-transcode`.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  253 tests passed, `cargo tree -p taru-client-protocol`, and
  `git diff --check`.

### M0-M2.1: Server Runtime Foundation

Status: completed.

Evidence:

- Rust workspace and crate stubs exist.
- SQLite persistence, server runtime, persisted jobs, pagination, logging, and
  developer docs are implemented.
- Related docs: [server-foundation milestones](workstreams/server-foundation/MILESTONES.md).

### M3.1-M3.6: Metadata, NFO, Profiles, and Catalog Planning

Status: completed for the first movie-focused foundation.

Evidence:

- Metadata merge policy, NFO policy, provider secret policy, library presets,
  catalog graph, scan state, and artwork resource-class ADRs exist.
- TMDB movie refresh, NFO import/export jobs, metadata profile execution, and
  catalog/search planning are implemented or documented.

### M4.0-M4.10: Catalog Ingestion and Playback MVP

Status: completed.

Evidence:

- Catalog ingestion, graph hydration, browse APIs, direct play, FFmpeg command
  planning, remux process runner guard, and remux application service
  integration are implemented.
- HTTP remux playback route is implemented.
- Remux/transcode session records are persisted in SQLite and exposed through
  an app/API lookup path.
- A minimal single-variant HLS transcode path can generate, persist, and serve
  playlists and segments.
- Hardware acceleration capability, policy, fallback, and resource-budget
  models are implemented without requiring real GPU hardware in tests.
- MVP stabilization audited API docs, config docs, error behavior, test gaps,
  performance constraints, and known limitations.
- Last completed implementation goal: M4.10 MVP stabilization.

## Recently Completed Goals

### Planning Docs: Goal Map and Refactoring Policy

Status: completed.

Objective:

- Give the project a single top-level route for roadmap, goal tracking,
  workstream ownership, and fearless refactoring policy.

Deliverables:

- `docs/README.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/workstreams/README.md`
- `docs/development/REFACTORING_POLICY.md`
- server-foundation milestone and TODO updates

Non-goals:

- no runtime code changes;
- no ADR status migration beyond documenting the hygiene rule;
- no workstream directory split yet.

Exit criteria:

- top-level docs link to current focus, roadmap, and active workstream;
- the next recommended implementation goal is explicit;
- refactoring policy documents crate boundaries, dependency direction, and
  validation gates;
- doc consistency checks pass.

Evidence:

- `git diff --check` passed for the docs-only change set.

### M4.5: Remux App Service Integration and Local Staging Policy

Status: completed.

Evidence:

- `taru-server::app` has a remux application service boundary.
- `remux_staging_root` config defines the local staging root.
- Remux outputs are deterministic by source ID and container.
- Completed staged outputs are reused.
- In-flight duplicate requests return `Conflict`.
- Tests cover app-service runner execution, completed-output reuse, duplicate
  conflict behavior, staging path validation, and config defaults.

### M4.6: Remux Playback Route

Status: completed.

Evidence:

- `GET /sources/{source_id}/stream/remux` is implemented.
- The handler calls the remux app service and streams staged output.
- `output_container=mp4|mkv` selects the staged remux container.
- Completed staged outputs are reused.
- In-flight duplicates map to `409 conflict`.
- Tests cover range streaming, completed-output reuse, duplicate conflict, and
  unchanged direct play behavior.

### M4.7: Playback Session Persistence

Status: completed.

Evidence:

- `transcode_sessions` persists remux and future transcode session state.
- Remux app-service requests create planned sessions, mark running sessions,
  and persist finished, failed, cancelled, and stale recovery states.
- Completed persisted remux sessions are reused after app restart.
- Active persisted sessions drive duplicate `409 conflict` behavior.
- `GET /playback/sessions/{session_id}` exposes current persisted state.

### M4.8: HLS Transcode Foundation

Status: completed.

Evidence:

- `taru-transcode` plans and runs minimal single-variant HLS sessions through
  FFmpeg.
- HLS output uses a staging layout with temporary directory promotion.
- HLS app service uses persisted transcode sessions for planned, running,
  finished, failed, cancelled, stale, duplicate, and reuse behavior.
- `GET /sources/{source_id}/stream/hls/playlist.m3u8` returns a rewritten HLS
  playlist.
- `GET /playback/sessions/{session_id}/hls/segments/{segment_name}` serves
  generated segments with path traversal protection.

### M4.9: Hardware Acceleration Policy

Status: completed.

Evidence:

- `taru-transcode` has a hardware acceleration capability report, detector
  boundary, policy selection, fallback behavior, and resource-budget model.
- HLS command planning can select CPU-only, VAAPI, NVENC, or QuickSync encoder
  arguments without requiring real hardware in tests.
- `taru-server` config exposes hardware acceleration, fallback, CPU slots, and
  GPU slots with conservative defaults.
- HLS app-service concurrency uses CPU/GPU resource budgets based on the
  selected acceleration class.

### M4.10: MVP Stabilization

Status: completed.

Evidence:

- HTTP API docs match the current local playback routes, including remux, HLS,
  persisted session lookup, and playback error behavior.
- Local setup docs cover scan, probe, metadata, remux, HLS staging, hardware
  policy, and CPU/GPU resource budget configuration.
- Test strategy docs reflect current coverage for browse, metadata/NFO, direct
  play, remux, HLS, persisted playback sessions, and hardware policy.
- Known MVP limitations are documented in the phase note.
- Focused HLS session readiness tests cover active-session conflict behavior at
  the app and HTTP layers.

## Recently Completed Goal

### M5: Extension and Automation Surface

Status: completed.

Implement webhook outbox, automation jobs, addon manifest schema, and one
reference addon. Keep AI-like experience improvements as explicit external
provider/API-key workflows rather than local model or vector infrastructure.

Deliverables:

- M5.0 Extension/Automation Design Baseline.
- M5.1 Event Outbox Foundation.
- M5.2 Webhook Delivery Worker.
- M5.3 Automation Job Model.
- M5.4 Addon Manifest and Resource Contract.
- M5.5 Reference Addon and Stabilization.

Non-goals:

- no local model runtime or vector database;
- no in-process native plugin ABI;
- no embedded JavaScript runtime in the first M5 slice;
- no remote storage backend implementation.

Evidence for M5.0:

- [ADR 0014](adr/0014-durable-event-outbox-for-webhooks-and-automation.md)
  documents durable event outbox and webhook/automation trigger policy.
- [ADR 0015](adr/0015-capability-scoped-http-addons-and-automation-providers.md)
  documents capability-scoped HTTP addons and external automation providers.
- [addons-automation workstream](workstreams/addons-automation/README.md)
  tracks M5 milestones, TODOs, phase notes, resource classes, and security
  boundaries.

Evidence for M5.1:

- `taru-core` defines domain event kinds, event subjects, outbox status, event
  records, and `EventOutboxRepository`.
- `taru-db` migration `0009_event_outbox.sql` persists durable outbox events
  with idempotency by event kind and key.
- `taru-server` writes outbox events for successful library scan, metadata
  refresh, NFO import/export, and playback session completion paths.
- Tests cover outbox persistence, idempotency, and payload safety constraints
  against plaintext secrets and raw local paths.

Evidence for M5.2:

- `taru-core` defines webhook endpoint configuration, delivery attempt records,
  statuses, and `WebhookRepository`.
- `taru-db` migration `0010_webhooks.sql` persists webhook endpoints and
  delivery attempts with per-event inspection.
- `taru-events` builds versioned webhook envelopes, signs payloads with
  HMAC-SHA256, enforces request timeouts, records failed attempts with retry
  timestamps, and provides a `reqwest` transport.
- `taru-server` exposes webhook endpoint configuration/inspection, per-event
  delivery-attempt inspection, explicit outbox event dispatch, and
  `webhook_concurrency` resource budgeting.
- Tests cover SQLite persistence, signed success delivery, failed retry state,
  real transport delivery to a mocked local webhook server, and HTTP
  configuration/inspection routes.

Evidence for M5.3:

- `taru-core` defines automation provider configuration, automation
  capabilities, job input/summary envelopes, artifact records, and
  `AutomationRepository`.
- `taru-db` migration `0011_automation.sql` persists provider configuration and
  generated artifacts.
- `taru-automation` runs mockable external providers through a timeout and
  cancellation-aware runner, persists proposed artifacts, writes job summaries,
  and rejects implicit canonical metadata mutation.
- `taru-server` exposes provider configuration, automation job enqueue, and
  artifact inspection APIs without calling external providers inline.
- Tests cover provider/artifact persistence, mocked provider execution, secret
  omission from job input, canonical-mutation rejection, and HTTP enqueue and
  inspection routes.

Evidence for M5.4:

- `taru-addon-protocol` defines the manifest, protocol version, resource
  declarations, scopes, auth modes, request/response envelopes, mockable
  transport, `ReqwestAddonTransport`, and bounded resource caller.
- `taru-core` defines addon registration status and records plus
  `AddonRepository`.
- `taru-db` migration `0012_addons.sql` persists addon registrations, manifest
  snapshots, granted scopes, and enabled/disabled status.
- `taru-server` exposes addon registration, list, status-filtered list, and
  detail APIs. Registrations are disabled by default and rejected when the
  manifest or granted scopes do not satisfy the resource contract.
- Tests cover manifest validation, invalid manifest rejection, scope denial,
  auth token enforcement, bounded retry behavior, response envelope mapping,
  persistence, and HTTP registration/inspection routes.

Evidence for M5.5:

- `taru-reference-addon` provides a minimal local metadata addon fixture with
  a valid manifest and HTTP resource route.
- `taru-server` end-to-end tests register the reference addon through
  `POST /addons`, query it through `GET /addons/{addon_id}`, and call the
  metadata resource through `ReqwestAddonTransport`.
- Addon author, webhook receiver, and automation provider guides document the
  current extension surface.
- [Phase 5.5](workstreams/addons-automation/PHASE5_5_REFERENCE_ADDON_STABILIZATION.md)
  documents M5 known limitations and stabilization evidence.

### M6.0: Remote Storage and VFS Design Baseline

Status: completed.

Objective:

- Define the remote-storage architecture before adding WebDAV or S3-compatible
  backend code.

Deliverables:

- ADR 0016 for remote storage and VFS cache boundaries.
- Dedicated `storage-vfs` workstream.
- Local-path dependency audit for `taru-vfs`, scan/probe, direct play, remux,
  and HLS.
- M6 milestone split with WebDAV selected as the first backend preview.
- Roadmap, goal map, ADR index, and workstream index updates.

Evidence:

- [ADR 0016](adr/0016-remote-storage-and-vfs-cache-boundary.md) documents
  WebDAV-first remote storage, VFS cache, staging, credential, and local-path
  boundaries.
- [storage-vfs workstream](workstreams/storage-vfs/README.md) owns M6 remote
  storage, VFS cache, remote staging, and playback policy work.
- [Phase 6.0](workstreams/storage-vfs/PHASE6_0_REMOTE_STORAGE_DESIGN_BASELINE.md)
  records the local-path dependency audit and M6 milestone split.

### M6.1: WebDAV Read-Only VFS Backend

Status: completed.

Evidence:

- `taru-vfs::WebDavBackend` implements read-only `stat`, `list`, and
  `open_range`.
- `VfsLibraryScanner` can scan a mocked WebDAV library without plaintext
  credentials in source locators.
- [Phase 6.1](workstreams/storage-vfs/PHASE6_1_WEBDAV_READ_ONLY_BACKEND.md)
  records validation and limitations.

### M6.2: Directory and Stat Cache

Status: completed.

Evidence:

- `taru-core` defines VFS cache object, listing, failure, and repository
  contracts.
- `taru-db` migration `0013_vfs_cache.sql` persists cached stat/list metadata
  and transient failure state.
- `taru-vfs::CachedStorageBackend` reuses fresh cache and serves stale cache on
  transient storage errors.
- `LibraryIndexService` skips tombstoning when a scan used stale VFS cache.
- [Phase 6.2](workstreams/storage-vfs/PHASE6_2_DIRECTORY_STAT_CACHE.md)
  records validation and remaining cache gaps.

### M6.3: Remote Probe Staging

Status: completed.

Evidence:

- `taru-vfs` defines `StageRequest`, `StagedFile`, deterministic staging paths,
  and `StorageBackend::stage`.
- `taru-vfs::WebDavBackend` can stage a remote media object to a deterministic
  local path and reuse it when size still matches.
- `LibraryProbeService` uses staging when a backend returns no local path hint.
- [Phase 6.3](workstreams/storage-vfs/PHASE6_3_REMOTE_PROBE_STAGING.md)
  records validation and remaining staging gaps.

### M6.4: Remote Playback Policy

Status: completed.

Evidence:

- `StorageBackend::read_range` gives direct play a VFS byte path when a source
  has no local path hint.
- `taru-vfs::WebDavBackend` uses HTTP `Range` GET for byte windows.
- Remux and HLS input planning stages remote sources under
  `remux_staging_root/inputs` before invoking FFmpeg.
- Tests cover remote direct-play bytes, remote FFmpeg staging, local path-hint
  reuse, WebDAV range GET, and WebDAV staging.
- [Phase 6.4](workstreams/storage-vfs/PHASE6_4_REMOTE_PLAYBACK_POLICY.md)
  records validation and remaining production config/API gaps.

### M6.5: Remote Storage Stabilization

Status: completed.

Evidence:

- `TaruServerConfig` supports `[library.webdav]` preview configuration with
  WebDAV root, base URL, username, password environment reference, timeout,
  and retry attempt limits.
- `taru-server::app` builds configured WebDAV storage through
  `WebDavBackend` wrapped in `CachedStorageBackend`.
- Configured WebDAV library scan/probe uses the configured library root;
  remote probe staging uses
  `remux_staging_root/probe-inputs`.
- HTTP API and local setup docs describe WebDAV direct play, remux/HLS staging,
  secret references, and preview limitations.
- [Phase 6.5](workstreams/storage-vfs/PHASE6_5_REMOTE_STORAGE_STABILIZATION.md)
  records validation and remaining known limitations.

## Recently Completed Goals

### M7: Playback Streaming and Remote Hardening

Status: completed.

Objective:

- Make remote playback practical after the M6 WebDAV preview by removing
  direct-play byte buffering, bounding staged remote inputs, improving playback
  failure visibility, adding remote playback resource budgets, and replacing
  the single-library preview shape with explicit multi-library configuration.

Deliverables:

- M7.0 Playback Streaming Design Baseline.
- M7.1 Remote Direct Body Streaming.
- M7.2 Staging Manifest, Disk Budget, and Cleanup.
- M7.3 Playback Error Taxonomy and HTTP Mapping.
- M7.4 Remote Playback Resource Budgets.
- M7.5 Multi-Library and Multi-Remote Backend Config.
- M7.6 Playback Streaming Stabilization.

Non-goals:

- no remote write/delete support;
- no direct FFmpeg remote URL input before a separate accepted design;
- no adaptive bitrate ladder in the first M7 slice;
- no client UI work before server playback contracts stabilize.

Evidence for M7.0:

- [ADR 0017](adr/0017-playback-streaming-and-remote-hardening-boundaries.md)
  documents playback streaming, staging, error mapping, resource budget, and
  configuration boundaries.
- [playback-streaming workstream](workstreams/playback-streaming/README.md)
  tracks M7 milestones, TODOs, phase notes, resource classes, and boundary
  rules.
- [Phase 7.0](workstreams/playback-streaming/PHASE7_0_PLAYBACK_STREAMING_DESIGN_BASELINE.md)
  records the M6 starting point and M7 implementation sequence.

Recommended next implementation goal:

- Start M8 multi-library correctness and operational hardening.

Evidence for M7.1 foundation:

- `taru-vfs` defines `ReadStream` and `StorageBackend::stream_range`.
- `WebDavBackend::stream_range` proxies remote byte streams without
  accumulating chunks into an in-memory direct-play body.
- `taru-server` direct play returns `DirectPlaySourceBody::Stream` for remote
  sources without local path hints, while local sources still use local file
  streaming.
- `HEAD /sources/{source_id}/stream` uses a preflight plan without opening the
  direct-play body.
- Playback app planning and HTTP response helpers are split into
  `crates/taru-server/src/app/playback.rs` and
  `crates/taru-server/src/http/playback.rs`.
- [Phase 7.1](workstreams/playback-streaming/PHASE7_1_REMOTE_DIRECT_BODY_STREAMING.md)
  records validation and remaining gaps.

Evidence for M7.2 foundation:

- `taru-core` defines staging manifest purpose, state, record, and repository
  contracts.
- `taru-db` migration `0014_staging_manifest.sql` persists staging manifest
  records.
- `taru-db/src/staging.rs` implements the staging repository in a dedicated DB
  module instead of growing the large `lib.rs`.
- `taru-server` records `probe_input` manifest entries when remote probe inputs
  are staged, using an app-side VFS wrapper rather than coupling
  `taru-library` to the staging repository.
- `taru-server` records `ffmpeg_input` manifest entries when remote WebDAV
  sources are staged for remux or HLS.
- `[staging].max_bytes` config and the app-side staging wrapper enforce a
  manifest-backed disk budget before remote probe or FFmpeg input staging.
- `[staging].retention_ms` and `[staging].cleanup_on_startup` drive startup
  cleanup of expired staged inputs; cleanup preserves active leases.
- [Phase 7.2](workstreams/playback-streaming/PHASE7_2_STAGING_MANIFEST_FOUNDATION.md)
  records validation and remaining runtime gaps.

Evidence for M7.3 first error-mapping slice:

- Playback/storage HTTP errors now expose stable codes for staging budget
  exhaustion, staging validation mismatch, storage timeout, storage
  unauthorized, storage rate limiting, and FFmpeg failures.
- [Phase 7.3](workstreams/playback-streaming/PHASE7_3_PLAYBACK_ERROR_MAPPING.md)
  records validation and remaining typed-error gaps.

Evidence for M7.4 resource-budget foundation:

- `[playback].remote_stream_concurrency` and
  `[playback].remote_stage_concurrency` define independent remote playback
  budgets.
- Remote direct-play holds a stream permit for the streamed response body, and
  remote probe/FFmpeg staging acquires a stage permit around staging.
- [Phase 7.4](workstreams/playback-streaming/PHASE7_4_REMOTE_PLAYBACK_RESOURCE_BUDGETS.md)
  records validation and remaining route-level stress-test gaps.

Evidence for NFO/VFS storage boundary:

- `run_nfo_import` and `run_nfo_export` now use
  `storage_backend_for_library_root`.
- NFO export checks `StorageCapabilities::WRITABLE`; WebDAV import works
  through the configured VFS backend and WebDAV export is rejected as read-only.
- [Phase 7.4.1](workstreams/playback-streaming/PHASE7_4_1_NFO_STORAGE_BOUNDARY.md)
  records validation.

Evidence for M7.5 multi-library backend foundation:

- `TaruServerConfig` uses `[[libraries]]` as the only server library
  configuration model.
- Server startup upserts every configured library.
- `MediaSource.library_id` gives scan/probe/NFO/playback/FFmpeg staging a
  direct library identity for backend resolution.
- Mixed local/WebDAV library parsing and runtime backend resolution are covered
  by config and app-level tests.
- [Phase 7.5](workstreams/playback-streaming/PHASE7_5_MULTI_LIBRARY_BACKENDS.md)
  records migration shape and known limitations.

Evidence for M7.6 stabilization:

- [Phase 7.6](workstreams/playback-streaming/PHASE7_6_STABILIZATION_AUDIT.md)
  maps every M7 completion criterion to concrete code, tests, docs, and
  validation gates.

### M8: Multi-Library Correctness and Operational Hardening

Status: completed.

Objective:

- Make multi-library operation data-safe by scoping source locator identity to
  the library, exposing explicit CLI multi-library commands, closing the remote
  staging disk-budget race, and documenting the new invariants.

Deliverables:

- `media_sources` uniqueness is `(library_id, locator)` instead of global
  `locator`.
- Repository source lookup by locator requires `library_id`.
- Local scan/index/probe/search tests cover two libraries with the same
  relative media path and the same resulting `local:///` locator.
- CLI supports `scan --library-id`, `scan-all`, and `list --library-id`.
- Staging budget check, staging, and manifest recording are serialized under a
  shared budget lock.
- The panic-style default library helper is replaced with
  `default_library_from_config`.
- [Phase 8.0](workstreams/multi-library-hardening/PHASE8_0_CORRECTNESS_BASELINE.md)
  records source identity, CLI, and staging budget invariants.

Later follow-up:

- M13-M23 completed the metadata, runtime, database, storage, ingestion, and
  API boundary hardening needed before the M24 server architecture pass.

### M13-M14: Metadata Maintenance and Scheduling

Status: completed.

Evidence:

- [metadata-operations milestones](workstreams/metadata-operations/MILESTONES.md)
  track library-scale maintenance jobs, scheduling, lifecycle, provider
  diagnostics, and raw-cache cleanup.
- [Phase 13.0](workstreams/metadata-operations/PHASE13_0_MAINTENANCE_JOB_BOUNDARY.md)
  and [Phase 14.0](workstreams/metadata-operations/PHASE14_0_SCHEDULING_AND_LIFECYCLE.md)
  record the implemented boundaries.

### M15-M16: Runtime Foundation and Storage Lease Lifecycle

Status: completed.

Evidence:

- [runtime-foundation milestones](workstreams/runtime-foundation/MILESTONES.md)
  track SQLite runtime behavior, migration execution, secret redaction,
  hardware selection, storage backend registry, and staged-input lease
  lifecycle.
- [Phase 15.0](workstreams/runtime-foundation/PHASE15_0_RUNTIME_HARDENING_BASELINE.md),
  [Phase 15.1](workstreams/runtime-foundation/PHASE15_1_RUNTIME_HARDENING_IMPLEMENTATION.md),
  and [Phase 16](workstreams/runtime-foundation/PHASE16_STORAGE_BACKEND_REGISTRY_AND_LEASE_LIFECYCLE.md)
  record the implementation evidence.

### M18-M19: Provider Runtime and Database Boundary Hardening

Status: completed.

Evidence:

- [Phase 18.0](workstreams/metadata-operations/PHASE18_0_PROVIDER_RUNTIME_PRODUCTIZATION.md)
  records the shared metadata provider runtime, secret resolution, and
  provider configuration cleanup.
- [Phase 19.0](workstreams/runtime-foundation/PHASE19_0_DATABASE_BOUNDARY_HARDENING.md)
  records the SQLite repository split, transaction boundaries, and database
  module cleanup.

### M20-M23: Server Surface, Storage, Ingestion, and API Boundary Cleanup

Status: completed.

Evidence:

- [server-foundation milestones](workstreams/server-foundation/MILESTONES.md)
  track M20-M23.
- [Phase 20.0](workstreams/server-foundation/PHASE20_0_SERVER_SURFACE_DECOMPOSITION.md)
  split oversized app and HTTP tests by bounded context.
- [Phase 21.0](workstreams/server-foundation/PHASE21_0_STORAGE_BACKEND_REGISTRY.md)
  recorded storage backend registry ownership.
- [Phase 22.0](workstreams/server-foundation/PHASE22_0_INGESTION_FAILURE_DIAGNOSTICS.md)
  recorded durable ingestion failure diagnostics.
- [Phase 23.0](workstreams/server-foundation/PHASE23_0_API_HTTP_DB_BOUNDARY_CLEANUP.md)
  recorded API DTO, HTTP router, and DB module cleanup.

## Latest Completed Goal

### M24: Server Architecture Hardening

Status: completed.

Objective:

- Turn `taru-server` back into a thin composition root with focused
  application services, explicit background-worker lifecycle ownership, clear
  repository and transaction boundaries, and no obsolete MVP helper paths.

Deliverables:

- [ADR 0019](adr/0019-server-architecture-hardening-boundaries.md) for server
  composition, service, supervisor, and repository boundaries.
- [server-architecture-hardening workstream](workstreams/server-architecture-hardening/README.md)
  with M24 milestones, TODOs, and a baseline phase note.
- App-service decomposition that moves workflow orchestration out of
  `TaruApp`.
- Runtime supervisor or worker registry for background jobs and cleanup loops.
- Repository/transaction cleanup for multi-record writes and broad concrete
  store dependencies.
- Removal of obsolete single-library, compatibility, or temporary helper code.

Non-goals:

- no new metadata provider feature work;
- no client implementation;
- no split into multiple deployable services;
- no in-process plugin ABI design;
- no adaptive bitrate playback ladder;
- no compatibility shims for deprecated shapes unless they have a testable
  migration purpose.

Exit criteria:

- `TaruApp` is a composition root rather than the main feature orchestration
  object.
- HTTP handlers call focused application services and keep response/error
  translation local to HTTP modules.
- Background workers are registered through one lifecycle owner with
  cancellation and failure visibility.
- Multi-record write sequences have explicit repository or unit-of-work
  boundaries.
- Obsolete MVP helpers are removed after their replacement invariants are
  covered by tests.
- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`

Evidence for M24.0:

- [ADR 0019](adr/0019-server-architecture-hardening-boundaries.md) documents
  the target server architecture boundaries.
- [server-architecture-hardening workstream](workstreams/server-architecture-hardening/README.md)
  tracks M24 milestones, TODOs, phase notes, and refactor policy.
- [Phase 24.0](workstreams/server-architecture-hardening/PHASE24_0_SERVER_ARCHITECTURE_BASELINE.md)
  records the starting surfaces and implementation sequence.

Evidence for M24.1-M24.4:

- [Phase 24.1](workstreams/server-architecture-hardening/PHASE24_1_IMPLEMENTATION_SLICE.md)
  records the service decomposition, runtime supervisor, catalog transaction
  boundary, removed root-app forwards, and NFO structured parser migration.
- `TaruApp` now composes focused service handles for jobs, library scan/probe,
  library administration, catalog, storage diagnostics, metadata, NFO,
  playback, addon, automation, and webhook workflows.
- Metadata, library scan, NFO jobs, metadata lifecycle loops, and staging lease
  cleanup use `RuntimeSupervisor`; webhook delivery is request-scoped
  structured concurrency and automation enqueue is synchronous.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`: 229 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Latest Completed Goal

### M25: Transcode Runtime Productization

Status: completed.

Objective:

- Turn playback/transcode from an MVP HLS/remux implementation into a clean
  runtime product boundary for hardware acceleration, session orchestration,
  resource budgets, and future adaptive streaming.

Deliverables:

- Create a dedicated transcode runtime workstream and design baseline.
- Decompose the large playback application service into focused direct-play,
  remux, HLS, staging, and transcode-runtime modules.
- Replace the CPU-only server hardware detector with an FFmpeg-backed
  capability probe when hardware acceleration is configured.
- Make VAAPI, NVENC, and QuickSync selection, fallback, and resource budget
  behavior explicit API/service contracts.
- Define the stable client-facing playback session lifecycle and error model
  before Flutter or web client work depends on it.

Non-goals:

- no adaptive bitrate ladder implementation in the first slice;
- no client UI implementation;
- no direct FFmpeg remote credential input until a separate storage security
  design is accepted.

Evidence:

- [transcode-runtime workstream](workstreams/transcode-runtime/README.md)
  tracks the M25 module split, runtime contracts, and post-M25 follow-ups.
- [Phase 25.1](workstreams/transcode-runtime/PHASE25_1_RUNTIME_PRODUCTIZATION.md)
  records the playback service decomposition, FFmpeg hardware detector, CPU/GPU
  budget selection, session lifecycle, validation evidence, and known follow-up
  work.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p taru-server --no-fail-fast`: 90 tests passed.
- `cargo nextest run --workspace --no-fail-fast`: 231 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Latest Completed Goal

### M26: Playback API Contract and Client Readiness

Status: completed.

Objective:

- Stabilize playback/session HTTP contracts before future web or Flutter
  clients depend on them.

Deliverables:

- Add a public playback session cancellation route.
- Wire cancellation to live remux/HLS FFmpeg runner tokens, not only persisted
  session rows.
- Keep inspection and successful cancellation on `TranscodeSessionResponse`.
- Document active/terminal playback session lifecycle states and stable error
  DTO behavior.
- Validate with route-level tests for active cancellation, terminal conflicts,
  process-local stale active-session conflicts, session inspection, and HLS
  segment readiness/error behavior.

Non-goals:

- no adaptive bitrate ladder;
- no client UI implementation;
- no distributed transcode queue or cross-process cancellation coordinator.

Evidence:

- [Phase 26.0](workstreams/transcode-runtime/PHASE26_0_PLAYBACK_CLIENT_CONTRACT.md)
  records the playback client contract scope, cancellation semantics, and
  validation gates.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check -p taru-server --tests`
- `cargo check --workspace --tests`
- `cargo nextest run -p taru-server http::tests::playback --no-fail-fast`: 16
  playback route tests passed.
- `cargo nextest run --workspace --no-fail-fast`: 234 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Latest Completed Goal

### M27.3: Hierarchy Confirmation and Provider/NFO Expansion Slice

Status: completed.

Objective:

- Build on M27.2's **Local Inference Evidence** and **Provisional Hierarchy**
  so NFO and built-in providers can confirm series, season, and episode items
  in place instead of replacing Taru item identity.

Deliverables:

- add a shared **Hierarchy Confirmation** service boundary for provider/NFO
  confirmation of provisional hierarchy;
- confirm provisional series, season, and episode items in place without
  replacing Taru `MediaItem` identity;
- write accepted **Provider Mapping** records when metadata provider refresh
  succeeds;
- connect NFO episode import to the shared confirmation service while
  preserving local/NFO authority;
- add TMDB series, season, and episode provider fetch support;
- keep Douban and Bangumi MVPs inside the shared **Provider Subject** and
  **Provider Mapping** boundary.

Non-goals:

- no Source Variant UI;
- no browse API;
- no artwork candidate, selected artwork, or managed artwork expansion;
- no rating, user state, or browse facet work;
- no automatic duplicate merge;
- no general **Hierarchy Repair** flow.

Evidence:

- [Phase 27.3](workstreams/metadata-catalog/PHASE27_3_HIERARCHY_CONFIRMATION_PROVIDER_NFO.md)
  records the hierarchy confirmation, provider mapping, TMDB series/season/
  episode, and NFO episode confirmation slice.
- `taru-metadata` owns the shared **Hierarchy Confirmation** service boundary.
- Metadata refresh writes accepted **Provider Subject** and **Provider
  Mapping** records for successful TMDB, Douban, and Bangumi fetches.
- `taru-nfo` confirms provisional episode hierarchy in place through the
  shared service.
- [metadata-catalog TODO](workstreams/metadata-catalog/TODO.md) marks the
  provider breadth checklist complete.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p taru-db --no-fail-fast`: 32 tests passed.
- `cargo nextest run -p taru-library --no-fail-fast`: 15 tests passed.
- `cargo nextest run -p taru-metadata --no-fail-fast`: 26 tests passed.
- `cargo nextest run -p taru-nfo --no-fail-fast`: 8 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

Next recommended implementation goal:

- M28 crate boundary and public protocol hardening.

## Latest Completed Goal

### M27.1: Catalog Schema and Repository Slice

Status: completed.

Objective:

- Turn the M27.0 metadata-catalog domain baseline into durable `taru-core`
  records, `taru-db` schema, repository traits, SQLite adapters, and focused
  repository tests without adding provider breadth.

Deliverables:

- persist **Provider Subject** and **Provider Mapping** separately from Taru
  **Media Item** identity;
- persist **Source Duplicate Relationship** separately from source identity
  and item merging;
- persist minimal **Local Inference Evidence** for inferred kind, title, year,
  season, episode, confidence, evidence source, and inference version;
- cover the selected video item hierarchy and multi-source item link behavior
  through repository tests;
- keep existing movie MVP `MediaItem` and `MediaSource` behavior compatible.

Evidence:

- [Phase 27.1](workstreams/metadata-catalog/PHASE27_1_CATALOG_SCHEMA_REPOSITORY_SLICE.md)
  records the schema/repository implementation and M27.2 boundaries.
- `crates/taru-db/migrations/0018_metadata_catalog_domain.sql` adds the
  durable catalog-domain tables.
- [metadata-catalog TODO](workstreams/metadata-catalog/TODO.md) marks the
  M27.1 checklist complete.

Close-out validation:

- `cargo nextest run -p taru-db`: 31 tests passed.
- `cargo nextest run -p taru-core`: 3 tests passed.
- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `git diff --check`: passed with Git CRLF normalization warnings only.

Next recommended implementation goal:

- M27.2 local inference and provisional hierarchy slice.

### M27.0: Metadata-Catalog Domain Baseline

Status: completed.

Objective:

- Turn the movie-first metadata and catalog foundation into a video-first
  media-server model using the project language defined in `CONTEXT.md` and
  ADR 0021.

Why this came next:

- the playback/runtime contract is now stable enough for client planning;
- the remaining product risk is the metadata domain shape, not FFmpeg or HTTP;
- the current `server-foundation` backlog mixes metadata, NFO, artwork, and
  search follow-ups that should be owned by a dedicated workstream.

Deliverables:

- create a `metadata-catalog` workstream;
- decide the first stable **Media Item** hierarchy for movie, series, season,
  episode, **Episode-Like Item**, **Extra Item**, **Franchise Collection**,
  and unknown video items;
- define **Provider Subject** and **Provider Mapping** rules for TMDB, Douban,
  Bangumi, and future provider/addon evidence;
- decide how **Media Domain** and **Library Preset** influence defaults without
  becoming item identity;
- decide the source-to-item and duplicate-source model;
- separate **Canonical Metadata**, **Media Technical Facts**, **Library Item
  State**, and **User Playback State**;
- define **Metadata Source Priority**, **NFO Round Trip**, **Browse Facet**,
  and **Sort Key** rules;
- define client-facing artwork concepts and search expansion boundaries;
- move the relevant TODO items out of `server-foundation`.

Non-goals:

- no schema migrations;
- no provider feature implementation;
- no runtime behavior changes;
- no public API changes.

Evidence:

- [Phase 27.0](workstreams/metadata-catalog/PHASE27_0_METADATA_CATALOG_DOMAIN_BASELINE.md)
  records the current code audit, baseline decisions, and M27.1/M27.2 handoff.
- [ADR 0021](adr/0021-video-first-media-server-domain-model.md) is accepted.
- [metadata-catalog TODO](workstreams/metadata-catalog/TODO.md) marks the
  M27.0 design-baseline checklist complete.
- [server-foundation TODO](workstreams/server-foundation/TODO.md) no longer
  owns active metadata/catalog/artwork/search follow-ups.

Close-out validation:

- `git diff --check`: passed with Git CRLF normalization warnings only

Next recommended implementation goal:

- M27.1 catalog schema and repository slice.
