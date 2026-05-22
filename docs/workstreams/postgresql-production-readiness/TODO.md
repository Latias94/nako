# PostgreSQL Production Readiness — TODO

Status: Completed
Last updated: 2026-05-20

Task IDs use the `PGR` prefix.

## M0 — Scope, Matrix, And Evidence Baseline

- [x] PGR-010 [owner=codex] [deps=none] [scope=docs/workstreams/postgresql-production-readiness,docs/GOALS.md,docs/ROADMAP.md,docs/workstreams/README.md]
  Goal: Open the M62 PostgreSQL production-readiness lane, record the current
  M61 proof state, define the contract-test/schema/runtime priorities, and
  publish initial gates.
  Validation: `git diff --check`.
  Review: Must not claim PostgreSQL production readiness yet. This task only
  creates the executable lane and matrix.
  Evidence: `docs/workstreams/postgresql-production-readiness/DESIGN.md`;
  `docs/workstreams/postgresql-production-readiness/EVIDENCE_AND_GATES.md`.
  Progress: Created the workstream and first contract-test matrix. Updated
  top-level goal, roadmap, and workstream index docs so M62 is the active
  implementation goal.
  Handoff: Continue with PGR-020 lifecycle/backend selection design.

## M1 — Lifecycle, Backend Selection, And Verification Harness

- [x] PGR-020 [owner=codex] [deps=PGR-010] [scope=crates/nako-db,crates/nako-server/src/config.rs,crates/nako-server/src/app,docs/adr,docs/workstreams/postgresql-production-readiness]
  Goal: Design and implement the first production-shaped database backend
  selection seam for SQLite and PostgreSQL.
  Validation: `cargo check -p nako-db --tests`; `cargo check -p nako-server
  --tests`; focused lifecycle/backend-selection tests; `git diff --check`.
  Review: Avoid URL-guessing-only backend selection. Server code must not
  depend on concrete SQLite/PostgreSQL adapters.
  Evidence: backend kind/config constructors, lifecycle tests, and updated
  evidence notes.
  Progress: Added explicit `DatabaseBackendKind` and `DatabaseConnectOptions`
  in `nako-db`; `NakoDatabase` now reports backend kind/capabilities and uses
  options-based construction. Server config now has an explicit
  `database_backend` field and `NakoApp::new` routes through the facade rather
  than concrete adapters. PostgreSQL runtime selection is intentionally gated
  with a named unsupported error until contract parity and migrations catch up.
  Handoff: Continue with PGR-030 contract harness generalization.

- [x] PGR-030 [owner=codex] [deps=PGR-020] [scope=crates/nako-db/src/contract_tests.rs,docs/workstreams/postgresql-production-readiness]
  Goal: Generalize the backend contract-test harness beyond job leases so
  contract families can be added without copy-paste backend runners.
  Validation: `cargo check -p nako-db --tests`; `cargo nextest run -p nako-db
  contract --no-fail-fast`; optional PostgreSQL ignored tests documented;
  `git diff --check`.
  Review: SQLite must remain always-on. PostgreSQL tests must stay opt-in and
  safe for local schemas.
  Evidence: reusable backend contract runner and matrix updates.
  Progress: Replaced per-backend job-lease runner functions with a reusable
  `ContractCase`/`ContractFamily` runner and `database_contract_pair!` macro.
  Added a lifecycle contract family with idempotent migrate coverage. SQLite
  remains always-on, and PostgreSQL remains ignored/opt-in with isolated schema
  setup and the existing `NAKO_TEST_POSTGRES_URL` gate.
  Handoff: Continue with PGR-040 Library/Media Source contract slice.

## M2 — Core Repository And Workflow Contracts

- [x] PGR-040 [owner=codex] [deps=PGR-030] [scope=crates/nako-db,crates/nako-core,docs/workstreams/postgresql-production-readiness]
  Goal: Add PostgreSQL parity for Media Library, Media Item, and Media Source
  identity contracts through the backend contract harness.
  Validation: `cargo check -p nako-db --tests`; focused SQLite/PostgreSQL
  contract nextest; `git diff --check`.
  Review: Media Source identity must remain scoped by Media Library; do not
  flatten Source Locator semantics for PostgreSQL convenience.
  Evidence: contract tests and PostgreSQL migrations for required tables.
  Progress: Added a `LibraryMedia` contract family that proves Library
  round-trip/list ordering, Media Item hierarchy/external IDs, Library-scoped
  Source Locator identity, source listing, Library Item State, and
  kind/parent/title lookup through the shared harness. Expanded the
  PostgreSQL proof migration with native `uuid`, `jsonb`, `boolean`, and
  backend-owned indexes for media/library-item tables. Implemented the required
  PostgreSQL `MediaRepository` and `LibraryItemRepository` slice.
  Handoff: Continue with scan commit contracts.

- [x] PGR-050 [owner=codex] [deps=PGR-040] [scope=crates/nako-db,crates/nako-library,docs/workstreams/postgresql-production-readiness]
  Goal: Add backend-neutral contracts and PostgreSQL parity for the Library
  scan commit unit: Source State, Local Inference Evidence, Media Technical
  Facts, ingestion failures, and Search Projection side effects.
  Validation: `cargo check -p nako-db --tests`; `cargo check -p nako-library
  --tests`; focused scan/index nextest; optional PostgreSQL contract run;
  `git diff --check`.
  Review: Commit behavior must be atomic across graph/source/evidence/search
  writes, not a sequence of independently visible writes.
  Evidence: scan contract tests and PostgreSQL migration parity.
  Progress: Added the `ScanCommit` contract family with success and rollback
  contracts. The success contract proves scan snapshots, directory snapshots,
  Media Item/Media Source writes, Source State, Library Item State, Local
  Inference Evidence, Search Projection hits, ingestion failure resolution, and
  Media Technical Facts through `MediaProbeRepository`. The rollback contract
  proves the Library scan source commit is transactional when a Search
  Projection write fails. Expanded the PostgreSQL proof migration with native
  tables for probes/streams, scan snapshots, directory snapshots, source
  states, search documents, local inference evidence, and ingestion failures.
  Implemented the PostgreSQL `ScanRepository`, `LocalInferenceRepository`,
  `IngestionFailureRepository`, `MediaProbeRepository`, and `SearchIndex`
  slices needed by the contracts.
  Handoff: Continue with Metadata/Catalog commit contracts.

- [x] PGR-060 [owner=codex] [deps=PGR-050] [scope=crates/nako-db,crates/nako-metadata,crates/nako-catalog,docs/workstreams/postgresql-production-readiness]
  Goal: Add backend-neutral contracts and PostgreSQL parity for metadata
  refresh/NFO import Catalog Item Graph and Search Projection commits.
  Validation: `cargo check -p nako-db --tests`; `cargo check -p
  nako-metadata --tests`; `cargo check -p nako-catalog --tests`; focused
  metadata/catalog contract nextest; `git diff --check`.
  Review: Provider-native payloads and Candidate Graph records must not leak
  into PostgreSQL-specific schema choices.
  Evidence: metadata/catalog contract tests and migration parity.
  Progress: Added the `MetadataCatalog` contract family for metadata refresh
  and NFO import commits. The metadata refresh contract now proves Provider
  Subject, Provider Mapping, provider raw response, metadata provider attempt,
  Media Item update, and Library Item State confirmation behavior. The NFO
  import contract now proves atomic Media Item, field lock, full Catalog Item
  Graph replacement, Search Projection, and rollback behavior across People,
  Credits, Genres, Tags, Franchise Collections, Studios, and Image Assets.
  Expanded the PostgreSQL proof migration with native tables for metadata
  locks/provider payloads/provider mappings/provider attempts and Catalog Item
  Graph records. Implemented PostgreSQL `MetadataRepository`,
  `ProviderMappingRepository`, and full `CatalogRepository` parity needed by
  the contract slice.
  Handoff: Continue with user playback/transcode/runtime contracts.

## M3 — Runtime State And Operational Contracts

- [x] PGR-070 [owner=codex] [deps=PGR-040] [scope=crates/nako-db,crates/nako-server/src/app/playback,docs/workstreams/postgresql-production-readiness]
  Goal: Add backend-neutral contracts and PostgreSQL parity for User Playback
  State and Transcode Session lifecycle.
  Validation: `cargo check -p nako-db --tests`; `cargo check -p nako-server
  --tests`; focused playback/user-playback nextest; `git diff --check`.
  Review: User Playback State remains principal-scoped; transcode output paths
  remain redacted and server-owned.
  Evidence: contracts and PostgreSQL migration parity.
  Progress: Added the `PlaybackRuntime` contract family with backend-neutral
  contracts for principal-scoped User Playback State, Continue Watching
  filtering, Transcode Session active/latest lookup, cancellation request,
  terminal state transition, filtered listing, and stale active session
  recovery. Expanded the PostgreSQL proof migration with
  `user_playback_states` and `transcode_sessions`, including active-request
  uniqueness. Implemented PostgreSQL `UserPlaybackStateRepository` and
  `TranscodeSessionRepository` parity.
  Handoff: Continue with event/webhook/addon contracts.

- [x] PGR-080 [owner=codex] [deps=PGR-040] [scope=crates/nako-db,crates/nako-events,crates/nako-automation,crates/nako-server/src/app/addons.rs,docs/workstreams/postgresql-production-readiness]
  Goal: Add backend-neutral contracts and PostgreSQL parity for event outbox,
  webhooks, Addons, and Automation Providers where they are enabled under
  PostgreSQL.
  Validation: `cargo check -p nako-db --tests`; `cargo check -p
  nako-events --tests`; `cargo check -p nako-automation --tests`; focused
  addon/event/automation nextest; `git diff --check`.
  Review: Addon Tokens, grants, side effects, and webhook payloads must stay
  redacted and idempotent.
  Evidence: contracts and PostgreSQL migration parity.
  Progress: Added the `EventAddonAutomation` contract family with
  backend-neutral contracts for Event Outbox idempotency/filtering, Webhook
  endpoint/delivery attempt state, Addon registration/token/grant/side-effect
  idempotency and apply outcome, and Automation Provider/artifact state.
  Expanded the PostgreSQL proof migration with event outbox, webhook,
  automation, Addon registration/token/grant, and Addon Side Effect tables.
  Implemented PostgreSQL repository parity for `EventOutboxRepository`,
  `WebhookRepository`, `AddonRepository`, and `AutomationRepository`.
  Handoff: Continue with Managed Artwork or explicitly split it.

- [x] PGR-090 [owner=codex] [deps=PGR-050] [scope=crates/nako-db,crates/nako-server/src/app/artwork.rs,docs/workstreams/postgresql-production-readiness]
  Goal: Decide whether Managed Artwork parity belongs in M62 and either
  implement backend-neutral contracts plus PostgreSQL migrations or split a
  named follow-on with expiry gates.
  Validation: decision note; if implemented, focused DB/server artwork tests;
  `git diff --check`.
  Review: Do not partially enable PostgreSQL while Managed Artwork runtime
  assumes SQLite-only state for enabled features.
  Evidence: implementation or split follow-on.
  Progress: Split Managed Artwork PostgreSQL parity out of M62 into the named
  follow-on `docs/workstreams/managed-artwork-postgresql-parity/`. Rationale:
  Managed Artwork spans Addon Artwork Candidates, ingest jobs, artifact records,
  Selected Artwork, gallery/lifecycle cleanup, drift diagnostics, remediation,
  thumbnail variants, artifact-store files, and public/Admin redaction. Partial
  PostgreSQL enablement would be less truthful than an explicit follow-on gate.
  Handoff: Continue with runtime diagnostics and cleanup.

## M4 — Runtime Diagnostics, Assumption Cleanup, And Closeout

- [x] PGR-100 [owner=codex] [deps=PGR-020,PGR-040] [scope=crates/nako-server/src/config.rs,crates/nako-server/src/http/admin.rs,docs/api,docs/workstreams/postgresql-production-readiness]
  Goal: Add safe database backend diagnostics and update HTTP/config docs for
  SQLite/PostgreSQL backend selection.
  Validation: `cargo check -p nako-api --tests`; `cargo check -p nako-server
  --tests`; focused admin/system route tests; `git diff --check`.
  Review: Diagnostics must reveal backend kind and migration state without
  leaking database credentials, local paths, or raw database errors.
  Evidence: admin DTO/route tests and docs.
  Progress: Added a sanitized database block to the Admin system config
  diagnostics DTO and generated Admin TypeScript contract. The route now
  reports configured backend kind, active backend kind, URL scheme,
  startup-migration status, runtime-support status, and active backend
  capability booleans without returning `database_url`, credentials, paths,
  hosts, query strings, or raw database errors. Server startup now records that
  migrations completed, and the system config route test asserts both the safe
  fields and the redaction boundaries.
  Handoff: Continue with SQLite assumption deletion sweep.

- [x] PGR-110 [owner=codex] [deps=PGR-020,PGR-030,PGR-100] [scope=workspace,docs/workstreams/postgresql-production-readiness]
  Goal: Delete or isolate remaining SQLite-only assumptions above the adapter
  seam.
  Validation: `rg` inventory; `cargo check --workspace --tests`; focused
  nextest for touched crates; `git diff --check`.
  Review: No server/facade path should require SQLite-specific row codecs,
  URL forms, timestamps, or SQL behavior unless explicitly named as a follow-on.
  Evidence: deletion inventory in EVIDENCE_AND_GATES.md.
  Progress: Removed the compatibility-style `NakoDatabase::connect(&str)` and
  `connect_with_sqlite_runtime(...)` helpers so production callers must use
  explicit `DatabaseConnectOptions`. Removed the remaining facade-level test
  dependency on `sqlite::codec` and direct `store.sqlite().pool()` inspection;
  the startup recovery test now verifies behavior through repository APIs
  instead of SQLite row access. Inventory confirms SQLite SQL dialect terms and
  row codecs are isolated to `nako-db::sqlite` or SQLite-owned tests, while
  `sqlite::memory:` remains only as test fixture/default SQLite config data.
  Handoff: Continue with closeout.

- [x] PGR-120 [owner=codex] [deps=PGR-110] [scope=docs/workstreams/postgresql-production-readiness,docs/GOALS.md,docs/ROADMAP.md,docs/workstreams/README.md]
  Goal: Close or split M62 with fresh verification evidence and updated
  roadmap/goal/workstream status.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; PostgreSQL opt-in contract
  gate when `NAKO_TEST_POSTGRES_URL` is available or documented skip evidence;
  `git diff --check`.
  Review: Use verify/review workstream before marking the thread goal complete.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Progress: PGR-120 closeout audit found PostgreSQL runtime startup was still
  gated by missing VFS Cache/Staging Manifest parity. Expanded scope inside
  PGR-120 rather than closing early: added a backend-neutral `VfsStaging`
  contract family, PostgreSQL VFS/staging schema parity, repository parity,
  and flipped PostgreSQL `vfs_cache` capability to supported while keeping
  Managed Artwork split/gated. Final gates passed: formatting, nako-db/server
  checks, workspace check, SQLite/default contract run, PostgreSQL opt-in full
  contract run against local test PostgreSQL, workspace nextest, and
  `git diff --check` with CRLF normalization warnings only.
  Handoff: M62 is closed. Continue PostgreSQL-specific work through the named
  Managed Artwork PostgreSQL parity follow-on or a new production hardening
  lane for performance/CI/containerized PostgreSQL setup.
