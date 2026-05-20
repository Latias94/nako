# Future-Ready Architecture Refactor — Handoff

Status: Completed
Last updated: 2026-05-20

## Current State

The workstream has been opened, FRA-020 is complete, FRA-030 has added the
first backend-neutral persistence contract suite, FRA-040 is complete,
FRA-050 is complete, FRA-060 is complete, FRA-070 is complete, FRA-080 is
complete, FRA-090 is complete, FRA-100 is complete, FRA-110 is complete,
FRA-120 is complete, FRA-130 is complete, and FRA-140 is complete. The SQLite
implementation now lives behind
SQLite-owned
store/runtime/migration/job/search/library/metadata/scan/media/catalog/
ingestion/staging/playback/addon/automation/artwork/event/webhook modules plus
the public `TaruDatabase` facade. The persistence target architecture is
recorded in ADR 0029 and ADR 0030:

- `taru-db` should become a database facade;
- SQLite details should move behind a SQLite-owned adapter module or crate;
- future PostgreSQL should implement the same backend-neutral contracts;
- contract tests should be added before large module movement;
- SQL dialects and migrations are backend-owned rather than shared across
  SQLite and PostgreSQL.

FRA-030 implementation notes:

- `JobRepository` now owns ordinary job enqueue/start/succeed/fail/get/list
  behavior.
- `JobLeaseRepository` now owns durable worker lease, heartbeat, run-token
  fencing, cancellation, and expired-lease recovery behavior.
- `crates/taru-db/src/contract_tests.rs` contains the first contract harness
  and runs the job lease lifecycle contract against SQLite.
- Duplicate SQLite-only job lease lifecycle tests were removed from
  `crates/taru-db/src/tests.rs`; SQLite-specific job listing and startup
  recovery coverage remains there.

FRA-040 implementation notes:

- `SqliteStore` identity now lives in `crates/taru-db/src/sqlite.rs`.
- `SqliteStore` is crate-private; downstream crates should no longer construct
  or store it directly.
- The root `taru-db` module exposes only `TaruDatabase` as the public database
  facade plus `SqliteRuntimeOptions` for SQLite runtime configuration.
- Durable job repository SQL moved to `crates/taru-db/src/sqlite/jobs.rs`.
- SQLite-backed search index SQL moved to
  `crates/taru-db/src/sqlite/search.rs`.
- SQLite library, library-item, metadata, and scan repository implementations
  moved to `crates/taru-db/src/sqlite/{library,library_item,metadata,scan}.rs`.
- SQLite media, provider-mapping, and catalog repository implementations moved
  to `crates/taru-db/src/sqlite/{media,provider_mapping,catalog}.rs`.
- SQLite ingestion, local-inference, staging, playback, and user-playback
  repository implementations moved to
  `crates/taru-db/src/sqlite/{ingestion,local_inference,staging,playback,user_playback}.rs`.
- SQLite Addon, automation, event outbox, webhook, catalog-governance, source
  duplicate, VFS cache, and artwork persistence implementations moved under
  `crates/taru-db/src/sqlite/`.
- SQLite connection/runtime policy moved to
  `crates/taru-db/src/sqlite/runtime.rs`.
- SQLite migrations and lifecycle implementation moved to
  `crates/taru-db/src/sqlite/migrations.rs`.
- `crates/taru-db/src/facade.rs` delegates repository/search/lifecycle traits
  from `TaruDatabase` to the active SQLite adapter.
- `taru-server` and the touched downstream crate tests now use `TaruDatabase`;
  a focused search found no remaining `SqliteStore` references outside
  `taru-db` code or historical docs.
- `crates/taru-db/src/contract_tests.rs` now constructs `TaruDatabase`, so the
  job lease lifecycle contract runs against the facade boundary.
- `crates/taru-db/src/tests.rs` now constructs `TaruDatabase` for main
  persistence behavior tests. SQLite-only module tests remain inside
  `sqlite/*` modules where they intentionally inspect migration/runtime or
  rollback details.
- The broad `sqlite::prelude` compatibility shim was deleted. SQLite modules
  now import `SqliteStore`, SQLite codec helpers, `taru_core`, `taru_search`,
  and `sqlx` types explicitly.
- Root `crates/taru-db/src` now contains only `contract_tests.rs`,
  `facade.rs`, `lib.rs`, `sqlite.rs`, and `tests.rs`.

FRA-050 implementation notes:

- Accepted ADR 0030, defining backend-owned migrations, SQL dialect policy,
  logical schema policy, row-codec ownership, test fixture rules, and the
  FRA-060 proof target.
- Inventoried SQLite-specific assumptions now isolated under `sqlite/`:
  SQLite `strftime`, `ON CONFLICT`, `?1` parameters, `LIMIT/OFFSET`, `*_json`
  text payloads, integer boolean codecs, ID string binds/parsers,
  `SqliteRow`, SQLx SQLite transactions, runtime PRAGMAs, and millisecond
  timestamp columns.
- Renamed the misleading `TransactionManager` trait to `DatabaseLifecycle`
  and moved its file to `crates/taru-core/src/repository/lifecycle.rs`.
  No compatibility alias was kept.

FRA-060 implementation notes:

- Reshaped `crates/taru-db/src/contract_tests.rs` into a reusable backend job
  lease contract harness.
- Kept SQLite as the always-on backend.
- Added ignored PostgreSQL job lease contract tests gated by
  `TARU_TEST_POSTGRES_URL`.
- Added a test-only `PostgresStore` with real lifecycle, migration, job, and
  job-lease behavior.
- Added backend-owned PostgreSQL proof migration under
  `crates/taru-db/migrations/postgres/`.
- The PostgreSQL proof uses native UUID/jsonb/timestamptz storage and
  `FOR UPDATE SKIP LOCKED` lease claiming.

FRA-070 implementation notes:

- Added `crates/taru-server/src/app/composition.rs`.
- Replaced the inline `TaruAppInner` construction path with
  `TaruAppComposition`.
- `TaruRuntimeResources` now owns construction of the process-local runtime
  supervisor, storage backend registry, scan/metadata/webhook concurrency
  permits, and metadata provider registry.
- `TaruAppServices` now owns construction of the app service handles for jobs,
  library scan, artwork, addons, automation, webhooks, catalog, library,
  storage diagnostics, metadata, NFO, playback, and user playback.
- Startup workflow execution and optional managed artwork ingest worker startup
  now live in the composition module after service construction.
- `TaruApp` now keeps the app handle surface and delegates runtime/service
  construction to the composition module.

FRA-080 implementation notes:

- `DiscoveredMediaSource` no longer contains `ParsedName`.
- `VfsLibraryScanner` no longer depends on `taru-naming`; it discovers VFS
  media source facts only.
- `LocalInferenceEngine` now owns path parsing, evidence generation,
  `SourceState`/`MediaSource` fact shaping, primary provisional item planning,
  and provisional hierarchy ancestor planning.
- `LibraryIndexService` now asks the engine for a `LocalInferencePlan`, then
  resolves or creates planned ancestors through repository lookups and commits
  source/evidence/search projection through `commit_library_scan_source`.
- Confirmed canonical metadata preservation remains in the index layer, so a
  non-provisional existing item is not overwritten by provisional local
  inference.

FRA-090 implementation notes:

- Added provider-neutral Metadata Candidate Graph records in
  `crates/taru-core/src/media/candidate.rs`:
  `MetadataCandidateGraph`, `MetadataCandidateNode`,
  `MetadataCandidateRecord`, `MetadataCandidateSubject`,
  `MetadataCandidateRelationship`, and `MetadataCandidateSource`.
- `MetadataCandidateRecord` stores provider/NFO/addon/automation-neutral field
  candidates with optional scalar fields and list fields, then projects into
  `CanonicalMetadata` only when the merge/import/export boundary needs the
  authoritative item state.
- `taru-metadata` now returns `MetadataCandidateGraph` from
  `MetadataCandidate` and `MetadataFetchResult`; provider callers use
  `metadata()` only as an explicit projection.
- TMDB, Douban, and Bangumi mapping functions now produce
  `MetadataCandidateRecord` instead of direct `CanonicalMetadata`.
- Provider search/fetch implementations wrap candidate records in a root
  provider subject with provider, subject kind, subject key, title, release
  year, and optional locale.
- Metadata refresh merges the fetched candidate graph's canonical projection,
  while provider mapping acceptance prefers the root candidate subject facts
  instead of deriving the whole subject shape from the merged canonical item.
- `NfoDocument` now carries only a local `candidate_graph` plus sidecar
  hierarchy/external-id facts. NFO import merges the graph projection rather
  than directly treating parsed XML metadata as the canonical input. NFO export
  builds a local candidate graph from the item being rendered.


FRA-100 implementation notes:

- `taru-core` now defines semantic search projection records: projection
  version, Browse Facets, keyed facet kinds, Sort Keys, aliases, provider
  identifiers, and `CatalogSearchProjection::searchable_text()`.
- `taru-search` now exposes `SearchDocument` and `SearchQuery` in terms of
  semantic Browse Facets rather than raw string facet bags. Legacy facet-label
  conversion is explicit through fallible `from_facet_labels` helpers.
- Catalog hydration and library indexing build semantic projections from item
  metadata, sources, labels, people, release years, aliases, sort titles, and
  provider IDs.
- SQLite persists the semantic projection metadata in backend-owned columns and
  matches Browse Facets by exact normalized label from `facets_json`, not by
  substring search over `facets_text`.
- Added tests for structured facet labels, exact facet matching, searchable
  aliases, and catalog hydration provider-id/release-year facet projection.

FRA-110 implementation notes:

- Audited the Admin API generated contract, Public Client OpenAPI, Public
  Client TypeScript SDK, `taru-api` DTOs, and job-returning HTTP routes after
  the persistence/runtime/search refactors.
- Confirmed the existing generated Admin API contract remains scoped to the
  eight admin-web read-model routes, and the full `taru-api` nextest suite
  keeps Admin/Public/OpenAPI/SDK separation guarded.
- Removed raw persisted job `input_json`, `summary_json`, and `error` echo
  from `JobResponse`.
- `JobResponse` now exposes explicit `has_input`, `has_summary`, and
  `has_error` flags instead of raw payloads.
- Managed artwork ingest failure diagnostics now expose a narrow safe
  `ManagedArtworkIngestSummary::failure_code` field instead of requiring
  callers to inspect raw job summary/error JSON.
- Updated HTTP tests for library scan, NFO import/export, metadata
  refresh/maintenance, automation jobs, and managed artwork acceptance/failure
  paths to assert redacted job responses.

FRA-120 implementation notes:

- Expanded root `.gitignore` for frontend dependency/build output, coverage,
  Playwright/test reports, TypeScript/Vite artifacts, and package-manager debug
  logs.
- Expanded `apps/admin-web/.gitignore` with the app-local artifact set.
- Added `sdk/typescript/.gitignore` for SDK package-local build, coverage,
  TypeScript build-info, and package-manager debug-log artifacts.
- Added `npm run generate:admin-api --prefix apps/admin-web`, wired
  admin-web `verify` to regenerate the Admin API contract before check/test/
  build, and documented the command in `apps/admin-web/README.md`.
- Verified admin-web and Public Client SDK generation commands produce no
  textual drift in tracked generated files.
- Did not delete ignored local artifacts or user files.

FRA-130 implementation notes:

- Deleted the `taru-api` root-level compatibility re-export shim
  (`admin::*`, `extension::*`, `metadata_diagnostics::*`, and
  `public_client::*`).
- `taru-api` now exposes explicit boundary modules only:
  `admin`, `admin_contract`, `extension`, `metadata_diagnostics`, `openapi`,
  `public_client`, and `sdk`.
- Updated `taru-api` internal callers plus `taru-server` app services, HTTP
  handlers, and tests to import DTOs/helpers through explicit modules.
- Updated `docs/api/HTTP_API.md` so job envelopes document redacted
  `has_input`, `has_summary`, and `has_error` flags instead of raw
  `input`/`summary`/`error` payloads.
- Audited but intentionally kept `admin::managed_artwork::*` as an Admin API
  module aggregation point and the explicit `taru-search` facet-label adapter
  helpers as current query/test adapters into semantic Browse Facets.

FRA-140 closeout notes:

- M61 is closed as completed.
- `cargo check --workspace --tests` passed.
- `cargo nextest run --workspace --no-fail-fast` passed: 466 tests passed,
  4 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
- Remaining broad work is future scope rather than closeout debt: production
  PostgreSQL operations, optional backend crate extraction, richer search
  backends, network tunneling, AI automation runtime, and further Admin UI
  surfaces.

The priority order remains:

1. persistence/PostgreSQL readiness;
2. server runtime composition;
3. Local Inference Engine;
4. Metadata Candidate Graph;
5. search semantics;
6. Admin/API/generated contract hygiene;
7. generated/frontend repository hygiene;
8. deletion and closeout.

## Active Task

- None. M61/FRA is closed.

## Decisions Since Last Update

- Use a new workstream instead of reopening completed
  `architecture-review-followups`, `core-architecture-deepening`, or
  `repository-seam-deepening`.
- Treat PostgreSQL readiness as the highest-priority architecture risk.
- Prefer deleting old paths over compatibility shims because Taru is not live
  and has no production compatibility burden.
- Accepted ADR 0029 and chose the `taru-db` facade plus backend-specific
  adapters target.
- Start contract testing with job lease lifecycle because PostgreSQL will need
  different SQL locking behavior while preserving the same observable contract.
- Completed FRA-030 by extracting `JobLeaseRepository` and adding the SQLite
  job lease contract suite.
- Started FRA-040 with a SQLite-owned module split for store identity, jobs,
  runtime, and migrations.
- Added the public `TaruDatabase` facade and updated production app services
  and downstream tests to depend on the facade rather than concrete
  `SqliteStore`.
- Made `SqliteStore` crate-private so future PostgreSQL work can change the
  active backend without a public API break.
- Moved the current SQLite table-backed `SearchIndex` implementation under
  `sqlite/search.rs`; search semantics are still a later FRA-100 concern.
- Moved library, library-item, metadata, and scan persistence under `sqlite/`
  because they contain SQLite transaction ordering, search projection writes,
  and library-scoped source state SQL that should not remain at the facade root.
- Moved media, provider mapping, and catalog persistence under `sqlite/` and
  rewired scan/metadata/search transaction helpers to call those SQLite-owned
  collaborators instead of root modules.
- Moved ingestion, local inference, staging, playback, and user playback
  persistence under `sqlite/`; scan workflow commits now call the SQLite-owned
  ingestion/local-inference helpers.
- Moved the remaining repository implementation modules under `sqlite/`; the
  `taru-db` root now holds facade, contract tests, SQLite module declaration,
  crate root, and SQLite-specific tests.
- Moved the SQLite row-codec/helper module under `sqlite/codec.rs`.
- Deleted the broad `sqlite::prelude` shim and replaced it with explicit
  SQLite-module imports.
- Moved main `taru-db` behavior tests onto the `TaruDatabase` facade.
- Marked FRA-040 complete with fresh `taru-db` nextest, `taru-server` nextest,
  workspace test compilation, formatting, and diff-whitespace evidence.
- Accepted ADR 0030 for PostgreSQL-ready SQL dialect and migration policy.
- Completed FRA-050 by making migration/schema/row-codec/dialect/test-fixture
  policy explicit, recording the SQLite-specific assumption inventory, and
  renaming `TransactionManager` to `DatabaseLifecycle`.
- Completed FRA-060 by adding the reusable backend job lease contract harness
  and optional PostgreSQL job lease proof with real PostgreSQL migration,
  lifecycle, row-codec, and locking behavior.
- Completed FRA-070 by extracting server runtime/service construction into the
  `app::composition` module, deleting the old inline `TaruAppInner`
  construction path, and preserving startup/runtime behavior with full
  `taru-server` nextest evidence.
- Completed FRA-080 by moving naming interpretation out of scan discovery into
  `LocalInferenceEngine`, keeping scanning as source discovery, inference as
  explanation/planning, and persistence as the commit owner.
- Completed FRA-090 by introducing a provider-neutral Metadata Candidate Graph
  seam and routing TMDB, Douban, Bangumi, and NFO through it before Canonical
  Metadata merge/import projection.
- Completed FRA-100 by replacing raw search facet bags with semantic projection
  records, exact Browse Facet matching, searchable aliases, Sort Keys, provider
  identifiers, and SQLite-owned projection persistence.
- Completed FRA-110 by keeping Admin/Public generated contract separation
  guarded and replacing raw `JobResponse` payload echo with explicit redacted
  job flags plus a safe managed-artwork failure-code projection.
- Completed FRA-120 by tightening frontend/SDK ignore hygiene and making the
  Admin API generated contract reproducible through `apps/admin-web`
  `generate:admin-api` and `verify`.
- Completed FRA-130 by deleting the `taru-api` root-level compatibility
  re-export shim, converting server/app/http/tests to explicit API boundary
  imports, and updating stale HTTP job-envelope docs to the redacted
  `has_input`/`has_summary`/`has_error` shape.
- Completed FRA-140 by refreshing closeout docs and running the full workspace
  verification gates.

## Blockers

- None.

## Next Recommended Action

No active task remains in this workstream.

Recommended future work should be opened as separate, narrower lanes when
prioritized:

1. Production PostgreSQL adapter operations beyond the contract proof.
2. Optional extraction of SQLite/PostgreSQL implementation crates after the
   facade boundary stabilizes.
3. Search backend selection or hybrid indexing once product search needs
   exceed the semantic projection contract.
4. Network tunnel provider design/implementation.
5. AI automation runtime and artifact acceptance policy.
6. Further Admin Console read/write surfaces.
