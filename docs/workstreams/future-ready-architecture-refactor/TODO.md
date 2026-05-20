# Future-Ready Architecture Refactor — TODO

Status: Completed
Last updated: 2026-05-20

Task IDs use the `FRA` prefix.

## M0 — Scope, Priority, And Evidence Freeze

- [x] FRA-010 [owner=codex] [deps=none] [scope=docs/workstreams/future-ready-architecture-refactor,docs]
  Goal: Open the fearless future-ready architecture lane, record priorities,
  non-goals, deletion rules, PostgreSQL-readiness assumptions, and initial
  gates.
  Validation: `git diff --check`.
  Review: Confirm this lane does not duplicate completed
  `core-architecture-deepening` or `repository-seam-deepening` scope.
  Evidence: `docs/workstreams/future-ready-architecture-refactor/DESIGN.md`.
  Handoff: Continue with FRA-020 before implementation code changes.

## M1 — Persistence And PostgreSQL-Ready Architecture

- [x] FRA-020 [owner=codex] [deps=FRA-010] [scope=crates/taru-db,crates/taru-core,crates/taru-server,docs/adr,docs/workstreams/future-ready-architecture-refactor]
  Goal: Deep-dive the current persistence shape and choose the clean target
  architecture for SQLite plus future PostgreSQL.
  Validation: persistence inventory in DESIGN/JOURNAL; proposed ADR or ADR
  update if crate boundaries, migration semantics, or transaction policy
  change; `git diff --check`.
  Review: Must explicitly decide whether to split `taru-db` into
  backend-neutral contracts plus backend adapters, keep a facade, or perform a
  staged module split first.
  Evidence: `docs/workstreams/future-ready-architecture-refactor/JOURNAL/2026-05-20-fra-020.md`;
  `docs/adr/0029-postgresql-ready-persistence-boundary.md`.
  Handoff: FRA-030 should create backend-neutral persistence contract tests,
  starting with the job lease lifecycle contract.

- [x] FRA-030 [owner=codex] [deps=FRA-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,docs]
  Goal: Introduce backend-neutral persistence contract tests and a first
  transaction/unit-of-work seam for multi-record workflow commits.
  Validation: `cargo check -p taru-core --tests`; `cargo check -p taru-db --tests`;
  focused `cargo nextest run -p taru-db --no-fail-fast`; `git diff --check`.
  Review: Tests must prove behavior through the contract, not only through
  SQLite implementation details.
  Evidence: `crates/taru-db/src/contract_tests.rs`;
  `JobLeaseRepository`; `docs/workstreams/future-ready-architecture-refactor/EVIDENCE_AND_GATES.md`.
  Handoff: FRA-040 can now split the SQLite implementation with the job lease
  contract suite as a safety rail.

- [x] FRA-040 [owner=codex] [deps=FRA-030] [scope=crates/taru-db,crates/taru-server,crates/taru-library,crates/taru-metadata,crates/taru-nfo]
  Goal: Remove the `SqliteStore` god-adapter shape by splitting persistence
  implementation into cohesive domain adapters or backend-specific modules
  selected by FRA-020.
  Validation: focused checks for every touched crate; focused nextest for DB,
  library, metadata, NFO, and server app/http paths; `git diff --check`.
  Review: No broad mechanical split that only moves code around; every new
  module must improve locality, leverage, or PostgreSQL portability.
  Evidence: changed crate/module boundaries and passing contract tests.
  Progress: first slice moved SQLite store identity, job adapter, runtime
  options, and migrations behind `crates/taru-db/src/sqlite*`. Second slice
  introduced the public `TaruDatabase` facade, updated server and downstream
  test helpers to depend on it, made `SqliteStore` crate-private, and moved the
  backend-neutral job lease contract to run through the facade rather than the
  SQLite adapter. Third slice moved SQLite search index implementation under
  `sqlite/search.rs`, because search projection currently uses SQLite table
  semantics and should not sit at the facade root. Fourth slice moved
  SQLite-owned `library`, `library_item`, `metadata`, and `scan` repository
  implementations under `sqlite/` because those modules contain workflow
  commit transaction ordering, search projection writes, and library-scoped
  state SQL that future PostgreSQL must isolate. Fifth slice moved `media`,
  `provider_mapping`, and `catalog` under `sqlite/`, and rewired the
  scan/metadata/search transaction helpers to those SQLite-owned collaborators.
  Sixth slice moved `ingestion`, `local_inference`, `staging`, `playback`, and
  `user_playback` under `sqlite/`, and rewired scan workflow helpers to the
  SQLite-owned ingestion/local-inference collaborators. Seventh slice moved
  Addon, automation, event outbox, webhook, catalog governance, source
  duplicate, VFS cache, and artwork persistence under `sqlite/`, leaving only
  facade, codec, contract tests, and SQLite-specific tests at the `taru-db`
  root at that point. Eighth slice moved the shared SQLite row-codec/helper
  module to `sqlite/codec.rs`, leaving the `taru-db` root with only
  `facade.rs`, `sqlite.rs`, `contract_tests.rs`, `tests.rs`, and `lib.rs`.
  Ninth slice deleted the broad root/SQLite prelude shim, made SQLite modules
  import `SqliteStore`, SQLite codec helpers, `taru_core`, `taru_search`, and
  `sqlx` types explicitly, and moved the main `taru-db` behavior tests onto
  the `TaruDatabase` facade while keeping SQLite-only module tests inside the
  SQLite adapter. Root `taru-db` now exposes only `TaruDatabase` and
  `SqliteRuntimeOptions`.
  Handoff: Continue with FRA-050 migration, schema, row-codec, SQL dialect,
  and fixture policy now that the SQLite adapter split is complete.

- [x] FRA-050 [owner=codex] [deps=FRA-040] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-library,crates/taru-metadata,crates/taru-nfo,crates/taru-automation,crates/taru-catalog,crates/taru-events,docs/adr,docs/workstreams/future-ready-architecture-refactor]
  Goal: Make migration, schema, row codec, SQL dialect, and test-fixture policy
  explicit enough for a PostgreSQL adapter to be added without redesign.
  Validation: migration/dialect policy doc or ADR; SQLite migration tests;
  `cargo nextest run -p taru-db migrations --no-fail-fast` or updated focused
  equivalent; `git diff --check`.
  Review: Identify every SQLite-specific assumption that remains and either
  isolate it or record why it is intentionally backend-specific.
  Evidence:
  `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`;
  `docs/workstreams/future-ready-architecture-refactor/JOURNAL/2026-05-20-fra-050.md`;
  `DatabaseLifecycle`.
  Progress: Inventoried SQLite-specific dialect assumptions, accepted ADR 0030,
  renamed the misleading `TransactionManager` lifecycle trait to
  `DatabaseLifecycle`, and updated facade/server/downstream imports without
  keeping a compatibility alias.
  Handoff: FRA-060 should add a reusable backend contract-test harness shape
  with SQLite always on and optional PostgreSQL runtime coverage. If PostgreSQL
  code is added, it must have a real lifecycle/connection shape and at least
  one meaningful contract target, preferably job leases.

- [x] FRA-060 [owner=codex] [deps=FRA-050] [scope=crates/taru-db,docs/workstreams/future-ready-architecture-refactor]
  Goal: Add the smallest PostgreSQL readiness proof chosen by FRA-050: a
  backend contract-test harness that keeps SQLite always on and can run an
  optional PostgreSQL backend, plus a real PostgreSQL lifecycle/connection and
  job-lease proof if PostgreSQL code is introduced in this slice.
  Validation: chosen proof gate documented in EVIDENCE_AND_GATES.md; workspace
  check for touched crates; `git diff --check`.
  Review: Do not add a fake PostgreSQL layer that hides unresolved dialect
  problems.
  Evidence:
  `crates/taru-db/src/contract_tests.rs`;
  `crates/taru-db/src/postgres.rs`;
  `crates/taru-db/migrations/postgres/0001_contract_jobs.sql`;
  `docs/workstreams/future-ready-architecture-refactor/JOURNAL/2026-05-20-fra-060.md`.
  Progress: Added the reusable backend job lease contract harness, kept SQLite
  always on, added ignored `TARU_TEST_POSTGRES_URL` PostgreSQL job lease
  contract tests, and implemented a test-only PostgreSQL proof with real
  lifecycle, migration, UUID/jsonb/timestamptz storage, and
  `FOR UPDATE SKIP LOCKED` lease claiming.
  Handoff: Continue with runtime and domain seam deepening unless the user
  explicitly wants to keep deepening PostgreSQL production support first.

## M2 — Runtime And Domain Seam Deepening

- [x] FRA-070 [owner=codex] [deps=FRA-020] [scope=crates/taru-server/src/app,docs]
  Goal: Slim `TaruApp` by introducing cohesive runtime modules for storage,
  metadata, playback, automation/addons, admin operations, and jobs where they
  hide real construction complexity.
  Validation: `cargo check -p taru-server --tests`; focused server app/http
  nextest; `git diff --check`.
  Review: Avoid new pass-through structs; runtime modules must concentrate
  construction and policy knowledge.
  Evidence: app module split and focused tests.
  Progress: Moved the `TaruApp::new_with_store` service/runtime wiring into
  `crates/taru-server/src/app/composition.rs`. `TaruApp` now keeps the public
  app surface and service accessors, while `TaruAppComposition`,
  `TaruAppServices`, and `TaruRuntimeResources` own construction of runtime
  supervisors, storage backends, concurrency permits, metadata provider
  registry, service handles, startup workflow execution, and startup worker
  registration. The old inline `TaruAppInner` construction path was deleted.
  Handoff: Continue with FRA-080 Local Inference Engine extraction.

- [x] FRA-080 [owner=codex] [deps=FRA-030] [scope=crates/taru-library,crates/taru-naming,crates/taru-core,docs]
  Goal: Extract a deep Local Inference Engine that separates Media Source
  discovery from provisional hierarchy planning and Local Inference Evidence.
  Validation: `cargo check -p taru-library --tests`; focused `cargo nextest run
  -p taru-library local_inference --no-fail-fast`; `git diff --check`.
  Review: Scanning should discover; inference should explain; persistence
  should commit.
  Evidence: new inference module tests and scan/index caller simplification.
  Progress: Removed `ParsedName` from `DiscoveredMediaSource`, so
  `VfsLibraryScanner` now discovers VFS media source facts only. Added
  `LocalInferenceEngine` in `crates/taru-library/src/local_inference.rs` to
  parse discovered source paths, produce `LocalInferenceEvidence`, build
  `SourceState`/`MediaSource` facts, and plan provisional hierarchy ancestors.
  `LibraryIndexService` now asks the engine for a plan, reuses or creates
  planned provisional ancestors through repository lookups, then commits the
  resulting source/evidence/search projection through the existing scan commit
  unit.
  Handoff: Continue with FRA-090 provider-neutral Metadata Candidate Graph.

- [x] FRA-090 [owner=codex] [deps=FRA-030,FRA-080] [scope=crates/taru-metadata,crates/taru-core,crates/taru-nfo,docs]
  Goal: Introduce a provider-neutral Metadata Candidate Graph seam for TMDB,
  Douban, Bangumi, NFO, Addons, and future Automation Providers.
  Validation: `cargo check -p taru-metadata --tests`; focused metadata nextest;
  NFO/provider mapping tests as touched; `git diff --check`.
  Review: Provider-native payloads must not shape Canonical Metadata directly.
  Evidence: candidate graph records/tests and one provider/NFO proof slice.
  Progress: Added provider-neutral candidate graph domain records in
  `taru-core`, rewired `taru-metadata` provider search/fetch output to carry
  `MetadataCandidateGraph`, and made TMDB, Douban, Bangumi, and NFO proof
  slices project candidate records into Canonical Metadata only at explicit
  merge/import/render projection boundaries. Provider mapping acceptance now
  prefers root candidate subject facts instead of deriving all subject shape
  from the already-merged canonical item. NFO no longer stores a parallel
  `CanonicalMetadata` field in `NfoDocument`; it carries one local candidate
  graph and exposes `metadata()` as a projection helper.
  Handoff: Continue with FRA-100 search semantics. Broader future work can add
  persistence for multi-node candidate graphs, Addon/Automation candidate
  proposal acceptance, and richer provider hierarchy edges.

- [x] FRA-100 [owner=codex] [deps=FRA-030] [scope=crates/taru-search,crates/taru-catalog,crates/taru-db,docs]
  Goal: Deepen `taru-search` into a semantic search module with explicit
  projection versioning, normalized fields, Browse Facets, Sort Keys, aliases,
  and provider identifiers.
  Validation: `cargo check -p taru-search --tests`; `cargo check -p
  taru-catalog --tests`; focused search/catalog nextest; `git diff --check`.
  Review: Search semantics should not be a thin database trait wrapper.
  Evidence: search semantic tests and updated catalog/search projection seam.
  Progress: Added explicit search projection semantics in `taru-core` and
  `taru-search`: projection version, Browse Facets, keyed facet kinds,
  aliases, Sort Keys, provider identifiers, semantic facet-label parsing, and
  searchable text projection. Catalog hydration and library indexing now build
  semantic projections instead of raw string-facet bags. SQLite stores the new
  projection metadata in backend-owned columns, includes aliases and provider
  identifiers in searchable text, and matches required Browse Facets by exact
  normalized label from `facets_json` instead of substring matching
  `facets_text`.
  Handoff: Continue with FRA-110 Admin/API and generated contract hygiene.
  Later storage-engine choices can use the semantic projection contract without
  changing callers.

## M3 — API, Generated Contract, And Repository Hygiene

- [x] FRA-110 [owner=codex] [deps=FRA-070] [scope=crates/taru-api,crates/taru-server/src/http,docs/api,docs/workstreams/admin-api-typescript-contract]
  Goal: Audit Admin API read models and Public Client API mappings after
  persistence/runtime refactors so DTOs stay explicit, redacted, and contract
  owned.
  Validation: `cargo check -p taru-api --tests`; `cargo check -p taru-server
  --tests`; OpenAPI/SDK leakage tests; `git diff --check`.
  Review: Admin DTOs must not mirror persistence internals for convenience.
  Evidence: DTO/read-model audit notes and focused tests.
  Progress: Audited `taru-api` Admin/Public/OpenAPI/SDK guardrails and
  `taru-server/src/http` job-returning routes. The generated Admin API
  contract still covers the eight read-model routes and the full `taru-api`
  nextest suite proves Public Client OpenAPI/SDK artifacts still exclude
  admin/internal surfaces. Removed the unsafe `JobResponse` raw
  `input`/`summary`/`error` JSON echo and replaced it with explicit
  `has_input`/`has_summary`/`has_error` flags. Managed artwork ingest now
  exposes a narrow safe `failure_code` on the ingest summary instead of
  requiring clients to read raw job summary/error payloads. HTTP tests for
  library scan, NFO import/export, metadata refresh/maintenance, automation,
  and managed artwork now assert redaction through public HTTP responses.
  Handoff: Continue with FRA-120 repository/generated frontend hygiene.

- [x] FRA-120 [owner=codex] [deps=FRA-110] [scope=.gitignore,apps/admin-web,sdk/typescript,docs]
  Goal: Clean generated/frontend repository hygiene: ignore build outputs,
  logs, Playwright artifacts, tsbuildinfo, and dependency folders; keep
  generated contracts reproducible by commands.
  Validation: `npm run verify` in touched frontend/SDK packages when practical;
  `git status --short`; `git diff --check`.
  Review: Do not delete user edits. Only remove tracked generated noise if the
  diff proves it is ours or the user approves.
  Evidence: ignore rules and generated-contract commands.
  Progress: Expanded root ignore rules for frontend build outputs,
  dependency folders, coverage/test reports, Playwright artifacts, Vite local
  caches/logs, TypeScript build-info files, and package-manager debug logs.
  Tightened `apps/admin-web/.gitignore` for app-local generated runtime
  artifacts and added `sdk/typescript/.gitignore` for SDK package-local
  outputs. Added `npm run generate:admin-api --prefix apps/admin-web` and
  wired `apps/admin-web` `verify` to regenerate the Admin API contract before
  check/test/build. Confirmed generated Admin API contract and Public Client
  SDK regeneration produced no textual drift. Did not delete ignored local
  artifacts or user files.
  Handoff: Continue with FRA-130 deletion sweep.

## M4 — Deletion Sweep And Closeout

- [x] FRA-130 [owner=codex] [deps=FRA-040,FRA-070,FRA-080,FRA-090,FRA-100,FRA-110,FRA-120] [scope=workspace,docs]
  Goal: Delete obsolete helpers, redundant adapters, compatibility shims,
  stale tests, and old production paths left behind by the refactor.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  focused nextest for touched crates; frontend/SDK checks if touched;
  `git diff --check`.
  Review: No old/new production parallel path remains without a named expiry
  task.
  Evidence:
  `docs/workstreams/future-ready-architecture-refactor/JOURNAL/2026-05-20-fra-130.md`;
  `docs/workstreams/future-ready-architecture-refactor/EVIDENCE_AND_GATES.md`.
  Progress: Deleted the `taru-api` root-level compatibility re-export shim,
  forcing callers to import DTOs and helpers through explicit API boundary
  modules (`admin`, `extension`, `metadata_diagnostics`, `public_client`,
  `openapi`, `sdk`, or `admin_contract`). Updated `taru-server` app, HTTP
  handlers, and tests to use those explicit module paths. Updated stale HTTP
  API job-envelope documentation so it matches the FRA-110 redacted
  `has_input`/`has_summary`/`has_error` contract. Audited remaining candidates
  and kept only deliberate, named paths: the intra-admin managed-artwork
  aggregation and the explicit search facet-label adapter needed by current
  query parsing/tests.
  Handoff: Continue with FRA-140 closeout or split remaining broad areas.

- [x] FRA-140 [owner=codex] [deps=FRA-130] [scope=docs/workstreams/future-ready-architecture-refactor,docs/GOALS.md,docs/ROADMAP.md,docs/workstreams/README.md]
  Goal: Close or split the lane with fresh verification evidence and updated
  roadmap/goal/workstream status.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast` unless narrowed with a
  documented reason; `git diff --check`.
  Review: Use `verify-rust-workstream` and `review-workstream` before closure.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Progress: Closed M61 as completed. Updated workstream, goal, roadmap, and
  workstream index docs; recorded residual broad product work as future
  follow-on scope rather than closeout debt; verified the full workspace with
  `cargo check --workspace --tests` and
  `cargo nextest run --workspace --no-fail-fast`.
  Handoff: Big refactor goal can now be marked complete.
