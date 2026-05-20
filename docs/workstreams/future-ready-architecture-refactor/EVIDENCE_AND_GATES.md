# Future-Ready Architecture Refactor — Evidence And Gates

Status: Completed
Last updated: 2026-05-20

## Smallest Current Repro

The current executable repro is the SQLite-backed job lease persistence
contract:

```bash
cargo nextest run -p taru-db sqlite_job_lease_contract --no-fail-fast
```

## Gate Set

### Planning Gate

```bash
git diff --check
```

Proves workstream docs do not contain whitespace errors and can be reviewed
before code changes begin.

### Persistence Iteration Gate

```bash
cargo check -p taru-core --tests
cargo check -p taru-db --tests
cargo nextest run -p taru-db --no-fail-fast
git diff --check
```

Proves backend-neutral persistence contracts and the SQLite adapter still build
and pass focused database behavior tests.

### Server Runtime Gate

```bash
cargo check -p taru-server --tests
cargo nextest run -p taru-server --no-fail-fast
git diff --check
```

Use narrower server filters during iteration when the touched surface is
specific. Record the filter and what it proves.

### Domain Seam Gates

```bash
cargo check -p taru-library --tests
cargo nextest run -p taru-library --no-fail-fast
cargo check -p taru-metadata --tests
cargo nextest run -p taru-metadata --no-fail-fast
cargo check -p taru-search --tests
cargo check -p taru-catalog --tests
git diff --check
```

Use only the relevant subset for each task, then broaden before closeout.

### API And Generated Contract Gates

```bash
cargo check -p taru-api --tests
cargo check -p taru-server --tests
```

When frontend or TypeScript SDK files are touched, also run the package-level
verify command where practical:

```bash
npm run verify
```

from the touched package directory.

### Broader Closeout Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

Use a narrower closeout gate only when the workspace is too large or a platform
dependency makes the full gate impractical. Explain the reason in this file.

### Review Gate

Before marking implementation tasks complete, run the appropriate workstream
review/verification workflow:

- `review-workstream` for task/lane compliance and code quality;
- `verify-rust-workstream` before completion claims with fresh command
  evidence.

## Evidence Anchors

- `docs/workstreams/future-ready-architecture-refactor/DESIGN.md`
- `docs/workstreams/future-ready-architecture-refactor/TODO.md`
- `docs/workstreams/future-ready-architecture-refactor/MILESTONES.md`
- `docs/workstreams/future-ready-architecture-refactor/WORKSTREAM.json`
- `docs/workstreams/future-ready-architecture-refactor/HANDOFF.md`
- Future `docs/workstreams/future-ready-architecture-refactor/JOURNAL/*.md`
  notes for deep dives and deletion inventories.

## Recorded Evidence

### 2026-05-20 — FRA-010 Workstream Open

Evidence:

- Opened the workstream docs.
- Recorded priority order and deletion rules.
- Confirmed this lane supersedes neither completed
  `core-architecture-deepening` nor completed `repository-seam-deepening`; it
  starts a broader PostgreSQL-ready architecture pass.

Validation to record before handoff:

```bash
git diff --check
```

### 2026-05-20 — FRA-020 Persistence Architecture Deep Dive

Evidence:

- Inventoried the current `SqliteStore` surface and confirmed it implements 26
  repository/search/lifecycle traits.
- Identified SQLite-specific assumptions that must be isolated before
  PostgreSQL: SQLx concrete types, SQLite migrations, `strftime` timestamps,
  transaction-local lease claims, JSON storage, ID storage, and widespread
  `SqliteStore::connect_in_memory()` tests.
- Accepted ADR 0029, choosing a `taru-db` facade plus backend-specific adapter
  target with SQLite first and future PostgreSQL.
- Chose FRA-030's first proof target: backend-neutral job lease lifecycle
  contract tests.

Validation:

```bash
git diff --check
```

Evidence anchors:

- `docs/adr/0029-postgresql-ready-persistence-boundary.md`
- `docs/workstreams/future-ready-architecture-refactor/JOURNAL/2026-05-20-fra-020.md`

### 2026-05-20 — FRA-030 Backend-Neutral Job Lease Contract

Evidence:

- Split durable job leasing out of the base `JobRepository` into
  `JobLeaseRepository`, so ordinary job CRUD/listing and leased worker
  ownership are separate persistence contracts.
- Added `crates/taru-db/src/contract_tests.rs` as the first backend-neutral
  contract harness. It runs through repository traits rather than SQLite row
  internals.
- The first contract suite proves:
  - filtered lease claim selects only claimable queued jobs and records worker
    ownership;
  - heartbeat, success, and failure are fenced by the run token;
  - cancellation requests are durable and only the owning lease can acknowledge
    running-job cancellation;
  - lease recovery fails only expired running leases.
- Moved duplicate SQLite-only job lease lifecycle tests out of
  `crates/taru-db/src/tests.rs`; SQLite-specific listing/startup recovery tests
  remain there.

Validation:

```bash
cargo check -p taru-core --tests
cargo check -p taru-db --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-db sqlite_job_lease_contract --no-fail-fast
cargo nextest run -p taru-db --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Results:

- `taru-core` check passed.
- `taru-db` check passed.
- `taru-server` check passed after importing the new `JobLeaseRepository`
  trait where durable lease methods are used.
- Focused contract nextest passed: 4 tests passed, 57 skipped.
- Full `taru-db` nextest passed: 61 tests passed.
- Formatting check passed.
- Diff whitespace check passed with Git CRLF normalization warnings only.

Broader gates not run:

- Workspace nextest was not run for FRA-030 because the code change is limited
  to `taru-core` job repository traits, `taru-db` job lease implementation and
  tests, and `taru-server` imports needed by the trait split. The full
  `taru-db` package suite plus `taru-server` check covers the touched behavior.

### 2026-05-20 — FRA-040 SQLite-Owned Adapter Split

Status: complete.

Evidence:

- Moved SQLite store identity into `crates/taru-db/src/sqlite.rs` and kept
  `SqliteStore` crate-private so downstream crates cannot bind to the concrete
  SQLite adapter.
- Moved durable job repository implementation from the root `jobs.rs` module
  to `crates/taru-db/src/sqlite/jobs.rs`, keeping the FRA-030
  backend-neutral job lease contract as the safety rail.
- Moved the SQLite-backed `SearchIndex` implementation from root `search.rs`
  to `crates/taru-db/src/sqlite/search.rs` because the current search
  projection is table-backed SQLite behavior, not facade-owned API shape.
- Moved `LibraryRepository`, `LibraryItemRepository`, `MetadataRepository`, and
  `ScanRepository` implementations from root modules to
  `crates/taru-db/src/sqlite/{library,library_item,metadata,scan}.rs`.
  This keeps workflow commit transaction ordering, search projection writes,
  and library-scoped source state SQL in the SQLite-owned adapter layer.
- Moved `MediaRepository`, `MediaProbeRepository`, `ProviderMappingRepository`,
  and `CatalogRepository` implementations from root modules to
  `crates/taru-db/src/sqlite/{media,provider_mapping,catalog}.rs`.
- Rewired `metadata`, `scan`, and `search` workflow helpers to call the
  SQLite-owned media, provider-mapping, and catalog transaction helpers through
  `crate::sqlite::*` paths instead of root facade paths.
- Moved `IngestionFailureRepository`, `LocalInferenceRepository`,
  `StagingManifestRepository`, `TranscodeSessionRepository`, and
  `UserPlaybackStateRepository` implementations from root modules to
  `crates/taru-db/src/sqlite/{ingestion,local_inference,staging,playback,user_playback}.rs`.
- Rewired scan workflow commits to call SQLite-owned ingestion and local
  inference transaction helpers.
- Moved Addon, automation, event outbox, webhook, catalog governance, source
  duplicate, VFS cache, and artwork persistence implementations from root
  modules to `crates/taru-db/src/sqlite/`.
- After the seventh slice, `crates/taru-db/src` contained only facade, codec,
  contract tests, SQLite module declaration, crate root, and SQLite-specific
  tests as root Rust files. Repository implementations lived under `sqlite/`.
- Moved the SQLite row-codec/helper module from root `codec.rs` to
  `crates/taru-db/src/sqlite/codec.rs`; the root crate now contains only
  `facade.rs`, `sqlite.rs`, `contract_tests.rs`, `tests.rs`, and `lib.rs`.
- Moved SQLite runtime options and connection construction from the root
  `runtime.rs` module to `crates/taru-db/src/sqlite/runtime.rs`.
- Moved SQLite migrations and `TransactionManager` implementation from the
  root `migrations.rs` module to `crates/taru-db/src/sqlite/migrations.rs`,
  making migration SQL clearly SQLite-owned.
- Added the public `TaruDatabase` facade in `crates/taru-db/src/facade.rs` and
  delegated repository/search/transaction traits to the active backend adapter.
- Updated `taru-server`, `taru-library`, `taru-metadata`, `taru-nfo`,
  `taru-automation`, `taru-catalog`, and `taru-events` touched test/service
  surfaces to construct or receive `TaruDatabase` instead of `SqliteStore`.
- Moved the backend-neutral job lease contract tests to construct
  `TaruDatabase::connect_in_memory()`, proving the contract through the facade
  that future PostgreSQL will also need to satisfy.
- Removed root-level `taru-db` import concentration from `lib.rs`; the current
  public root exposes only `TaruDatabase` and `SqliteRuntimeOptions`.
- Deleted the broad internal `sqlite::prelude` shim. SQLite modules now import
  `SqliteStore`, SQLite codec helpers, `taru_core`, `taru_search`, and `sqlx`
  types explicitly from their owning modules.
- Moved the main `taru-db` behavior tests to construct `TaruDatabase` instead
  of `SqliteStore`, so ordinary persistence behavior is now exercised through
  the same facade boundary used by server and downstream crates. SQLite-only
  migration/runtime/workflow rollback tests remain inside SQLite-owned modules.
- Confirmed the root `crates/taru-db/src` file set is now only:
  `contract_tests.rs`, `facade.rs`, `lib.rs`, `sqlite.rs`, and `tests.rs`.

Validation:

```bash
cargo check -p taru-core --tests
cargo check -p taru-db --tests
cargo check -p taru-server --tests
cargo check -p taru-automation --tests
cargo check -p taru-catalog --tests
cargo check -p taru-events --tests
cargo check -p taru-library --tests
cargo check -p taru-metadata --tests
cargo check -p taru-nfo --tests
cargo check --workspace --tests
cargo nextest run -p taru-db sqlite_job_lease_contract --no-fail-fast
cargo nextest run -p taru-db --no-fail-fast
cargo nextest run -p taru-server --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Results:

- `taru-core`, `taru-db`, `taru-server`, `taru-automation`, `taru-catalog`,
  `taru-events`, `taru-library`, `taru-metadata`, and `taru-nfo` checks
  passed.
- Workspace test compilation passed.
- Focused job lease contract nextest passed: 4 tests passed, 57 skipped.
- Full `taru-db` nextest passed: 61 tests passed.
- Full `taru-server` nextest passed: 173 tests passed.
- After moving search implementation under `sqlite/search.rs`, the refreshed
  `taru-db` check passed, full `taru-db` nextest still passed with 61 tests,
  and `taru-server` check passed.
- After moving library, library item, metadata, and scan implementations under
  `sqlite/`, the refreshed full `taru-db` nextest still passed with 61 tests,
  `taru-server` check passed, and workspace test compilation passed.
- After moving media, provider mapping, and catalog implementations under
  `sqlite/`, the refreshed full `taru-db` nextest still passed with 61 tests,
  `taru-server` check passed, and workspace test compilation passed.
- After moving ingestion, local inference, staging, playback, and user playback
  implementations under `sqlite/`, the refreshed full `taru-db` nextest still
  passed with 61 tests, `taru-server` check passed, and workspace test
  compilation passed.
- After moving Addon, automation, event outbox, webhook, catalog governance,
  source duplicate, VFS cache, and artwork implementations under `sqlite/`,
  the refreshed full `taru-db` nextest still passed with 61 tests,
  `taru-server` check passed, and workspace test compilation passed.
- After moving the SQLite row-codec/helper module under `sqlite/`, the
  refreshed full `taru-db` nextest still passed with 61 tests, `taru-server`
  check passed, and workspace test compilation passed.
- After deleting `sqlite::prelude` and moving main `taru-db` behavior tests to
  `TaruDatabase`, refreshed `cargo check -p taru-db --tests` passed.
- Fresh verification before marking FRA-040 complete:
  - `cargo nextest run -p taru-db --no-fail-fast` passed: 61 tests passed.
  - `cargo check -p taru-server --tests` passed.
  - `cargo check --workspace --tests` passed.
  - `cargo nextest run -p taru-server --no-fail-fast` passed: 173 tests
    passed.
  - `cargo fmt --all -- --check` passed after applying `cargo fmt --all`.
  - `git diff --check` passed with Git CRLF normalization warnings only.
- Formatting check passed after applying `cargo fmt --all`.
- Diff whitespace check passed with Git CRLF normalization warnings only.
- A focused ripgrep check across server and downstream crates found no
  remaining production/test references to `SqliteStore` outside `taru-db`.

FRA-040 closeout notes:

- Repository implementations no longer live as root modules inside `taru-db`;
  the remaining root files are facade, contract tests, SQLite module
  declaration, crate root, and SQLite-specific tests.
- The transitional internal `sqlite::prelude` has been deleted rather than
  carried forward as a compatibility shim.
- `TaruDatabase` is currently a simple single-backend facade. FRA-050/FRA-060
  should decide whether the future PostgreSQL proof uses an enum backend,
  trait-object backend, feature-selected adapter crate, or compile-only
  skeleton.
- Workspace nextest was not run for FRA-040 closeout. The changed behavior is
  concentrated in `taru-db` persistence boundaries plus server construction and
  HTTP/app usage. Full `taru-db` nextest, full `taru-server` nextest, and
  workspace test compilation provide the required fresh coverage for this
  slice without spending a whole-workspace runtime gate before FRA-050.

### 2026-05-20 — FRA-050 PostgreSQL-Ready Dialect And Migration Policy

Status: complete.

Evidence:

- Inventoried SQLite-specific assumptions now isolated under `taru-db::sqlite`
  and the SQLite migration tree:
  - 30 embedded SQLite migration files;
  - SQLite timestamp expressions (`strftime(...)`);
  - SQLite positional bind markers (`?1`, `?2`, ...);
  - SQLite `ON CONFLICT` upserts;
  - `LIMIT`/`OFFSET` pagination SQL;
  - `*_json` text payload storage and `serde_json` codecs;
  - integer boolean codecs;
  - ID string bind and parse helpers;
  - SQLx SQLite transactions and `SqliteRow` row access;
  - SQLite runtime PRAGMAs for foreign keys, WAL, and busy timeout;
  - millisecond timestamp columns for cache/staging/ingestion/playback facts.
- Accepted ADR 0030, which defines:
  - backend-owned migration trees and backend-local migration versions;
  - `DatabaseLifecycle` as the truthful startup lifecycle trait;
  - logical schema policy for IDs, provider keys, enums, JSON, booleans,
    unsigned integers, audit timestamps, and millisecond timestamps;
  - backend-specific SQL dialect policy;
  - job lease locking policy for SQLite vs future PostgreSQL;
  - row-codec ownership;
  - backend-neutral contract fixture rules;
  - FRA-060's proof target.
- Renamed the misleading `TransactionManager` trait to `DatabaseLifecycle` and
  moved its file from `crates/taru-core/src/repository/transaction.rs` to
  `crates/taru-core/src/repository/lifecycle.rs`.
- Updated `taru-db`, `taru-server`, `taru-library`, `taru-metadata`,
  `taru-nfo`, `taru-automation`, `taru-catalog`, and `taru-events` imports to
  use `DatabaseLifecycle`.
- Did not keep a `TransactionManager` compatibility alias, because Taru is not
  live and this workstream explicitly prefers deletion over compatibility
  shims.
- Chose FRA-060's next proof: a reusable backend contract-test harness with
  SQLite always on and optional PostgreSQL runtime coverage. If PostgreSQL code
  is added in FRA-060, it must have a real connection/lifecycle shape and at
  least one meaningful contract target, preferably job leases.

Validation:

```bash
cargo check -p taru-core --tests
cargo check -p taru-db --tests
cargo nextest run -p taru-db migrations --no-fail-fast
cargo nextest run -p taru-db sqlite_job_lease_contract --no-fail-fast
cargo nextest run -p taru-db --no-fail-fast
cargo check -p taru-server --tests
cargo check --workspace --tests
cargo fmt --all -- --check
git diff --check
```

Results:

- `cargo check -p taru-core --tests` passed after the lifecycle trait rename.
- `cargo check -p taru-db --tests` passed after the lifecycle trait rename.
- `cargo nextest run -p taru-db migrations --no-fail-fast` passed: 2 tests
  passed, 59 skipped. This proves SQLite migration splitting and rollback
  behavior still work after the lifecycle rename.
- `cargo nextest run -p taru-db sqlite_job_lease_contract --no-fail-fast`
  passed: 4 tests passed, 57 skipped. This proves the current backend-neutral
  job lease contract still runs through the `TaruDatabase` facade.
- `cargo nextest run -p taru-db --no-fail-fast` passed: 61 tests passed. This
  proves the SQLite adapter and facade behavior still pass the full `taru-db`
  package suite after the policy/code changes.
- `cargo check -p taru-server --tests` passed, proving server startup and app
  code import/use `DatabaseLifecycle` correctly.
- `cargo check --workspace --tests` passed after formatting, proving all
  workspace tests compile with the lifecycle trait rename and facade imports.
- `cargo fmt --all -- --check` passed after applying `cargo fmt --all`.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Workspace nextest was not run for FRA-050. The code change is a trait rename
  plus docs/policy; full `taru-db` nextest, server check, and workspace test
  compilation cover the touched behavior before FRA-060 begins.

### 2026-05-20 — FRA-060 PostgreSQL Job Lease Readiness Proof

Status: complete.

Evidence:

- Reshaped `crates/taru-db/src/contract_tests.rs` into a reusable backend
  contract harness for the job lease contract.
- Kept SQLite as the always-on contract backend.
- Added ignored PostgreSQL contract tests gated by `TARU_TEST_POSTGRES_URL`.
- Added test-only `crates/taru-db/src/postgres.rs` with a real PostgreSQL
  connection/lifecycle/job/job-lease proof implementation.
- Added backend-owned PostgreSQL proof migration:
  `crates/taru-db/migrations/postgres/0001_contract_jobs.sql`.
- Scoped PostgreSQL SQLx features to `taru-db` dev-dependencies instead of
  enabling PostgreSQL in the production workspace dependency by default.
- PostgreSQL proof uses native PostgreSQL behavior:
  - UUID columns for Taru UUID-backed IDs;
  - `jsonb` for library fixture JSON;
  - `timestamptz` timestamps;
  - PostgreSQL `$n` bind markers;
  - `FOR UPDATE SKIP LOCKED` for lease claim selection.
- PostgreSQL contract tests create and drop a temporary schema per test, with a
  process-local mutex to serialize the ignored PostgreSQL tests.

Validation:

```bash
cargo check -p taru-db --tests
cargo nextest run -p taru-db sqlite_job_lease_contract --no-fail-fast
cargo nextest run -p taru-db postgres_job_lease_contract --run-ignored only --no-fail-fast
cargo nextest run -p taru-db --no-fail-fast
cargo check --workspace --tests
cargo fmt --all -- --check
git diff --check
```

Results:

- `cargo check -p taru-db --tests` passed.
- `cargo nextest run -p taru-db sqlite_job_lease_contract --no-fail-fast`
  passed: 4 tests passed, 61 skipped. This proves SQLite still satisfies the
  backend-neutral job lease contract through the reshaped harness.
- `cargo nextest run -p taru-db postgres_job_lease_contract --run-ignored only
  --no-fail-fast` passed with `TARU_TEST_POSTGRES_URL` unset in this session:
  4 ignored PostgreSQL contract tests compiled and exercised the env-gated
  skip path, with 61 other tests skipped by the filter. This proves the
  PostgreSQL proof code is type-checked and opt-in by default. A real
  PostgreSQL runtime pass still requires setting `TARU_TEST_POSTGRES_URL` and
  recording the refreshed command output.
- `cargo nextest run -p taru-db --no-fail-fast` passed: 61 tests passed, 4
  skipped. This proves default `taru-db` behavior remains SQLite-only and the
  PostgreSQL contract tests are opt-in.
- `cargo check --workspace --tests` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Workspace nextest was not run for FRA-060. The code change is concentrated in
  `taru-db` test-only PostgreSQL readiness proof plus dependency metadata. Full
  `taru-db` nextest, optional PostgreSQL contract nextest, and workspace test
  compilation cover the slice.

### 2026-05-20 — FRA-070 Server Runtime Composition

Status: complete.

Evidence:

- Added `crates/taru-server/src/app/composition.rs` as the server composition
  runtime boundary.
- Moved the `TaruApp::new_with_store` inline construction block into
  `TaruAppComposition::build`.
- Introduced `TaruRuntimeResources` to own construction of process-local
  runtime resources:
  - `RuntimeSupervisor`;
  - `StorageBackendRegistry`;
  - scan, metadata/NFO/Addons, and webhook concurrency permits;
  - metadata provider registry.
- Introduced `TaruAppServices` to own construction of cohesive app service
  handles for jobs, library scan, artwork, addons, automation, webhooks,
  catalog, library, storage diagnostics, metadata, NFO, playback, and user
  playback.
- Kept startup ordering in one place by running `ServerStartupWorkflow` from
  the composition module after service construction and before optional artwork
  ingest worker registration.
- Deleted the old inline `TaruAppInner` construction path. `TaruApp` now keeps
  the public app state/API surface and delegates runtime/service construction
  to the composition module.
- Confirmed a focused search no longer finds `TaruAppInner`, `SqliteStore`,
  `TransactionManager`, or direct runtime construction calls inside
  `crates/taru-server/src/app.rs`.

Validation:

```bash
cargo check -p taru-server --tests
cargo nextest run -p taru-server startup --no-fail-fast
cargo nextest run -p taru-server app::tests::metadata --no-fail-fast
cargo nextest run -p taru-server --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Results:

- `cargo check -p taru-server --tests` passed after moving the shared test
  imports for `DatabaseLifecycle` and `Semaphore` into `app/tests/mod.rs`.
- `cargo nextest run -p taru-server startup --no-fail-fast` passed: 19 tests
  passed, 154 skipped. This proves startup migration/recovery/reconciliation,
  metadata lifecycle startup, and runtime-supervised scan job behavior still
  work through the new composition boundary.
- `cargo nextest run -p taru-server app::tests::metadata --no-fail-fast`
  passed: 12 tests passed, 161 skipped. This proves metadata provider
  registry construction, metadata permits, raw-cache lifecycle workers, and
  supervised metadata jobs still work through the composition boundary.
- `cargo nextest run -p taru-server --no-fail-fast` passed: 173 tests passed.
  This proves server app and HTTP behavior still pass after the composition
  split.
- `cargo fmt --all -- --check` passed after applying `cargo fmt --all`.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Workspace nextest was not run for FRA-070. The code change is concentrated in
  `taru-server` app composition, with no domain/schema/API changes. Full
  `taru-server` nextest plus formatting and diff checks cover this slice.

### 2026-05-20 — FRA-080 Local Inference Engine

Status: complete.

Evidence:

- Removed `ParsedName` from `DiscoveredMediaSource`, so the VFS scanner no
  longer stores naming-parser output in scan discovery records.
- Removed `DefaultNameParser` and `NameParser` use from
  `crates/taru-library/src/scan.rs`. `VfsLibraryScanner` now discovers media
  source facts only.
- Added `LocalInferenceEngine` in
  `crates/taru-library/src/local_inference.rs`.
- `LocalInferenceEngine` now owns:
  - parsing a discovered source path through `taru-naming`;
  - producing `LocalInferenceEvidence`;
  - building `SourceState` and `MediaSource` facts from the discovered source;
  - planning the primary provisional `MediaItem`;
  - planning provisional hierarchy ancestors such as Series and Season.
- `LibraryIndexService` now asks the engine for a `LocalInferencePlan`, then
  resolves/reuses planned ancestors through repository lookups and commits the
  resulting source/evidence/search projection through
  `commit_library_scan_source`.
- Confirmed metadata preservation remains in the indexing layer: non-
  provisional existing library items still bypass provisional item replacement
  while fresh local inference evidence is recorded for the source.

Validation:

```bash
cargo check -p taru-library --tests
cargo nextest run -p taru-library local_inference --no-fail-fast
cargo nextest run -p taru-library --no-fail-fast
cargo check -p taru-server --tests
cargo check --workspace --tests
cargo fmt --all -- --check
git diff --check
```

Results:

- `cargo check -p taru-library --tests` passed.
- `cargo nextest run -p taru-library local_inference --no-fail-fast` passed:
  4 tests passed, 13 skipped. This proves the new Local Inference Engine plans
  episode hierarchy and unknown flat items, and the indexing path still
  persists local inference evidence.
- `cargo nextest run -p taru-library --no-fail-fast` passed: 17 tests passed.
  This proves scanner discovery, indexing, inference, tombstoning, scan
  failures, probe behavior, and WebDAV staging behavior still pass after the
  seam split.
- `cargo check -p taru-server --tests` passed, proving server app callers still
  compile with the changed `taru-library` scan/inference boundary.
- `cargo check --workspace --tests` passed, proving all workspace tests compile
  after removing parsed naming output from scanner discoveries.
- `cargo fmt --all -- --check` passed after applying `cargo fmt --all`.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Workspace nextest was not run for FRA-080. The behavior change is
  concentrated in `taru-library` scan/index/local-inference code. Full
  `taru-library` nextest, server test compilation, and workspace test
  compilation cover this slice.

### 2026-05-20 — FRA-090 Metadata Candidate Graph

Status: complete.

Evidence:

- Added provider-neutral Metadata Candidate Graph records in
  `crates/taru-core/src/media/candidate.rs`:
  - `MetadataCandidateGraph`;
  - `MetadataCandidateNode`;
  - `MetadataCandidateRecord`;
  - `MetadataCandidateSubject`;
  - `MetadataCandidateRelationship`;
  - `MetadataCandidateSource`.
- Re-exported the candidate graph domain records through `taru-core::media`
  and `taru-metadata`.
- Changed `taru-metadata` provider-facing search/fetch DTOs so
  `MetadataCandidate` and `MetadataFetchResult` carry a
  `MetadataCandidateGraph` rather than direct `CanonicalMetadata`.
- Kept canonical projection explicit through `metadata()` and
  `canonical_metadata()` helpers. This makes the provider-neutral candidate
  seam visible while preserving current merge and API behavior.
- Rewired TMDB, Douban, and Bangumi mapping functions to produce
  `MetadataCandidateRecord`, then wrap the records in a provider-rooted
  candidate graph with subject kind, provider key, title, release year, and
  optional locale.
- Updated metadata refresh so field-lock-aware merge consumes the fetched
  candidate graph projection, while accepted provider mapping uses the root
  candidate subject facts instead of deriving the whole Provider Subject from
  already-merged Canonical Metadata.
- Added `NfoDocument::candidate_graph`, `NfoDocument::metadata()`, and
  `NfoDocument::from_metadata`. Movie NFO parsing now creates an NFO-rooted
  candidate graph, import merges the graph projection, and export builds the
  local candidate graph from the item being rendered. `NfoDocument` no longer
  stores a parallel `CanonicalMetadata` field.
- Added behavior tests proving:
  - TMDB provider payload maps into a provider-neutral candidate graph before
    Canonical Metadata projection;
  - NFO parsing produces a local candidate graph;
  - Douban and Bangumi accepted provider mappings retain subject facts from
    candidate graph roots.

Validation:

```bash
cargo check -p taru-metadata --tests
cargo check -p taru-nfo --tests
cargo nextest run -p taru-metadata metadata_candidate_graph --no-fail-fast
cargo nextest run -p taru-nfo metadata_candidate_graph --no-fail-fast
cargo nextest run -p taru-metadata metadata_refresh_accepts_douban_and_bangumi_provider_mappings --no-fail-fast
cargo nextest run -p taru-metadata --no-fail-fast
cargo nextest run -p taru-nfo --no-fail-fast
cargo check --workspace --tests
cargo fmt --all -- --check
git diff --check
```

Results:

- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-nfo --tests` passed.
- Focused metadata candidate graph nextest passed: 1 test passed, 28 skipped.
  This proves a TMDB provider payload can become a provider-neutral candidate
  graph before Canonical Metadata projection.
- Focused NFO candidate graph nextest passed: 1 test passed, 22 skipped. This
  proves Movie NFO parsing creates a local candidate graph whose projection
  matches the parsed metadata.
- Focused Douban/Bangumi provider mapping nextest passed: 1 test passed, 28
  skipped. This proves accepted provider mappings now retain candidate subject
  title and release-year facts for non-TMDB providers.
- Full `taru-metadata` nextest passed: 29 tests passed. This covers provider
  runtime, metadata refresh, strategy fallback, provider attempts, locks, raw
  response caching, provider mapping acceptance, and candidate graph behavior.
- Full `taru-nfo` nextest passed: 23 tests passed. This covers NFO parse,
  import, export, preservation, backup, cancellation, field locks, and the new
  candidate graph proof.
- `cargo check --workspace --tests` passed after the public DTO shape changed,
  proving server and workspace callers compile with the candidate graph seam.
- `cargo fmt --all -- --check` passed after applying `cargo fmt --all`.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Workspace nextest was not run for FRA-090. The public change is concentrated
  in `taru-core` candidate graph records, `taru-metadata` provider DTOs and
  mappings, and `taru-nfo` import/export/codec behavior. Full metadata and NFO
  package nextest plus workspace test compilation cover the touched runtime and
  API surface for this slice.

## Notes

Do not list commands without explaining what behavior they prove. Fresh
verification is required before marking any task, Codex goal, or lane complete.

### 2026-05-20 — FRA-100 Search Semantics

Status: complete.

Evidence:

- Added semantic search projection fields in `taru-core`:
  - `CATALOG_SEARCH_PROJECTION_VERSION`;
  - `CatalogSearchProjection::projection_version`;
  - `aliases`;
  - `browse_facets`;
  - `sort_keys`;
  - `provider_identifiers`;
  - `CatalogSearchProjection::searchable_text()`;
  - `CatalogSearchProjection::try_from_facet_labels()` for test and adapter fixtures.
- Added explicit Browse Facet and Sort Key records:
  - `BrowseFacet`;
  - `BrowseFacetKind` with keyed `ExternalId(provider)`, keyed `CreditRole(value)`, and generic `Other(key)` support;
  - `SortKey`;
  - `SortKeyKind`.
- Changed `taru-search` public records from raw string facets to semantic Browse Facets:
  - `SearchDocument` now carries projection version, aliases, and `browse_facets`;
  - `SearchQuery` now carries `browse_facets`;
  - legacy label conversion is explicit and fallible through `from_facet_labels`.
- Rewired catalog hydration to build semantic projections from Media Item facts, sources, genres, tags, collections, studios, credits, release year, provider IDs, aliases, and sort keys.
- Rewired library indexing search projection construction to use the same semantic fields instead of a raw `Vec<String>` facet bag.
- Updated the SQLite search table shape in the backend-owned migration to persist projection version, aliases JSON, sort keys JSON, and provider identifiers JSON alongside facet JSON/text.
- Updated SQLite search persistence to store `projection.searchable_text()` and exact facet labels from `browse_facets`.
- Updated SQLite search querying to:
  - include aliases in the text haystack;
  - parse `facets_json` and require exact case-insensitive facet-label equality;
  - stop using substring containment over `facets_text` for facet filtering.
- Updated server/catalog, NFO, metadata, addon, and library tests/callers to use explicit semantic query/document conversion.
- Added behavior tests proving:
  - `taru-search` emits keyed semantic facet labels such as `external_id:tmdb:603`;
  - legacy facet labels are parsed into semantic facets;
  - SQLite facet filtering does not match partial substrings (`genre:Science` does not match `genre:Science Fiction`);
  - aliases remain structured but are searchable;
  - catalog hydration builds a semantic projection searchable by alias, release-year facet, and provider-id facet.

Validation:

```bash
cargo check -p taru-search --tests
cargo check -p taru-catalog --tests
cargo check -p taru-db --tests
cargo check -p taru-library --tests
cargo check -p taru-metadata --tests
cargo check -p taru-nfo --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-search --no-fail-fast
cargo nextest run -p taru-db browse_facets_exactly --no-fail-fast
cargo nextest run -p taru-db searches_aliases --no-fail-fast
cargo nextest run -p taru-catalog semantic_search_projection --no-fail-fast
cargo nextest run -p taru-catalog --no-fail-fast
cargo nextest run -p taru-db taru_database_sqlite_round_trips_scan_state_search_and_artwork_tasks --no-fail-fast
cargo fmt --all --check
git diff --check
cargo check --workspace --tests
```

Results:

- `cargo check -p taru-search --tests` passed.
- `cargo check -p taru-catalog --tests` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-library --tests` passed.
- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-nfo --tests` passed.
- `cargo check -p taru-server --tests` passed.
- `cargo nextest run -p taru-search --no-fail-fast` passed: 2 tests passed.
- `cargo nextest run -p taru-db browse_facets_exactly --no-fail-fast` passed: 1 test passed, 66 skipped. This proves exact Browse Facet matching replaces substring matching.
- `cargo nextest run -p taru-db searches_aliases --no-fail-fast` passed: 1 test passed, 66 skipped. This proves aliases remain structured while participating in text search.
- `cargo nextest run -p taru-catalog semantic_search_projection --no-fail-fast` passed: 1 test passed, 3 skipped. This proves catalog hydration emits semantic aliases, release-year facets, and keyed provider-id facets.
- `cargo nextest run -p taru-catalog --no-fail-fast` passed: 4 tests passed. This covers the touched catalog hydration projection behavior.
- `cargo nextest run -p taru-db taru_database_sqlite_round_trips_scan_state_search_and_artwork_tasks --no-fail-fast` passed: 1 test passed, 66 skipped. This proves the existing scan/search/artwork persistence path still works with the semantic search table projection.
- `cargo fmt --all --check` passed after applying `cargo fmt --all`.
- `git diff --check` passed with Git CRLF normalization warnings only.
- `cargo check --workspace --tests` passed, proving all workspace test targets compile with the semantic search API shape.

Broader gates not run:

- Full workspace nextest was not run for FRA-100. The public API shape changed in the search/catalog seam, so workspace test compilation was run. Runtime behavior was verified with full `taru-search` and `taru-catalog` nextest plus focused SQLite DB search/indexing tests that cover exact facet matching, alias search, and the scan commit path.

### 2026-05-20 — FRA-110 Admin/API Contract Hygiene

Status: complete.

Evidence:

- Audited `crates/taru-api/src/admin.rs`, `admin_contract.rs`, `openapi.rs`,
  `sdk.rs`, `public_client.rs`, and the job-returning HTTP routes under
  `crates/taru-server/src/http`.
- Existing Admin API generated contract guardrails remain in place: the
  generated app-local contract covers the eight read-model routes recorded by
  `docs/workstreams/admin-api-typescript-contract/ADMIN_CONTRACT_INVENTORY.md`,
  and Public Client SDK/OpenAPI tests still reject `/admin` and internal
  surfaces.
- Removed raw `input`, `summary`, and `error` fields from `JobResponse`.
  `JobResponse` now exposes only job identity/status/resource fields plus
  `has_input`, `has_summary`, and `has_error` flags.
- Replaced the old NFO backup-summary-preservation test with
  `job_response_redacts_raw_payloads_summaries_and_errors`, proving raw job
  JSON and error strings are not serialized through `JobResponse`.
- Added a narrow, explicit `ManagedArtworkIngestSummary::failure_code` field
  so managed artwork failure diagnostics use a safe domain DTO field instead
  of raw job summary/error JSON.
- Updated HTTP tests for library scan, NFO import/export, metadata refresh,
  metadata maintenance, automation job enqueueing, managed artwork acceptance,
  managed artwork failure, and managed artwork requeue paths to assert
  redacted job responses and safe failure-code projection.

Validation:

```bash
cargo check -p taru-api --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-api --no-fail-fast
cargo nextest run -p taru-server scan_route_queues_background_job nfo_routes_queue_background_jobs metadata_refresh_route_queues_background_job metadata_maintenance_route_enqueues_batch_job automation_routes_configure_provider_and_enqueue_jobs_without_secrets admin_accept_artwork_candidate_queues_managed_ingest_without_public_artwork_or_url_echo admin_process_next_managed_artwork_ingest_fails_with_redacted_safe_summary_for_unsupported_media_type admin_managed_artwork_ingest_requeue_retries_failed_ingest_without_leaks admin_process_next_managed_artwork_ingest_fails_with_redacted_safe_summary_for_invalid_image --no-fail-fast
cargo fmt --all --check
git diff --check
```

Results:

- `cargo check -p taru-api --tests` passed.
- `cargo check -p taru-server --tests` passed.
- `cargo nextest run -p taru-api --no-fail-fast` passed: 41 tests passed.
  This covers Admin DTO redaction tests, generated Admin API contract tests,
  Public Client OpenAPI leakage tests, and Public Client TypeScript SDK
  leakage tests.
- Focused `taru-server` nextest passed: 9 tests passed, 164 skipped. This
  proves job-returning HTTP routes now return redacted job DTOs across library,
  metadata, automation, and managed artwork paths.
- `cargo fmt --all --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Full `taru-server` nextest and workspace nextest were not run for FRA-110.
  The implementation changed DTO projection and HTTP response assertions, not
  persistence/runtime behavior. Full `taru-api` nextest covers OpenAPI, SDK,
  admin contract, and DTO redaction guardrails; focused `taru-server` nextest
  covers every touched job-returning HTTP route family.

### 2026-05-20 — FRA-120 Generated/Frontend Repository Hygiene

Status: complete.

Evidence:

- Audited current frontend/SDK generated artifacts and ignored files:
  `apps/admin-web/dist`, `apps/admin-web/.playwright-cli`,
  `apps/admin-web/.vite-server.*.log`,
  `apps/admin-web/tsconfig.tsbuildinfo`,
  `apps/admin-web/node_modules`, and `sdk/typescript/node_modules`.
- Expanded root `.gitignore` to cover frontend dependency/build outputs,
  coverage/test reports, Playwright artifacts, TypeScript/Vite local artifacts,
  and package-manager debug logs.
- Expanded `apps/admin-web/.gitignore` for app-local build, coverage,
  Playwright, Vite, TypeScript build-info, and debug-log artifacts.
- Added `sdk/typescript/.gitignore` for SDK package-local build, coverage,
  TypeScript build-info, and debug-log artifacts.
- Added `apps/admin-web` script `generate:admin-api`, backed by
  `taru-api --example emit-admin-typescript-contract`, and wired
  `npm run verify --prefix apps/admin-web` to regenerate the Admin API
  contract before `check`, `test`, and `build`.
- Updated `apps/admin-web/README.md` to document the generated Admin API
  contract refresh command.
- Did not delete ignored local artifacts or revert any working-tree changes.
  Regenerating the Admin API contract and Public Client TypeScript SDK produced
  no textual diff in the generated tracked files.

Validation:

```bash
git check-ignore -v apps/admin-web/dist apps/admin-web/.vite-server.err.log apps/admin-web/.playwright-cli/console-2026-05-19T14-31-31-067Z.log apps/admin-web/tsconfig.tsbuildinfo sdk/typescript/tsconfig.tsbuildinfo
npm run generate:admin-api --prefix apps/admin-web
npm run verify --prefix apps/admin-web
npm run verify --prefix sdk/typescript
cargo fmt --all --check
git diff --check
git status --short -- .gitignore apps/admin-web sdk/typescript docs/workstreams/future-ready-architecture-refactor
```

Results:

- `git check-ignore -v ...` showed the expected package/root ignore rules for
  admin-web build/log/Playwright/tsbuildinfo artifacts and SDK tsbuildinfo
  artifacts.
- `npm run generate:admin-api --prefix apps/admin-web` passed and produced no
  textual diff in `apps/admin-web/src/adminApi/generated/contract.ts`.
- `npm run verify --prefix apps/admin-web` passed: generated Admin API
  contract, TypeScript check, Vitest suite, and Vite build all completed. The
  app test suite reported 3 files and 9 tests passed.
- `npm run verify --prefix sdk/typescript` passed: public SDK generation and
  `tsc --noEmit` completed, with no textual diff in `sdk/typescript/src/index.ts`.
- `cargo fmt --all --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
- Focused `git status --short` showed only intended hygiene/doc changes in the
  frontend/SDK scope plus the active workstream docs; ignored local artifacts
  remained ignored.

Broader gates not run:

- Rust workspace checks were not rerun for FRA-120 because this slice only
  changed ignore rules, frontend package scripts/docs, and package-local
  hygiene. The Rust generator code was exercised through the admin-web and SDK
  generation commands.

### 2026-05-20 — FRA-130 Deletion Sweep

Status: complete.

Deletion inventory:

- Deleted the `taru-api` root-level compatibility facade:
  `pub use admin::*`, `pub use extension::*`,
  `pub use metadata_diagnostics::*`, and `pub use public_client::*`.
  `taru-api` now exposes explicit boundary modules only.
- Updated `taru-api` internal callers to import Public Client constants,
  OpenAPI helpers, and managed-artwork image-reference helpers through their
  owning modules instead of relying on root re-exports.
- Updated `taru-server` app services, HTTP handlers, and tests to import DTOs
  through explicit `taru_api::{admin, extension, metadata_diagnostics,
  public_client}` module paths.
- Updated `docs/api/HTTP_API.md` to remove stale raw job `input`, `summary`,
  and `error` response examples and document the redacted
  `has_input`/`has_summary`/`has_error` envelope.
- Added the safe `failure_code` field to the managed-artwork accept-response
  example so docs match the FRA-110 DTO shape.

Candidates audited but intentionally kept:

- `crates/taru-api/src/admin.rs` still re-exports
  `admin::managed_artwork::*` inside the Admin API module. This is not the
  deleted root compatibility shim; it is the current Admin API aggregation
  surface and can be split later if the admin module itself becomes too broad.
- `taru-search` still exposes fallible facet-label conversion helpers. They
  are explicit adapters from current HTTP/query/test string facets into
  semantic Browse Facets, not a hidden old/new production path.

Validation:

```bash
cargo check -p taru-api --tests
cargo check -p taru-server --tests
cargo check --workspace --tests
cargo nextest run -p taru-api --no-fail-fast
cargo nextest run -p taru-server --no-fail-fast
cargo fmt --all
cargo fmt --all -- --check
git diff --check
```

Results:

- `cargo check -p taru-api --tests` passed.
- `cargo check -p taru-server --tests` passed.
- `cargo check --workspace --tests` passed, proving all workspace test targets
  compile without the deleted `taru-api` root compatibility exports.
- `cargo nextest run -p taru-api --no-fail-fast` passed: 41 tests passed.
  This covers Admin DTO redaction, Admin generated contract, Public Client
  OpenAPI, and Public Client SDK guardrails after the API boundary deletion.
- `cargo nextest run -p taru-server --no-fail-fast` passed: 173 tests passed.
  This covers the explicit API module imports across app services, HTTP route
  handlers, and route tests.
- `cargo fmt --all` was run to apply rustfmt formatting after import rewrites.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Full workspace nextest was not run for FRA-130. The slice removed an API
  compatibility shim and updated imports/docs, so workspace test compilation
  plus full `taru-api` and full `taru-server` nextest cover the touched
  public-boundary and route behavior. Frontend/SDK package verification was
  not rerun because generated TypeScript artifacts were not changed in this
  slice.

### 2026-05-20 — FRA-140 Closeout

Status: complete.

Closeout review:

- M0 through M4 tasks are complete in `TODO.md`.
- ADR 0029 and ADR 0030 record the persistence boundary and SQL
  dialect/migration policy.
- The `taru-db` root public surface is a facade plus SQLite runtime options;
  SQLite implementation details are under SQLite-owned modules.
- Backend-neutral job lease contracts are present, with SQLite always-on and
  PostgreSQL proof coverage ignored unless `TARU_TEST_POSTGRES_URL` is set.
- Runtime composition, Local Inference, Metadata Candidate Graph, semantic
  search, Admin/API redaction, frontend/SDK hygiene, and the final deletion
  sweep all have task-local evidence above.
- No old/new production path remains without a named explanation in this
  workstream. Remaining large ideas are future product/architecture scope:
  production PostgreSQL operations, optional backend crate extraction, richer
  search engines, network tunneling, AI automation runtime, and more Admin UI
  surfaces.

Validation:

```bash
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Results:

- `cargo check --workspace --tests` passed.
- `cargo nextest run --workspace --no-fail-fast` passed: 466 tests passed,
  4 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- None for this lane. The full workspace nextest gate was run for closeout.
