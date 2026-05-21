# PostgreSQL Production Readiness — Evidence And Gates

Status: Completed
Last updated: 2026-05-20

This file records fresh evidence for M62. Do not claim PostgreSQL production
readiness without command evidence that matches the claim scope.

## Gate Policy

Always-on gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run` for touched crates and contract families
- `git diff --check`

PostgreSQL opt-in gates:

- Set `TARU_TEST_POSTGRES_URL` to a test PostgreSQL database URL.
- Run PostgreSQL contract tests with explicit ignored-test inclusion once the
  relevant contract family exists.
- Tests must create isolated schemas or otherwise avoid clobbering developer
  databases.

Skip policy:

- A PostgreSQL gate may be skipped only when `TARU_TEST_POSTGRES_URL` is not
  available. The evidence entry must state that the gate was skipped and what
  SQLite/compile evidence still proves.
- Ignored PostgreSQL contract tests must fail fast when
  `TARU_TEST_POSTGRES_URL` is absent. A green ignored-test run without a real
  PostgreSQL URL is not valid PostgreSQL evidence.

## Evidence

### 2026-05-20 — PGR-010 Workstream Opened

Status: complete.

Evidence:

- Created `docs/workstreams/postgresql-production-readiness/`.
- Recorded current state from M61:
  - `TaruDatabase` currently dispatches to SQLite only in production code.
  - `PostgresStore` is test-only and proves libraries/jobs/job leases.
  - `crates/taru-db/src/contract_tests.rs` has SQLite always-on and
    PostgreSQL ignored opt-in job lease contracts.
  - PostgreSQL currently has one proof migration:
    `crates/taru-db/migrations/postgres/0001_contract_jobs.sql`.
- Defined the first backend contract-test matrix covering lifecycle, library,
  jobs/leases, media/source, scan, metadata/catalog, search, user playback,
  transcode sessions, event/webhook/addon/automation, Managed Artwork, and
  VFS/staging state.
- Updated top-level docs so M62 is visible as the active implementation goal.

Validation:

```bash
git diff --check
```

Result:

- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Rust workspace gates are not required for PGR-010 because this task only
  opens planning/workstream docs and does not change code.

### 2026-05-20 — PGR-020 Backend Selection Seam

Status: complete.

Implementation evidence:

- Added `DatabaseBackendKind::{Sqlite, Postgres}` and
  `DatabaseConnectOptions` to `taru-db`.
- Added facade-level backend identity and capability reporting through
  `TaruDatabase::backend_kind()` and `TaruDatabase::capabilities()`.
- Added `TaruDatabase::connect_with_options(...)` so production construction
  is explicit instead of URL-guessing-only.
- Kept SQLite as the default production and test backend.
- Added `TaruServerConfig.database_backend` with a default of `sqlite`.
- Updated `TaruApp::new` to pass explicit backend options into `TaruDatabase`.
- Kept PostgreSQL production runtime honest by returning a named unsupported
  error until the M62 contract parity matrix and migrations are implemented.

Validation:

```bash
cargo check -p taru-db --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-db backend_kind backend_kind_rejects sqlite_connect_options taru_database_connect_options_select_sqlite_explicitly taru_database_rejects_postgres_until_contract_parity_is_complete taru_database_sqlite_persists_libraries
cargo nextest run -p taru-server config_round_trips_from_toml config_uses_default_runtime_settings config_accepts_explicit_postgres_backend_without_inferring_from_url
git diff --check
```

Result:

- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-server --tests` passed.
- Focused `taru-db` nextest passed: 6 passed, 66 skipped.
- Focused `taru-server` nextest passed: 3 passed, 171 skipped.
- `git diff --check` passed with Git CRLF normalization warnings only.

PostgreSQL gate:

- Runtime PostgreSQL startup is intentionally not enabled by PGR-020. The
  existing ignored PostgreSQL contract proof remains opt-in for job leases;
  broader PostgreSQL gates start in PGR-030/PGR-040 after the harness and
  schema parity slices are expanded.

### 2026-05-20 — PGR-030 Contract Harness Generalization

Status: complete.

Implementation evidence:

- Reworked `crates/taru-db/src/contract_tests.rs` around explicit contract
  metadata:
  - `ContractFamily`
  - `ContractSetup`
  - `ContractCase`
- Added `database_contract_pair!` so each contract family registers SQLite
  always-on and PostgreSQL opt-in variants from the same contract function.
- Preserved isolated PostgreSQL schema setup and serialized PostgreSQL
  contract execution.
- Added a lifecycle contract family proving idempotent `migrate()` behavior.
- Kept the job-lease contract family green through the new runner.

Validation:

```bash
cargo fmt --all
cargo check -p taru-db --tests
cargo nextest run -p taru-db contract --no-fail-fast
cargo nextest run -p taru-db contract --run-ignored ignored-only --no-fail-fast
git diff --check
```

Result:

- `cargo check -p taru-db --tests` passed.
- SQLite/default contract run passed: 6 passed, 68 skipped.
- PostgreSQL ignored contract run passed: 5 passed, 69 skipped.
- `git diff --check` passed with Git CRLF normalization warnings only.

PostgreSQL gate:

- `cargo nextest run -p taru-db contract --run-ignored ignored-only
  --no-fail-fast` passed in this environment, which means
  `TARU_TEST_POSTGRES_URL` was available and the existing PostgreSQL proof
  contracts still pass through the generalized harness.

### 2026-05-20 — PGR-040 Library/Media Source PostgreSQL Parity

Status: complete.

Implementation evidence:

- Added a `LibraryMedia` contract family in
  `crates/taru-db/src/contract_tests.rs`.
- The new contract proves:
  - Media Library upsert/get/list behavior;
  - Media Item hierarchy and external ID round-trip behavior;
  - Media Source identity scoped by Media Library and Source Locator;
  - item-source and library-source listing;
  - Library Item State round-trip/list behavior;
  - library-scoped kind/parent/title lookup.
- Expanded `crates/taru-db/migrations/postgres/0001_contract_jobs.sql` with
  backend-owned PostgreSQL tables and indexes for:
  - `media_items`;
  - `media_item_external_ids`;
  - `media_sources`;
  - `library_item_states`.
- Implemented PostgreSQL `MediaRepository` and `LibraryItemRepository` methods
  required by the contract slice in `crates/taru-db/src/postgres.rs`.
- PostgreSQL uses native `uuid`, `jsonb`, and `boolean` storage for this slice.

Validation:

```bash
cargo fmt --all
cargo check -p taru-db --tests
cargo nextest run -p taru-db contract --no-fail-fast
cargo nextest run -p taru-db contract --run-ignored ignored-only --no-fail-fast
git diff --check
```

Result:

- `cargo check -p taru-db --tests` passed.
- SQLite/default contract run passed: 7 passed, 69 skipped.
- PostgreSQL ignored contract run passed: 6 passed, 70 skipped.
- `git diff --check` passed with Git CRLF normalization warnings only.

Review notes:

- The contract intentionally inserts the same Source Locator under two
  different Media Libraries to prove Source Locator identity remains
  library-scoped.
- This slice does not claim scan, metadata/catalog, search projection,
  playback, transcode, webhooks, Addons, automation, Managed Artwork, VFS, or
  staging PostgreSQL parity. Those remain tracked by PGR-050 and later tasks.

### 2026-05-20 — PGR-050 Library Scan Commit PostgreSQL Parity

Status: complete.

Implementation evidence:

- Added a `ScanCommit` contract family in
  `crates/taru-db/src/contract_tests.rs`.
- Added a success contract proving:
  - scan snapshot begin/complete behavior;
  - directory snapshot persistence;
  - Media Item and Media Source writes;
  - Source State persistence and listing;
  - Library Item State writes;
  - Local Inference Evidence round-trip/list behavior;
  - Search Projection side effects through `SearchIndex::search`;
  - ingestion failure resolution;
  - Media Technical Facts round-trip through `MediaProbeRepository`.
- Added a rollback contract proving `commit_library_scan_source` rolls back
  item/source/source-state/library-state/evidence writes and leaves the
  ingestion failure open when a Search Projection write fails.
- Expanded the PostgreSQL proof migration with native PostgreSQL tables for:
  - `media_source_probes`;
  - `media_streams`;
  - `scan_snapshots`;
  - `directory_snapshots`;
  - `source_states`;
  - `search_documents`;
  - `local_inference_evidence`;
  - `ingestion_failures`.
- Implemented PostgreSQL parity slices for:
  - `ScanRepository`;
  - `LocalInferenceRepository`;
  - `IngestionFailureRepository`;
  - `MediaProbeRepository`;
  - `SearchIndex`.

Validation:

```bash
cargo fmt --all
cargo check -p taru-db --tests
cargo check -p taru-library --tests
cargo nextest run -p taru-db scan_commit --no-fail-fast
cargo nextest run -p taru-db scan_commit --run-ignored ignored-only --no-fail-fast
cargo nextest run -p taru-db contract --no-fail-fast
cargo nextest run -p taru-db contract --run-ignored ignored-only --no-fail-fast
git diff --check
```

Result:

- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-library --tests` passed.
- Focused SQLite/default scan commit contract run passed: 2 passed, 78
  skipped.
- Focused PostgreSQL ignored scan commit contract run passed: 2 passed, 78
  skipped.
- SQLite/default contract run passed: 9 passed, 71 skipped.
- PostgreSQL ignored contract run passed: 8 passed, 72 skipped.
- `git diff --check` passed with Git CRLF normalization warnings only.

Review notes:

- The scan commit success path intentionally keeps Media Technical Facts as a
  separate `MediaProbeRepository` operation because the current domain commit
  object does not embed probe results. The contract still proves PostgreSQL
  parity for the technical-facts repository required by the scan workflow.
- This slice does not claim metadata refresh, NFO import, provider mapping,
  Catalog Item Graph, user playback, transcode, webhooks, Addons, automation,
  Managed Artwork, VFS, or staging PostgreSQL parity. Those remain tracked by
  PGR-060 and later tasks.

### 2026-05-20 — PGR-060 Metadata/Catalog Commit PostgreSQL Parity

Status: complete.

Implementation evidence:

- Added the `MetadataCatalog` contract family in
  `crates/taru-db/src/contract_tests.rs`.
- Added a metadata refresh contract proving:
  - Media Item canonical metadata update;
  - provider raw response write/list behavior;
  - Provider Subject upsert/find/list behavior;
  - Provider Mapping creation and confidence/source persistence;
  - Library Item State confirmation for provisional items;
  - Metadata Provider Attempt insert/list behavior by job and by item filter.
- Added an NFO import contract proving:
  - Media Item and Metadata Field Lock writes;
  - full Catalog Item Graph replacement for People, Item Credits, Genres,
    Tags, Franchise Collections, Studios, and Image Assets;
  - Search Projection side effects through `SearchIndex::search`;
  - transactional rollback when the graph/search projection references a
    missing item.
- Expanded the PostgreSQL proof migration with native PostgreSQL tables for:
  - `metadata_field_locks`;
  - `provider_raw_responses`;
  - `provider_subjects`;
  - `provider_mappings`;
  - `metadata_provider_attempts`;
  - `people` and `person_external_ids`;
  - `item_credits`;
  - `genres` and `item_genres`;
  - `tags` and `item_tags`;
  - `collections`, `collection_external_ids`, and `collection_items`;
  - `studios`, `studio_external_ids`, and `item_studios`;
  - `image_assets`.
- Implemented PostgreSQL parity slices for:
  - `MetadataRepository`;
  - `ProviderMappingRepository`;
  - `CatalogRepository`;
  - Catalog Item Graph transactional replacement helpers.

Validation:

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check -p taru-db --tests
cargo check -p taru-metadata --tests
cargo check -p taru-catalog --tests
cargo nextest run -p taru-db metadata_catalog --no-fail-fast
cargo nextest run -p taru-db metadata_catalog --run-ignored ignored-only --no-fail-fast
cargo nextest run -p taru-db contract --no-fail-fast
cargo nextest run -p taru-db contract --run-ignored ignored-only --no-fail-fast
git diff --check
```

Result:

- `cargo fmt --all` passed.
- `cargo fmt --all -- --check` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-catalog --tests` passed.
- Focused SQLite/default metadata catalog contract run passed: 2 passed, 82
  skipped.
- Focused PostgreSQL ignored metadata catalog contract run passed: 2 passed,
  82 skipped.
- SQLite/default contract run passed: 11 passed, 73 skipped.
- PostgreSQL ignored contract run passed: 10 passed, 74 skipped.
- `git diff --check` passed with Git CRLF normalization warnings only.

Review notes:

- Provider-native payloads were initially stored in PostgreSQL JSONB here, but
  PGR-120 changed provider raw response bodies to text to preserve byte-for-byte
  raw payload round trips required by the repository contract.
- The PostgreSQL proof schema uses native `uuid`, `jsonb`, `boolean`, and
  `bigint` types instead of copying SQLite text/integer encodings.
- Managed Artwork download/cache lifecycle is still not claimed here; PGR-060
  only covers Image Assets that belong to Catalog Item Graph projection. The
  Managed Artwork runtime decision remains tracked by PGR-090.

### 2026-05-20 — PGR-070 Playback Runtime State PostgreSQL Parity

Status: complete.

Implementation evidence:

- Added the `PlaybackRuntime` contract family in
  `crates/taru-db/src/contract_tests.rs`.
- Added a User Playback State contract proving:
  - playback progress is scoped by `UserPrincipalId` and `MediaItemId`;
  - the same Media Item can have separate state per principal;
  - state upsert increments version;
  - Continue Watching excludes watched items, zero/no progress items, and other
    principals;
  - Continue Watching ordering follows latest play time.
- Added a Transcode Session lifecycle contract proving:
  - session create/get and active lookup behavior;
  - running transition records `started_at`;
  - cancellation request only affects active sessions and records cancelled
    failure category;
  - terminal cancellation records `completed_at` and removes active lookup;
  - latest-session lookup still returns terminal sessions;
  - filtered listing by source/kind/state;
  - stale active session recovery fails only active sessions.
- Expanded the PostgreSQL proof migration with native PostgreSQL tables for:
  - `user_playback_states`;
  - `transcode_sessions`;
  - the partial unique active-request index used to prevent duplicate active
    playback sessions for the same source/kind/request key.
- Implemented PostgreSQL parity slices for:
  - `UserPlaybackStateRepository`;
  - `TranscodeSessionRepository`.

Validation:

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check -p taru-db --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-db playback_runtime --no-fail-fast
cargo nextest run -p taru-db playback_runtime --run-ignored ignored-only --no-fail-fast
cargo nextest run -p taru-db contract --no-fail-fast
cargo nextest run -p taru-db contract --run-ignored ignored-only --no-fail-fast
git diff --check
```

Result:

- `cargo fmt --all` passed.
- `cargo fmt --all -- --check` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-server --tests` passed.
- Focused SQLite/default playback runtime contract run passed: 2 passed, 86
  skipped.
- Focused PostgreSQL ignored playback runtime contract run passed: 2 passed,
  86 skipped.
- SQLite/default contract run passed: 13 passed, 75 skipped.
- PostgreSQL ignored contract run passed: 12 passed, 76 skipped.
- `git diff --check` passed with Git CRLF normalization warnings only.

Review notes:

- User Playback State remains principal-scoped; no global watched/progress
  state was introduced.
- PostgreSQL persists transcode output paths because the server-owned playback
  runtime needs them for reuse/cancellation/cleanup. Public/admin DTO redaction
  remains owned by `taru-api`/HTTP boundaries and was not weakened.

### 2026-05-20 — PGR-080 Event, Addon, Webhook, And Automation PostgreSQL Parity

Status: complete.

Implementation evidence:

- Added the `EventAddonAutomation` contract family in
  `crates/taru-db/src/contract_tests.rs`.
- Added an Event Outbox + Webhook delivery contract proving:
  - outbox event enqueue is idempotent by `(DomainEventKind, idempotency_key)`;
  - payload, subject, library, and Source Locator-scoped source identity
    round-trip through repository APIs;
  - outbox list filtering works by kind/status/library/source;
  - only enabled webhook endpoints are listed;
  - webhook delivery attempts are created as pending, then record failed
    delivery result, HTTP status, safe error, completion time, and retry time.
- Added an Addon contract proving:
  - Addon registration upsert/find/list behavior;
  - Addon Token creation, hash lookup, last-used update, rotation, and
    revocation;
  - Addon grants replace atomically by addon;
  - Addon Side Effects are idempotent by `(addon_id, idempotency_key)`;
  - side-effect provenance/payload stay repository-internal fields and apply
    outcome records applied item/source/report without exposing raw token data.
- Added an Automation contract proving:
  - enabled/disabled Automation Provider filtering;
  - capability and secret-env metadata round-trip through repository APIs;
  - Automation Artifact creation starts proposed;
  - accepting an artifact records `accepted_at`;
  - rejecting an artifact clears `accepted_at`;
  - artifact listing by job and by item returns the expected records.
- Expanded the PostgreSQL proof migration with native PostgreSQL tables for:
  - `event_outbox`;
  - `webhook_endpoints`;
  - `webhook_delivery_attempts`;
  - `automation_providers`;
  - `automation_artifacts`;
  - `addon_registrations`;
  - `addon_tokens`;
  - `addon_grants`;
  - `addon_side_effects`.
- Implemented PostgreSQL parity slices for:
  - `EventOutboxRepository`;
  - `WebhookRepository`;
  - `AddonRepository`;
  - `AutomationRepository`.

Validation:

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check -p taru-db --tests
cargo check -p taru-events --tests
cargo check -p taru-automation --tests
cargo nextest run -p taru-db event_addon_automation --no-fail-fast
cargo nextest run -p taru-db event_addon_automation --run-ignored ignored-only --no-fail-fast
cargo nextest run -p taru-events --no-fail-fast
cargo nextest run -p taru-automation --no-fail-fast
cargo nextest run -p taru-db contract --no-fail-fast
cargo nextest run -p taru-db contract --run-ignored ignored-only --no-fail-fast
git diff --check
```

Result:

- `cargo fmt --all` passed.
- `cargo fmt --all -- --check` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-events --tests` passed.
- `cargo check -p taru-automation --tests` passed.
- Focused SQLite/default event/addon/automation contract run passed: 3
  passed, 91 skipped.
- Focused PostgreSQL ignored event/addon/automation contract run passed: 3
  passed, 91 skipped.
- `taru-events` nextest passed: 3 passed, 0 skipped.
- `taru-automation` nextest passed: 3 passed, 0 skipped.
- SQLite/default contract run passed: 16 passed, 78 skipped.
- PostgreSQL ignored contract run passed: 15 passed, 79 skipped.
- `git diff --check` passed with Git CRLF normalization warnings only.

Review notes:

- Addon Tokens keep only prefixes/hashes in repository records; raw token
  values are not persisted or introduced into PostgreSQL schema.
- Webhook and Automation credentials remain environment-variable references
  (`secret_env`), not secret values.
- Event payloads and Addon Side Effect payload/provenance are still persisted
  for durable delivery/apply behavior, but the contract exercises them only
  through repository records that already mark sensitive side-effect fields as
  skipped for serialization.
- Post-close AAD-090 update: Addon Side Effects now also require
  `request_fingerprint` in the clean PostgreSQL proof schema, matching the
  SQLite base schema. AAD-090 did not rerun PostgreSQL opt-in contracts because
  `TARU_TEST_POSTGRES_URL` was not set in that session.

### 2026-05-20 — PGR-090 Managed Artwork PostgreSQL Parity Split

Status: complete.

Decision:

- Managed Artwork PostgreSQL parity is split out of M62 into
  `docs/workstreams/managed-artwork-postgresql-parity/`.
- M62 may close without implementing Managed Artwork PostgreSQL parity only if
  PostgreSQL runtime diagnostics/selection remain truthful and do not partially
  enable Managed Artwork routes or workers.

Rationale:

- `ManagedArtworkRepository` is not a narrow tail of the previous contract
  matrix. It spans:
  - Addon Artwork Candidate intake;
  - candidate acceptance and durable Managed Artwork Ingest records;
  - ingest job ownership/claim/commit/fail/requeue;
  - Managed Artwork Artifact records and artifact-store file authority;
  - Selected Artwork publication/unpublication;
  - Admin gallery snapshots;
  - lifecycle cleanup and retention protection;
  - drift diagnostics and remediation policy;
  - thumbnail variants and redaction-sensitive public/Admin image serving.
- `PostgresStore` has no `ArtworkCandidateRepository` or
  `ManagedArtworkRepository` implementation yet.
- Forcing all of this into PGR-090 would either make M62 too broad or tempt a
  partial implementation that violates the workstream rule: do not claim
  PostgreSQL production readiness for runtime surfaces that still assume
  SQLite-only state.

Evidence:

- Added follow-on workstream docs:
  - `docs/workstreams/managed-artwork-postgresql-parity/README.md`;
  - `docs/workstreams/managed-artwork-postgresql-parity/DESIGN.md`;
  - `docs/workstreams/managed-artwork-postgresql-parity/TODO.md`;
  - `docs/workstreams/managed-artwork-postgresql-parity/MILESTONES.md`;
  - `docs/workstreams/managed-artwork-postgresql-parity/EVIDENCE_AND_GATES.md`;
  - `docs/workstreams/managed-artwork-postgresql-parity/HANDOFF.md`;
  - `docs/workstreams/managed-artwork-postgresql-parity/WORKSTREAM.json`.
- Updated this workstream's contract matrix and milestone notes to mark Managed
  Artwork as a named follow-on instead of an implicit gap.

Validation:

```bash
rg "impl ManagedArtworkRepository for PostgresStore|impl ArtworkCandidateRepository for PostgresStore|impl ArtworkTaskRepository for PostgresStore" crates/taru-db/src/postgres.rs -n
rg "Managed Artwork|PGR-090|managed-artwork-postgresql-parity" docs/workstreams/postgresql-production-readiness docs/workstreams/managed-artwork-postgresql-parity -n
git diff --check
```

Result:

- PostgreSQL Managed Artwork repository implementations are absent, confirming
  this is a real parity gap rather than an already-completed slice.
- Documentation now records the split target and the PostgreSQL runtime gating
  requirement.
- `git diff --check` passed with Git CRLF normalization warnings only.

### 2026-05-20 — PGR-100 Safe Database Backend Diagnostics

Status: complete.

Implementation evidence:

- Added `AdminDatabaseConfigDiagnostics` and
  `AdminDatabaseBackendCapabilitiesDiagnostics` to the Admin system config DTO.
- Updated `GET /admin/v1/system/config` to return a sanitized database block:
  - configured backend kind;
  - active backend kind;
  - database URL scheme only;
  - runtime support flag;
  - startup migration flag;
  - active backend capability booleans.
- Added a read-only `TaruApp::store()` accessor and retained the active
  `TaruDatabase` in app composition so HTTP diagnostics can report actual
  backend identity/capabilities without depending on concrete adapters.
- Added `ServerStartupReport.database_migrated` so diagnostics report migration
  state from startup workflow completion rather than raw database errors.
- Updated the Admin TypeScript contract and generated Admin Web contract.
- Extended the system config route test to assert the database diagnostics
  fields and to keep `database_url`, database path fragments, credentials, and
  raw path-like details out of the response body.
- Updated HTTP API documentation to describe explicit SQLite/PostgreSQL
  backend-selection diagnostics and the redaction boundary.

Validation:

```bash
cargo fmt --all
cargo check -p taru-api --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-server admin_v1_system_config_reports_sanitized_configuration --no-fail-fast
cargo nextest run -p taru-api admin_contract --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result:

- `cargo fmt --all` passed.
- `cargo check -p taru-api --tests` passed.
- `cargo check -p taru-server --tests` passed.
- Focused Admin system config route test passed: 1 passed, 173 skipped.
- Admin contract nextest passed: 4 passed, 37 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Review notes:

- The route exposes only `url_scheme`; it does not expose `database_url`,
  credentials, host names, database names, filesystem paths, query strings, or
  raw database errors.
- `runtime_supported` is derived from configured backend kind matching the
  active `TaruDatabase` backend. This keeps test/injected-store diagnostics
  honest and avoids claiming PostgreSQL runtime support before the M62 closeout
  gate allows it.

### 2026-05-20 — PGR-110 SQLite Assumption Deletion Sweep

Status: complete.

Implementation evidence:

- Deleted `TaruDatabase::connect(&str)`, which silently selected SQLite and
  made production callers look URL-driven.
- Deleted `TaruDatabase::connect_with_sqlite_runtime(...)`, replacing the
  facade-level API shape with explicit `DatabaseConnectOptions` for production
  construction.
- Kept `TaruDatabase::connect_in_memory()` as a deliberately named test fixture
  convenience.
- Removed the remaining `taru-db` facade behavior-test imports of
  `sqlite::codec::{provider_to_parts,row_get}`.
- Removed direct `store.sqlite().pool()` SQL inspection from the startup job
  recovery test. That test now verifies through repository-visible behavior
  instead of reading SQLite columns.

Inventory:

```bash
rg -n "taru_db::sqlite|crate::sqlite|SqliteStore|sqlite::codec|store\\.sqlite\\(\\)" crates/taru-server/src crates/taru-api/src crates/taru-db/src/facade.rs crates/taru-db/src/tests.rs crates/taru-db/src/contract_tests.rs -g "*.rs"
rg -n "TaruDatabase::connect\\(|connect_with_sqlite_runtime|DatabaseConnectOptions::sqlite\\(" crates/taru-server/src crates/taru-db/src -g "*.rs"
rg -n "strftime|PRAGMA|rowid|json_extract|SqliteRow|INSERT OR" crates/taru-db/src crates/taru-server/src -g "*.rs" -g "!crates/taru-db/src/sqlite/**" -g "!crates/taru-db/src/postgres.rs"
rg -n "sqlite::memory:|sqlite://" crates/taru-server/src crates/taru-api/src -g "*.rs"
```

Result:

- No `taru_db::sqlite`, `sqlite::codec`, or `store.sqlite()` references remain
  in server/api code or facade-level tests.
- `SqliteStore` references outside `taru-db::sqlite` are confined to
  `TaruDatabase`'s internal facade dispatch.
- No SQLite SQL dialect terms (`strftime`, `PRAGMA`, `rowid`, `json_extract`,
  `SqliteRow`, `INSERT OR`) remain above the SQLite adapter, excluding
  PostgreSQL's own adapter file.
- Remaining `DatabaseConnectOptions::sqlite(...)` occurrence is the explicit
  backend-selection unit test.
- Remaining `sqlite::memory:` and `sqlite://taru.db` occurrences in
  `taru-server` are test fixture/default SQLite configuration values, not
  production backend-selection logic.

Validation:

```bash
cargo fmt --all
cargo check -p taru-db --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-db taru_database_sqlite_marks_running_jobs_failed_on_startup_and_preserves_queued_jobs --no-fail-fast
cargo nextest run -p taru-db taru_database_connect_options_select_sqlite_explicitly taru_database_rejects_postgres_until_contract_parity_is_complete --no-fail-fast
cargo nextest run -p taru-db taru_database_sqlite_round_trips_media_items_and_sources --no-fail-fast
cargo nextest run -p taru-db contract --no-fail-fast
cargo fmt --all -- --check
cargo check --workspace --tests
git diff --check
```

Result:

- `cargo fmt --all` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-server --tests` passed.
- Startup recovery focused nextest passed: 1 passed, 93 skipped.
- Backend-selection focused nextest passed: 2 passed, 92 skipped.
- Media item/source focused nextest passed: 1 passed, 93 skipped.
- SQLite/default contract run passed: 16 passed, 78 skipped.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Review notes:

- This task does not enable PostgreSQL production runtime by itself. It removes
  adapter-seam leaks so PGR-120 can honestly close or split the remaining
  runtime gating decisions.
- Managed Artwork PostgreSQL parity remains a named follow-on and is still not
  partially enabled.

### 2026-05-20 — PGR-120 Runtime Promotion And VFS/Staging PostgreSQL Parity

Status: complete.

Implementation evidence:

- Promoted `PostgresStore` out of test-only compilation for the supported
  backend scope.
- Changed `TaruDatabase` from a SQLite field plus SQLite-only dispatch into an
  internal backend adapter trait object that can dispatch every repository
  trait to SQLite or PostgreSQL.
- Changed PostgreSQL runtime selection from a static unsupported error to a
  real PostgreSQL connection path through `DatabaseConnectOptions::postgres`.
- Kept PostgreSQL capability reporting explicit:
  - lifecycle, libraries, jobs, job leases, media, scan commits, metadata,
    catalog, playback state, transcode sessions, event outbox, Addons,
    Automation, VFS cache/staging, webhooks, and search index are supported for
    the M62 scope;
  - Managed Artwork remains unsupported and split to
    `docs/workstreams/managed-artwork-postgresql-parity/`.
- Added PostgreSQL runtime gap contracts and parity for surfaces that blocked
  default server startup:
  - `VfsCacheRepository`;
  - `StagingManifestRepository`.
- Expanded `crates/taru-db/migrations/postgres/0001_contract_jobs.sql` with
  PostgreSQL-owned tables and indexes for:
  - `vfs_cache_objects`;
  - `vfs_cache_listings`;
  - `vfs_cache_listing_entries`;
  - `vfs_cache_failures`;
  - `staging_manifest_records`.
- Added the backend-neutral `VfsStaging` contract family proving:
  - VFS cached object/listing round trip;
  - listing replacement semantics;
  - VFS cache failure upsert/count behavior;
  - VFS cache summary counts and stale listing/object calculations;
  - staging reservation path conflict behavior;
  - staging disk budget accounting;
  - staging start/complete/fail/delete lifecycle behavior;
  - staging lease acquire/release and active-lease expiry protection;
  - cleanup candidate ordering and filtered listing behavior.
- Removed the PostgreSQL startup gate for staging cleanup because
  `capabilities.vfs_cache` is now true. Managed Artwork worker gating remains.

Validation:

```bash
cargo check -p taru-db --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-db vfs_staging taru_database_postgres_runtime_capabilities_name_supported_and_split_surfaces --no-fail-fast
cargo nextest run -p taru-db postgres_vfs_staging --run-ignored ignored-only --no-fail-fast
TARU_TEST_POSTGRES_URL=<local-test-url> cargo nextest run -p taru-db postgres_vfs_staging --run-ignored ignored-only --no-fail-fast
cargo nextest run -p taru-db contract --no-fail-fast
TARU_TEST_POSTGRES_URL=<local-test-url> cargo nextest run -p taru-db contract --run-ignored ignored-only --no-fail-fast
cargo nextest run -p taru-server system_config --no-fail-fast
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

Result:

- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-server --tests` passed.
- Focused SQLite/default VFS/Staging + capability nextest passed: 3 passed, 98
  skipped.
- Focused PostgreSQL ignored VFS/Staging nextest first failed when
  `TARU_TEST_POSTGRES_URL` was unset, proving the harness no longer reports
  false green without a real PostgreSQL test URL.
- Focused PostgreSQL ignored VFS/Staging nextest passed with
  `TARU_TEST_POSTGRES_URL` set to the local test database: 2 passed, 99
  skipped.
- `cargo fmt --all` passed.
- `cargo fmt --all -- --check` passed.
- SQLite/default contract run passed: 18 passed, 83 skipped.
- PostgreSQL opt-in full contract run passed against a local test PostgreSQL
  URL: 18 passed, 83 skipped.
- Focused Admin system-config nextest passed: 2 passed, 173 skipped.
- `cargo check --workspace --tests` passed.
- `cargo nextest run --workspace --no-fail-fast` passed: 488 passed, 18
  skipped. Nextest reported 4 leaky tests in existing Managed Artwork HTTP
  coverage; the command still exited successfully.
- `git diff --check` passed with Git CRLF normalization warnings only.

Review notes:

- PGR-120 closes M62 for the supported PostgreSQL backend scope.
- Managed Artwork PostgreSQL parity remains intentionally split and must not be
  partially enabled under PostgreSQL until the follow-on workstream proves it.
