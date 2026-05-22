# Taru Roadmap

Taru is currently a Rust modular monolith focused on the self-hosted media
server backend. The roadmap is intentionally staged so storage, metadata,
playback, search, automation, and future clients can grow without collapsing
into a single tightly coupled crate.

Goal numbers are historical identifiers. Earlier gaps such as M10-M12 and M17
are not reused; new work uses the next number after the highest documented
milestone.

## Current Architecture Focus

Status: no active architecture/productization focus is set after Addon Install
Guide Generation closeout.

Latest completed architecture focus:

Addon Install Guide Generation turns the completed Addon Operations backend and
Admin Web surface into an operator-friendly deployment guide for externally run
Addon Sidecars:

- server-owned Admin API guide generation for registered Addons;
- Docker Compose and systemd snippets as inert text guidance;
- Secret Reference checklist without resolved secret values;
- Addon Health Check and registration verification steps;
- Admin Web preview through the generated Admin API TypeScript contract.

This focus is intentionally not Addon Manager work. Taru will not install,
launch, stop, restart, update, remove, log, or supervise Addon Sidecar
processes in this lane.

Authoritative lane:

- `docs/workstreams/addon-install-guide-generation/`

Previous architecture focus:

Admin Web Addon Operations closed on 2026-05-22. Admin Addon Operations MVP had
already closed the backend lifecycle, health, surfaces, and diagnostics
capabilities; this lane exposed those capabilities safely in the Admin Web
Console:

- generated Admin API TypeScript contract coverage for Addon Operations;
- live-capable Admin Web Addon data-source seam with safe mock fallback;
- Addon list/detail operations surface;
- enable/disable, **Addon Health Check**, and resource-call diagnostic actions;
- manifest surface rendering for **Addon Entry Points**, **Addon Hosted
  Pages**, configuration schema metadata, **Addon Tasks**, and **Addon Event
  Subscriptions**.

Authoritative lane:

- `docs/workstreams/admin-web-addon-operations/`

Previous architecture focus:

Admin Addon Operations MVP closed on 2026-05-21. Release Packaging and Addon
Architecture Deepening were already complete, and this lane productized
operator control and diagnostics for manually registered Addon Sidecars:

- enable/disable lifecycle mutation;
- terminal unregister semantics;
- redaction-safe Addon Health Check;
- hosted Addon surface read models;
- bounded resource-call diagnostics;
- redaction-safe Admin DTOs and docs.

Authoritative lane:

- `docs/workstreams/admin-addon-operations-mvp/`

Previous architecture focus:

Addon Architecture Deepening completed on 2026-05-21. It intentionally removed
compatibility seams while Taru still has no deployed users and deepened Addon
boundaries before broader Addon breadth:

- Addon Side Effect runtime lifecycle;
- fingerprinted Addon Side Effect idempotency;
- Protected Write payload protocol contracts;
- Addon Manifest declarations;
- Library File Write runtime;
- Admin Addon API DTO shielding under `/admin/v1/addons`;
- `taru-addon-protocol` / `taru-addon-client` crate separation;
- SQLite/PostgreSQL Addon Side Effect fingerprint schema parity.

Authoritative lane:

- `docs/workstreams/addon-architecture-deepening/`

Earlier architecture focus:

M63 was the fearless architecture-deepening lane after M62 PostgreSQL
Production Readiness. It intentionally prioritized deeper Modules and workflow
seams before new feature breadth:

- Addon Side Effect Module depth;
- Addon Canonical Metadata commit atomicity;
- Library ingestion workflow depth;
- Playback/transcode request identity;
- hardware diagnostics;
- search semantics;
- test locality around touched Interfaces.

Authoritative lane:

- `docs/workstreams/fearless-architecture-deepening/`

Closeout:

- Completed on 2026-05-20.
- Final workspace nextest passed with 498 tests run and 19 skipped.
- PostgreSQL opt-in contracts were skipped because no
  `TARU_TEST_POSTGRES_URL` was available.

Latest completed follow-on:

- `docs/workstreams/admin-api-typescript-contract/`

## Phase Bands

### Foundation: M0-M2.1

Status: completed.

The repository has a Rust workspace, crate boundaries, SQLite persistence,
minimal server runtime, persisted jobs, pagination, logging, local setup,
testing strategy, and license notes.

### Metadata and Catalog: M3.1-M4.1

Status: completed for the first movie-focused slice.

Taru can scan local sources, persist metadata, merge TMDB/NFO inputs through
local-authority rules, hydrate a normalized catalog graph, rebuild search
projection, and expose browse APIs for items, people, tags, genres, credits,
and images.

Important remaining breadth:

- TMDB series, season, and episode support.
- Douban provider MVP.
- Bangumi provider MVP.
- item-level metadata profile overrides.
- image proxy/cache routes and preview-frame generation jobs.

### Playback and Transcode: M4.2-M4.10

Status: completed for the local video-library playback MVP.

Completed:

- playback decision model;
- direct play byte-range route and HEAD preflight;
- direct play planning boundary;
- FFmpeg copy-remux command planning;
- remux session lifecycle model;
- remux FFmpeg process runner with cancellation, timeout, concurrency guard,
  temporary output cleanup, and server runtime budget configuration.
- remux application service with local staging, deterministic output naming,
  completed-output reuse, in-flight duplicate conflict behavior, and API-safe
  error mapping.
- HTTP remux playback route backed by the remux application service.
- persisted remux/transcode session records with startup stale-session
  recovery and lookup API.
- minimal single-variant HLS command planning, session orchestration, playlist
  route, and segment route.
- hardware acceleration capability, fallback, and CPU/GPU budget policy for
  VAAPI, NVENC, and QuickSync command planning.
- MVP stabilization for API docs, config docs, route error behavior, test
  coverage, known limitations, and bounded resource notes.

Next phase:

- M5 extension and automation surface. Completed.

Future playback work:

- adaptive bitrate HLS ladder.

### Extension and Automation: M5

Status: completed.

This phase turns the early architectural decisions into a usable external
surface:

- webhook outbox and delivery policy;
- automation job model for API-key backed providers;
- Taru addon manifest schema;
- addon resource routes and response envelopes;
- timeout, retry, authentication, and trust model;
- one reference addon.

Completed:

- M5.0 design baseline with ADRs for durable event outbox, webhook/automation
  trigger policy, capability-scoped HTTP addons, external provider boundaries,
  resource classes, and security constraints.
- M5.1 event outbox foundation with domain event records, SQLite persistence,
  repository boundary, idempotent enqueue behavior, and write points from scan,
  metadata, NFO, and playback session completion.
- M5.2 webhook delivery worker with endpoint configuration, delivery attempts,
  signed webhook envelopes, retry/backoff state, `webhook_concurrency`
  budgeting, explicit event dispatch, and delivery inspection APIs.
- M5.3 automation job model with external provider configuration, automation
  job inputs/summaries, generated artifact persistence, timeout/cancellation
  runner boundary, and provider/job/artifact inspection APIs.
- M5.4 addon manifest and resource contract with protocol versioning, resource
  envelopes, scope grants, disabled-by-default registration, bounded addon
  HTTP caller, SQLite persistence, and registration/inspection APIs.
- M5.5 reference addon and stabilization with a local addon fixture, end-to-end
  register/query/call test, addon/webhook/automation guides, and documented M5
  limitations.

Next phase:

- M6 remote storage and VFS expansion. Completed.

### Remote Storage and VFS Expansion: M6

Status: completed.

This phase proves that remote sources are first-class storage backends instead
of pretending to be local paths:

- WebDAV read-only backend preview;
- directory and stat cache;
- remote byte-range reads and local staging path;
- remote listing rate limits and retry policy;
- remote-source playback policy.

Completed:

- M6.0 design baseline with ADR 0016, a dedicated `storage-vfs` workstream,
  local-path dependency audit, WebDAV-first decision, and M6 milestone split.
- M6.1 WebDAV read-only VFS backend.
- M6.2 directory/stat cache and stale-cache tombstone protection.
- M6.3 remote probe staging.
- M6.4 remote playback policy.
- M6.5 remote storage stabilization.

Recommended next goal:

- M7 playback streaming and remote hardening. Completed.

### Playback Streaming and Remote Hardening: M7

Status: completed.

This phase hardens the remote playback path after the M6 WebDAV preview:

- remote direct response-body streaming;
- staging manifest, disk budget, and cleanup;
- precise playback/storage error mapping;
- remote playback stream and staging resource budgets;
- multi-library and multi-remote backend configuration.

Completed:

- M7.0 design baseline with ADR 0017, a dedicated `playback-streaming`
  workstream, M7 milestone split, and ownership for M6 deferred playback
  hardening tasks.
- M7.1 foundation for remote direct response-body streaming, including VFS
  `ReadStream`, WebDAV byte streams, direct-play stream bodies, HEAD preflight,
  and initial playback app/http module split.
- M7.2 foundation for durable staging manifest persistence, including core
  records, repository contract, SQLite migration, a dedicated DB module,
  remote probe input manifest recording, and remux/HLS FFmpeg input manifest
  recording.
- M7.2 disk budget wiring with `[staging].max_bytes` and app-side preflight
  checks before remote probe or FFmpeg input staging.
- M7.2 startup cleanup for expired staged inputs, with active-lease protection.
- M7.3 first HTTP error-mapping slice for staging budget, staging validation,
  storage timeout/auth/rate-limit, and FFmpeg provider failures.
- M7.4 resource-budget foundation with independent remote stream and stage
  concurrency limits.
- NFO import/export now use the configured VFS backend boundary and gate export
  on writable storage capabilities.
- M7.5 multi-library backend foundation with `[[libraries]]` as the only
  library configuration shape, startup persistence for all configured
  libraries, and `MediaSource.library_id` for library-aware backend resolution.
- M7.6 stabilization audit mapping the full M7 objective to concrete evidence,
  known limitations, and validation gates.

Recommended next goal:

- M8 multi-library correctness and operational hardening. Completed.

### Multi-Library Correctness and Operational Hardening: M8

Status: completed.

This phase made multi-library operation data-safe before the next server
architecture pass:

- source locator identity is scoped by library;
- source lookup by locator requires library identity;
- CLI scan/list commands expose explicit library selection;
- staging budget reservation is serialized across check, stage, and manifest
  recording;
- panic-style default library helpers were replaced with explicit config
  selection.

Completed:

- [Phase 8.0](workstreams/multi-library-hardening/PHASE8_0_CORRECTNESS_BASELINE.md)
  records source identity, CLI, and staging budget invariants.

Later follow-up:

- M13-M23 completed the metadata, runtime, database, storage, ingestion, and
  API boundary hardening needed before the M24 server architecture pass.

### Operational and Boundary Hardening: M13-M23

Status: completed.

After the first remote playback and multi-library hardening waves, Taru added
several focused operational cleanup phases before the final server composition
pass:

- M13-M14 metadata maintenance, scheduling, provider diagnostics, and raw-cache
  lifecycle.
- M15-M16 runtime foundation, SQLite/migration behavior, secret redaction,
  hardware selection policy, storage backend registry, and staged-input lease
  lifecycle.
- M18 metadata provider runtime productization.
- M19 database boundary hardening.
- M20 server test-surface decomposition.
- M21 storage backend registry documentation in the server-foundation stream.
- M22 ingestion failure diagnostics.
- M23 API, HTTP router, and DB boundary cleanup.

The detailed evidence lives in the `metadata-operations`, `runtime-foundation`,
and `server-foundation` workstreams.

### Server Architecture Hardening: M24

Status: completed.

This phase turns the server into a cleaner modular-monolith composition
boundary:

- `taru-server::app::TaruApp` becomes a thin composition root;
- workflow orchestration moves into focused application services;
- background jobs and cleanup loops register through an explicit runtime
  supervisor or worker registry;
- high-level services use narrow ports or service handles instead of broad
  concrete store access;
- multi-record writes get explicit repository or unit-of-work boundaries;
- obsolete MVP helpers and compatibility paths are deleted once replacements
  are covered.

Completed:

- M24.0 design baseline with ADR 0019, a dedicated
  `server-architecture-hardening` workstream, M24 milestone split, and audit
  notes for server composition, runtime ownership, repository boundaries, NFO,
  catalog hydration, and obsolete helpers.
- M24.1-M24.4 implementation has decomposed `TaruApp` into focused service
  handles, added runtime supervisor ownership for detached workers, moved
  catalog graph hydration behind an atomic repository operation, removed
  temporary root-app forwards, and replaced hand-written NFO XML parsing with a
  `roxmltree` parser boundary.

Completed:

- M25 transcode runtime productization.

### Transcode Runtime Productization: M25

Status: completed.

This phase turns the existing remux/HLS MVP into a cleaner playback and
transcode runtime boundary before client work depends on it:

- split playback application code into focused direct-play, remux, HLS,
  staging, and runtime modules;
- replace the CPU-only detector with FFmpeg-backed hardware capability probing
  when hardware acceleration is configured;
- make VAAPI, NVENC, and QuickSync selection, fallback, and resource budgets
  explicit contracts;
- define stable playback session lifecycle, error categories, and client-facing
  URL behavior;
- keep adaptive bitrate HLS ladder as a follow-up after the runtime boundary is
  clean.

Completed:

- M25.1 split playback orchestration into focused direct-play, FFmpeg input,
  remux, and HLS app modules.
- M25.2 replaced CPU-only runtime detection with FFmpeg-backed encoder
  capability probing for VAAPI, NVENC, and QuickSync/QSV.
- M25.3 documented the session lifecycle and validated acceleration fallback,
  fail-fast policy, CPU/GPU resource budgets, runner timeout/cancellation, and
  stale-startup recovery behavior.

### Playback Client Contract: M26

Status: completed.

This phase hardens the server contract that future web or Flutter clients will
depend on before client UI work dominates:

- keep playback session inspection on a stable `TranscodeSessionResponse`
  envelope;
- add a public playback session cancellation route backed by live remux/HLS
  runner cancellation tokens;
- document active and terminal playback session states, cancellation conflict
  behavior, and error DTOs;
- validate the route behavior with focused HTTP tests.

Future client and product experience work remains intentionally deferred until
the server browse and playback contracts are coherent.

### Metadata-Catalog Expansion: M27

Status: completed for M27.0 design baseline, M27.1 schema/repository slice,
M27.2 local inference/provisional hierarchy slice, and M27.3 provider/NFO
expansion.

M27.0 turned the movie-first metadata/catalog foundation into a video-first
media-server model using the project language in `CONTEXT.md` and ADR 0021:

- treat **Media Library**, **Media Domain**, **Library Preset**, **Media
  Source**, **Media Item**, **Canonical Metadata**, **Media Technical Facts**,
  **Metadata Source Priority**, **NFO Round Trip**, **Managed Artwork**, and
  **Source Duplicate Relationship** as first-class planning terms;
- split the remaining metadata, NFO, artwork, and search follow-ups out of the
  historical `server-foundation` backlog into a dedicated
  `metadata-catalog` workstream;
- design the movie, series, season, episode, **Episode-Like Item**,
  **Extra Item**, and **Franchise Collection** model before adding TMDB series,
  Douban, or Bangumi breadth;
- map TMDB, Douban, Bangumi, and future provider concepts through **Provider
  Subject** and **Provider Mapping**, not provider-owned item identity;
- define stable **Browse Facets** and **Sort Keys** instead of exposing raw
  database columns as client query contracts;
- preserve local/NFO authority and library-scoped policy while opening a clean
  path for provider and addon metadata contributions.

Completed implementation slice:

- M27.1 catalog schema and repository slice persisted **Provider Subject**,
  **Provider Mapping**, **Source Duplicate Relationship**, and **Local
  Inference Evidence** records through `taru-core` records, `taru-db`
  migration `0018_metadata_catalog_domain.sql`, repository traits, SQLite
  adapters, and focused repository tests.
- M27.2 local inference and provisional hierarchy connected scan indexing to
  source-owned **Local Inference Evidence**, added weak-name **Unknown Media
  Item** fallback, keeps inference evidence as a current source/version
  snapshot, preserves confirmed canonical metadata during rescan, and creates
  provisional series/season/episode hierarchy without provider, NFO, Source
  Variant UI, or browse API breadth.
- M27.3 hierarchy confirmation and provider/NFO expansion added a shared
  confirmation service, in-place provisional hierarchy confirmation, accepted
  provider mapping writes for TMDB/Douban/Bangumi metadata refreshes, TMDB
  series/season/episode fetch support, and NFO episode hierarchy confirmation.

Recommended next goal:

- M28 crate boundary and public protocol hardening.

### Crate Boundary And Public Protocol Hardening: M28

Status: completed.

This phase deepens Taru's crate and module seams so public client wire types
can live in a permissive protocol crate while server internals remain AGPL and
the large workflow crates become easier to navigate.

Completed slices:

- M28.0 boundary baseline and scope freeze.
- M28.1 public client protocol extraction.
- M28.2 core module deepening and repository seam narrowing.
- M28.3 library and NFO module decomposition.
- M28.4 playback seam clarification.
- M28.5 closeout and follow-on split.

Recommended next goal:

- M29 public client API contract and catalog browse surface.

### Public Client API Contract: M29

Status: completed.

This phase grows the permissive client protocol boundary into a useful API
contract for future Flutter, web, and CLI clients:

- move the first stable library/catalog browse DTOs into
  `taru-client-protocol`;
- define browse/search/list/detail wire response envelopes;
- move public playback decision response types without exposing
  `taru_streaming` internals;
- keep `taru-api` as the AGPL adapter layer that maps server/domain records
  into protocol DTOs;
- keep server-admin diagnostics, job internals, provider runtime state,
  webhook, automation, and addon administration out of the first public
  protocol expansion.

Completed slices:

- M29.0 scope and evidence freeze.
- M29.1 public browse protocol DTO slice.
- M29.2 public playback decision DTO slice.
- M29.3 contract docs and route evidence.
- M29.4 closeout.

Recommended next goal:

- M30 public API versioning and error envelope hardening.

### Public API Contract Hardening: M30

Status: completed.

This phase hardens the HTTP compatibility contract that future Flutter, web,
CLI, and SDK clients will depend on after M29's public DTO extraction:

- define public API v1 version identity and compatibility rules;
- move stable public error-code vocabulary into the permissive protocol
  boundary;
- keep the existing `code/message` error envelope compatible while making
  codes test-visible and protocol-owned;
- document Public Client API vs Server Admin/Internal API compatibility
  boundaries;
- validate catalog/library/playback/system success envelopes, pagination
  metadata, and error status/code behavior with route-level tests.

Completed slices:

- M30.0 scope and contract baseline.
- M30.1 protocol error vocabulary slice.
- M30.2 server error mapping and version identity slice.
- M30.3 public route contract evidence slice.
- M30.4 closeout.

Recommended next goal:

- M31 should choose one concrete client-readiness risk: OpenAPI/SDK generation,
  auth/session boundary design, or deeper public route coverage based on the
  next client implementation need.

### Access Boundary And Token Authentication: M31

Status: completed.

This phase establishes an inbound HTTP access boundary before future clients,
remote access, or tunnel/NAT traversal depend on unauthenticated server APIs:

- define inbound client/admin auth as separate from addon, webhook, metadata,
  automation, and storage outbound integration secrets;
- add bearer-token config with auth enabled by default and token resolution
  from an environment reference;
- keep `GET /health` public for readiness/preflight;
- protect non-health routes with `Authorization: Bearer <token>`;
- return M30-compatible public error envelopes for missing or invalid tokens;
- document local development setup and follow-ons for users, sessions, RBAC,
  OpenAPI auth schemes, and tunnel integration.

Completed slices:

- M31.0 scope and boundary baseline.
- M31.1 protocol and config slice.
- M31.2 HTTP middleware slice.
- M31.3 docs and route evidence slice.
- M31.4 closeout.

Recommended next goal:

- M32 OpenAPI and public client SDK contract foundation.

### OpenAPI And Public Client SDK Contract: M32

Status: completed.

This phase turns the stabilized public protocol, version/error envelope, and
bearer-auth boundary into a machine-readable Public Client API contract:

- define OpenAPI/Public Client SDK contract ownership across
  `taru-client-protocol`, `taru-api`, and `taru-server`;
- keep public wire DTOs protocol-owned and avoid server/internal record leakage;
- add an OpenAPI v1 artifact/generator for core future Flutter, web, CLI, and
  SDK route surfaces;
- express `x-taru-api-version`, bearer auth, `ErrorResponse`, pagination, and
  common public errors in the schema;
- verify that the public spec excludes admin/internal routes, local path
  fields, secret references, raw provider cache, job internals, addon,
  webhook, automation, and storage diagnostics.

Completed slices:

- M32.0 scope and boundary baseline.
- M32.1 protocol response hygiene slice.
- M32.2 OpenAPI artifact slice.
- M32.3 server route contract evidence slice.
- M32.4 closeout.

Recommended next goal:

- M33 SDK generation and client integration scaffold.

### SDK Generation And Client Integration Scaffold: M33

Status: completed.

This phase turns the OpenAPI v1 contract into the first repeatable client
integration scaffold:

- generate a dependency-free TypeScript/Web/CLI client wrapper from
  `taru-api`;
- standardize bearer auth, `x-taru-api-version` inspection, error envelope
  parsing, pagination helpers, and core public route calls;
- add static checks that the scaffold follows the public route inventory and
  excludes admin/internal routes;
- keep Dart/Flutter SDK generation, package publishing, and UI implementation
  as follow-ons.

Completed slices:

- M33.0 scope and boundary baseline.
- M33.1 TypeScript SDK scaffold generator.
- M33.2 SDK contract smoke checks.
- M33.3 docs and closeout.

Recommended next goal:

- M34 TypeScript SDK package hardening and contract compile check.

### TypeScript SDK Package Hardening: M34

Status: completed.

This phase turns the M33 TypeScript SDK scaffold into a minimal compile-checked
package:

- add `sdk/typescript` as the private TypeScript SDK package location;
- keep generated SDK source in `sdk/typescript/src/index.ts`;
- add package-local TypeScript tooling and a committed lockfile;
- add repeatable generation and strict compile scripts;
- make `taru-api` test that the package entry matches the Rust generator;
- keep npm publishing, Flutter/Dart SDK, Rust SDK, and concrete clients as
  follow-ons.

Completed slices:

- M34.0 scope and boundary baseline.
- M34.1 strict compile fix.
- M34.2 package skeleton and generation command.
- M34.3 contract sync checks.
- M34.4 docs and closeout.

Recommended next goal:

- M35 Rust Client SDK Foundation. Completed.

### Rust Client SDK Foundation: M35

Status: completed.

This phase adds the first Rust SDK crate for public client consumers without
making clients depend on server internals:

- `crates/taru-client` is the Apache-2.0 Rust SDK runtime crate;
- public DTOs come from `taru-client-protocol`, not OpenAPI-generated Rust
  duplicates;
- the SDK owns HTTP mechanics such as base URL normalization, bearer auth,
  `x-taru-api-version` checking, public error envelope parsing, pagination,
  playback capability query serialization, and path encoding;
- `ClientTransport` makes the SDK testable without a live server, while
  `ReqwestTransport` provides the default runtime HTTP backend;
- JSON route methods cover health, libraries, catalog items/search, source
  probe, playback decision, playback session inspection, and playback session
  cancellation;
- route inventory and leakage tests keep admin/internal/secret/local-path
  surfaces out of the SDK.

Streaming/raw byte body APIs, crates.io publishing, Rust CLI commands,
Flutter/Dart SDK work, and concrete web/mobile clients remain separate
follow-ons.

Recommended next goal:

- M36 client SDK inventory extraction and streaming request builders.
  Completed.

### Client SDK Contract Inventory And Streaming Builders: M36

Status: completed.

This phase removes route inventory duplication and extends the Rust SDK to
cover streaming route construction while preserving the license boundary:

- public route inventory now lives in the Apache-2.0
  `taru-client-protocol` crate;
- `taru-api` still owns OpenAPI rendering and TypeScript SDK generation, but it
  consumes the protocol-owned inventory;
- `taru-client` consumes the same inventory and distinguishes JSON methods
  from streaming request builders;
- Rust SDK builders cover direct stream GET, direct stream HEAD preflight,
  remux stream GET, HLS playlist GET, and HLS segment GET;
- builder tests cover methods, auth headers, range headers, path encoding, and
  playback/remux query serialization;
- full streaming body abstraction, download manager, HLS player, SDK
  publishing, Rust CLI, and Flutter/Dart SDK remain follow-ons.

### Client CLI Entrypoint: M37

Status: completed.

This phase adds the first concrete public client program on top of the Rust SDK
instead of expanding directly into Flutter or full streaming body ownership:

- `crates/taru-client-cli` is Apache-2.0;
- the CLI uses `taru-client` as the only Taru client API entrypoint;
- commands cover health, library/item/search browse, source probe, playback
  decision, playback session get/cancel, and streaming request construction;
- streaming commands print method, URL, and safe headers without executing
  streaming bodies or implementing downloads/playback;
- dependency gates keep AGPL server/internal crates out of the client CLI.

Completed:

- M37.0 opened the `client-cli` workstream and fixed the license boundary.
- M37.1 added `crates/taru-client-cli` as an Apache-2.0 crate.
- M37.2 added focused tests for mocked SDK transport behavior, streaming
  request output, bearer-token redaction, and dependency boundaries.
- M37.3 documented usage and validated the workspace.

Full streaming response abstraction, download manager, Rust SDK publishing,
Flutter/Dart SDK, and concrete Web/mobile clients remain separate follow-ons.

### Server Runtime Deepening: M38

Status: completed.

This phase deepens the server startup and runtime job seams before the next
client or playback breadth:

- startup side effects move from `TaruApp::new_with_store` into a
  `ServerStartupWorkflow`;
- startup reports become the test surface for migration, recovery, cleanup,
  configured library persistence, metadata raw-cache cleanup, and lifecycle
  task registration;
- `RuntimeSupervisor` gains a durable job execution helper and job
  success/failure diagnostics;
- library scan, metadata refresh, and metadata maintenance background jobs use
  the runtime job helper;
- playback source selection, NFO round-trip preservation, broad repository
  trait splitting, and public client changes remain follow-ons.

Completed:

- M39 repository seam deepening.

### Repository Seam Deepening: M39

Status: completed.

This phase deepens repository seams by introducing workflow-shaped ports rather
than mechanically splitting every broad repository trait:

- catalog hydration becomes the first workflow port slice;
- `CatalogHydrationPort` hides the catalog graph/search persistence details
  behind snapshot, lookup, and commit operations;
- metadata refresh, hierarchy confirmation, and NFO import use the catalog
  hydration port instead of carrying the full catalog/media/search trait
  combination;
- SQLite schema, public API/SDK/CLI, playback decisions, and NFO Round Trip
  remain unchanged.

Completed:

- M39.1 added `CatalogHydrationPort` in `taru-catalog` with snapshot, lookup,
  and commit operations.
- M39.2 narrowed metadata refresh, hierarchy confirmation, and NFO import
  bounds to the workflow port.
- M39.3 validated the focused crates and workspace with 285 nextest tests
  passed.

Completed:

- M40 metadata refresh workflow port and provider runtime seam deepening.

### Metadata Refresh Seam: M40

Status: completed.

This phase continues repository seam deepening after M39:

- metadata refresh becomes the next workflow-port slice;
- refresh strategy, hierarchy confirmation, provider mapping, raw cache, and
  attempt records are audited as one workflow surface;
- the first implementation slice should hide real persistence detail without
  mechanically splitting every `MetadataRepository` method;
- provider breadth, public API/SDK/CLI, NFO Round Trip, playback, and DB schema
  changes remain unchanged.

Completed:

- M40.1 added `MetadataRefreshPort` and `MetadataAttemptPort` in
  `taru-metadata`.
- M40.2 moved refresh persistence, provider mapping, raw response, and
  library-item confirmation behind `commit_refresh`.
- M40.3 added a fake-port refresh test without SQLite while preserving
  existing metadata behavior.

### Durable Job Recovery: M41

Status: completed.

This phase fixes the durable job correctness gap left after M38:

- unfinished queued/running jobs are recovered at server startup;
- startup reports expose recovered durable job count;
- SQLite and server startup tests lock down the behavior;
- the unused old catalog search projection seam is removed if it has no caller;
- generic job retry/dispatch, public API changes, and `CatalogHydrationPort`
  lookup deepening remain follow-ons.

Completed:

- M41.1 added `JobRepository::fail_unfinished_jobs` and SQLite recovery for
  queued/running jobs.
- M41.2 wired recovery into `ServerStartupWorkflow` and exposed
  `ServerStartupReport::recovered_jobs`.
- M41.3 removed the unused old `rebuild_search_projection` catalog entrypoint.
- M41.4 validated focused gates and the workspace with 288 nextest tests
  passed.

### Catalog Hydration Lookup Deepening: M42

Status: completed.

This phase deepens the M39 catalog hydration seam:

- `CatalogHydrationPort` should express a workflow-level hydrate operation;
- snapshot, lookup, and commit internals should stay inside `taru-catalog`;
- metadata and NFO workflow tests should not construct lookup match vectors;
- existing catalog graph/search behavior remains unchanged;
- database schema, public API, SDK, client, and provider breadth remain
  unchanged.

Completed:

- M42.1 changed `CatalogHydrationPort` into a workflow-level hydrate
  operation.
- M42.2 narrowed metadata fake-port tests so they no longer model lookup
  internals.
- M42.3 validated focused catalog, metadata, and NFO gates plus the full
  workspace.

Recommended next goal after M42:

- Review and either adopt or defer the proposed Android client foundation
  workstream, or continue server-side playback/transcode seam deepening if
  server architecture remains the priority.

### Future-Ready Architecture Refactor: M61

Status: completed.

This phase reopens server-side architecture work with an explicit fearless
refactor mandate before Taru has production compatibility burden:

- make persistence PostgreSQL-ready instead of SQLite-shaped;
- replace the `SqliteStore` god-adapter shape with backend-neutral persistence
  contracts, contract tests, and explicit transaction/unit-of-work seams;
- slim `TaruApp` through cohesive runtime modules where they hide real
  construction or policy complexity;
- separate Media Source discovery from Local Inference and provisional
  hierarchy planning;
- introduce a provider-neutral Metadata Candidate Graph before deeper TMDB,
  Douban, Bangumi, Addon, or AI automation writes;
- deepen search semantics beyond a thin storage trait;
- keep Admin API read models and generated frontend/SDK contracts explicit and
  redacted;
- delete obsolete helpers, compatibility shims, generated noise, and replaced
  production paths.

Authoritative workstream:

- `docs/workstreams/future-ready-architecture-refactor/`

Completed:

- accepted ADR 0029 and ADR 0030 for the PostgreSQL-ready persistence boundary,
  SQL dialect, migration, row-codec, and fixture policy;
- moved `taru-db` toward a facade plus SQLite-owned adapter modules, with
  backend-neutral job lease contract tests and an optional PostgreSQL proof
  harness;
- extracted server runtime/service construction into `app::composition`;
- split source discovery from Local Inference planning;
- introduced provider-neutral Metadata Candidate Graph records and provider/NFO
  proof slices;
- deepened search projection semantics with Browse Facets, aliases, Sort Keys,
  provider identifiers, and projection versioning;
- preserved redacted Admin/Public API boundaries and reproducible Admin/Public
  TypeScript generation;
- deleted the `taru-api` root-level compatibility re-export shim and updated
  callers to explicit module boundaries;
- closed with `cargo check --workspace --tests` and
  `cargo nextest run --workspace --no-fail-fast`.

### PostgreSQL Production Readiness: M62

Status: completed.

This phase turned the M61 PostgreSQL proof into a production-shaped database
backend:

- expanded backend-neutral contract tests beyond jobs and leases;
- added PostgreSQL migration/schema parity for supported repository and workflow
  families;
- added explicit runtime backend selection through `TaruDatabase` and server
  configuration;
- removed or isolated SQLite-only assumptions above the adapter seam;
- documented repeatable SQLite always-on and PostgreSQL opt-in verification
  commands.

Authoritative workstream:

- `docs/workstreams/postgresql-production-readiness/`

Completed:

- `PostgresStore` is available in runtime code for the supported backend scope.
- `TaruDatabase` dispatches through an internal backend adapter trait and can
  select SQLite or PostgreSQL through explicit `DatabaseConnectOptions`.
- Backend-neutral contract families now cover lifecycle, jobs/leases,
  library/media, scan commits, metadata/catalog, playback runtime,
  event/webhook/addon/automation, runtime-promotion gap surfaces, and
  VFS/Staging.
- PostgreSQL opt-in full contract gates passed against a local test PostgreSQL
  URL.
- Managed Artwork PostgreSQL parity is intentionally split to
  `docs/workstreams/managed-artwork-postgresql-parity/`.

### Fearless Architecture Deepening: M63

Status: completed.

This phase deepened high-leverage Modules before new provider, plugin, AI,
playback, and remote-access breadth could harden shallow Interfaces:

- split Addon Side Effect behavior into deeper principal, intake, apply-router,
  metadata-write, library-file-write, artwork-write, and target Modules;
- added transactional Addon Canonical Metadata write commits for
  metadata/catalog/search/apply-outcome consistency;
- narrowed Library ingestion behind a workflow-shaped seam;
- stabilized playback/transcode request/cache identity around source revision
  and profile identity;
- separated hardware encoder discovery, device initialization, and smoke-probe
  diagnostics;
- added shared search semantics and projection-version discipline;
- improved test locality around focused SearchIndex semantics tests.

Authoritative workstream:

- `docs/workstreams/fearless-architecture-deepening/`

Completed:

- FAD-020 through FAD-090 are complete.
- Full workspace nextest passed with 498 tests run and 19 skipped.
- Existing independent tails remain in named lanes:
  - `docs/workstreams/managed-artwork-postgresql-parity/`
  - `docs/workstreams/admin-api-typescript-contract/`

## Workstream Split Direction

`server-foundation` was the initial planning hub. M5 split
`addons-automation` into its own completed workstream, M6 split `storage-vfs`,
M7 split `playback-streaming`, M13-M19 used metadata and runtime operations for
specialized hardening, M24 split `server-architecture-hardening` for the server
composition pass, M25 split `transcode-runtime` for playback runtime
productization, and M27 split `metadata-catalog` for media-library domain
expansion. As implementation grows, split the remaining broad domains
into narrower workstreams:

- `clients`: SDK package publishing, client streaming/download helpers,
  Dart/Flutter SDK, Rust CLI, or concrete Flutter/web client contracts.

Do future splits when a domain needs independent milestones or ADRs. Do not
split merely because a domain exists conceptually.
