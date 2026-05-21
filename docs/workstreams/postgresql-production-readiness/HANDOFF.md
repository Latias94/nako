# PostgreSQL Production Readiness — Handoff

Status: Completed
Last updated: 2026-05-20

## Current State

M62 is closed. M61 left a clean PostgreSQL-ready baseline, and PGR-020 through
PGR-120 have extended that baseline into the supported PostgreSQL runtime
backend scope:

- `TaruDatabase` is the public database facade.
- SQLite implementation details live under `crates/taru-db/src/sqlite/`.
- `PostgresStore` is compiled in runtime code for the supported PostgreSQL
  backend scope.
- PostgreSQL migration coverage for this lane lives in
  `crates/taru-db/migrations/postgres/0001_contract_jobs.sql`.
- `contract_tests.rs` contains SQLite always-on backend-neutral contract
  families and PostgreSQL ignored opt-in variants gated by
  `TARU_TEST_POSTGRES_URL`; ignored PostgreSQL gates now fail fast when the URL
  is missing instead of reporting false green.
- `DatabaseBackendKind` and `DatabaseConnectOptions` now define explicit
  runtime backend selection.
- `TaruServerConfig.database_backend` defaults to SQLite, and `TaruApp::new`
  constructs `TaruDatabase` through backend options.
- `TaruDatabase` reports backend kind and capability metadata and internally
  dispatches through a backend adapter trait object instead of a SQLite-only
  field.
- PostgreSQL runtime selection now enters a real PostgreSQL connection path.
  Managed Artwork remains explicitly unsupported/split for PostgreSQL.
- The backend-neutral contract harness now uses explicit contract metadata and
  paired SQLite/PostgreSQL runners instead of per-backend copy-paste runner
  functions.
- Lifecycle and job-lease contract families run through the generalized
  harness. PostgreSQL ignored contract tests passed in this environment.
- Library/Media Source contracts now run through the harness and pass against
  SQLite plus opt-in PostgreSQL.
- PostgreSQL proof schema now includes media items, media item external IDs,
  media sources, and library item states.
- `PostgresStore` implements the `MediaRepository` and
  `LibraryItemRepository` slice needed by PGR-040.
- PostgreSQL proof schema now also includes media technical facts, scan
  snapshots, directory snapshots, source states, search documents, local
  inference evidence, and ingestion failures.
- `PostgresStore` implements the PGR-050 `ScanRepository`,
  `LocalInferenceRepository`, `IngestionFailureRepository`,
  `MediaProbeRepository`, and `SearchIndex` slices.
- PostgreSQL proof schema now includes metadata field locks, provider raw
  responses, Provider Subjects, Provider Mappings, metadata provider attempts,
  and Catalog Item Graph tables for people, credits, genres, tags, Franchise
  Collections, studios, and Image Assets.
- `PostgresStore` implements the PGR-060 `MetadataRepository`,
  `ProviderMappingRepository`, and full `CatalogRepository` parity needed by
  metadata refresh and NFO import contract commits.
- PostgreSQL proof schema now includes runtime state for User Playback State,
  Transcode Sessions, Event Outbox, Webhooks, Addons, and Automation.
- PostgreSQL proof schema now includes VFS Cache and Staging Manifest state.
- `PostgresStore` implements `VfsCacheRepository` and
  `StagingManifestRepository`; PostgreSQL `vfs_cache` capability is true.
- Managed Artwork PostgreSQL parity was intentionally split to
  `docs/workstreams/managed-artwork-postgresql-parity/`; that follow-on later
  proved support and enabled the PostgreSQL Managed Artwork runtime capability.
- Admin system config diagnostics now include a sanitized database block for
  configured backend kind, active backend kind, URL scheme, startup migration
  status, runtime support, and active backend capabilities without exposing
  `database_url`, credentials, host/path/query details, or raw database errors.
- Compatibility-style facade constructors that implied SQLite have been
  removed; production construction now uses `DatabaseConnectOptions`, while
  test-only in-memory construction remains available.
- Facade-level DB behavior tests no longer import SQLite row codecs or inspect
  the SQLite pool directly.

PGR-010 created the durable workstream and contract matrix. PGR-020 added the
first production-shaped backend selection seam without exposing concrete
adapters to server code. PGR-030 generalized the contract harness so new
families can be added as vertical slices. PGR-040 added the first core
Library/Media parity slice beyond jobs. PGR-050 added Library scan commit
contracts and PostgreSQL parity for Source State, Local Inference Evidence,
Media Technical Facts, ingestion failures, and Search Projection side effects.
PGR-060 added Metadata/Catalog commit contracts and PostgreSQL parity for
metadata refresh, provider mappings/raw responses/attempts, NFO import, full
Catalog Item Graph replacement, Search Projection, and rollback behavior.
PGR-070 added Playback Runtime contracts and PostgreSQL parity for
principal-scoped User Playback State, Continue Watching, Transcode Session
active/latest lookup, cancellation, filtered listing, terminal transitions, and
stale active session recovery.
PGR-080 added Event/Addons/Automation contracts and PostgreSQL parity for Event
Outbox idempotency/filtering, Webhook endpoints/delivery attempts, Addon
registration/token/grant/side-effect state, and Automation provider/artifact
state. A later Addon architecture pass (AAD-090, 2026-05-21) updated the clean
Addon Side Effect schema to require `request_fingerprint` in both SQLite and
the PostgreSQL proof schema; PostgreSQL opt-in contracts still require
`TARU_TEST_POSTGRES_URL`. PGR-090 split Managed Artwork PostgreSQL parity out
of M62 into a named follow-on instead of enabling a large SQLite-only runtime
surface partially.
PGR-100 added safe database backend diagnostics and updated API/Admin contract
docs for explicit SQLite/PostgreSQL backend selection. PGR-110 deleted the
remaining facade-level SQLite convenience constructors and isolated direct
SQLite row/pool assumptions back under SQLite-owned code/tests. PGR-120 found
that closing with PostgreSQL runtime still gated by VFS/Staging would be too
weak, so it added backend-neutral `VfsStaging` contracts, PostgreSQL
VFS/Staging migration parity, repository parity, true PostgreSQL runtime
connection selection, and backend adapter dispatch in the facade.

## Closeout

- Task ID: PGR-120 completed.
- M62 is closed for the supported PostgreSQL backend scope.
- Final validation included formatting, workspace checks, default SQLite
  contracts, PostgreSQL opt-in full contracts against a local test PostgreSQL
  URL, workspace nextest, and `git diff --check`.
- Remaining PostgreSQL work is not hidden inside M62: Managed Artwork parity is
  split to `docs/workstreams/managed-artwork-postgresql-parity/`.

## Decisions So Far

- M62 should not be a broad refactor repeat of M61. It should prove backend
  production readiness through concrete contracts, migrations, runtime
  selection, and verification.
- SQLite remains the default backend and always-on test backend.
- PostgreSQL contract tests remain opt-in because they require an external test
  database URL, but the opt-in gate now fails fast when
  `TARU_TEST_POSTGRES_URL` is absent.
- Backend selection should be explicit; URL guessing may be a convenience but
  must not be the only behavior source.
- PGR-020 deliberately does not fake production PostgreSQL runtime support.
  The config can select PostgreSQL, but the facade rejects it until repository
  contracts and migrations define the supported scope.
  Superseded by PGR-120 for the supported scope: the facade now connects to
  PostgreSQL at runtime and reports the supported capability set truthfully.
- PGR-030 proved the new harness with both default SQLite and opt-in
  PostgreSQL contract runs.
- PGR-040 preserved library-scoped Source Locator identity and used native
  PostgreSQL `uuid`, `jsonb`, and `boolean` types rather than copying SQLite's
  text/integer storage choices.
- PGR-060 initially used native JSONB for provider raw response storage, but
  PGR-120 changed provider raw response bodies back to text under PostgreSQL to
  preserve the repository contract's byte-for-byte raw provider payload
  semantics. Provider-owned structured payloads still stay behind repository
  APIs and do not leak into PostgreSQL-specific catalog schema choices.
- PGR-060 covers Catalog Image Assets but does not claim Managed Artwork
  download/cache lifecycle; that remains tracked by PGR-090.
- PGR-070 keeps User Playback State principal-scoped and keeps transcode output
  paths as server-owned persistence state. Public/admin redaction remains at
  DTO/HTTP boundaries and was not weakened.
- PGR-090 split Managed Artwork PostgreSQL parity because the subsystem spans
  candidate intake, ingest jobs, artifacts, Selected Artwork, galleries,
  lifecycle cleanup, drift/remediation diagnostics, thumbnails, artifact-store
  files, and redaction-sensitive public/Admin serving. Partial support under
  PostgreSQL is forbidden.
- PGR-100 database diagnostics deliberately expose only backend identity,
  scheme, migration status, support status, and coarse capability booleans. The
  raw database URL, credentials, host, path, query, and backend errors remain
  outside Admin DTOs.
- PGR-110 kept `TaruDatabase::connect_in_memory()` because it is explicitly a
  test fixture convenience, but deleted implicit SQLite production constructors
  from the facade. Remaining `sqlite::memory:` occurrences above adapters are
  test fixture data or default SQLite config examples, not production selection
  logic.
- PGR-120 treats missing `TARU_TEST_POSTGRES_URL` as a failed PostgreSQL
  ignored contract gate, not a skipped success. This prevents false-positive
  opt-in gate evidence.
- PGR-120 promotes VFS Cache/Staging Manifest under PostgreSQL because default
  server startup can run staging cleanup; leaving this disabled would make the
  PostgreSQL runtime shape misleading.

## Blockers

- None for M62 closeout.

## Next Recommended Action

1. Commit M62 after user confirmation.
2. If continuing PostgreSQL work, open or activate
   `managed-artwork-postgresql-parity`.
3. Optionally open a separate production hardening/CI lane for containerized
   PostgreSQL setup, performance indexes, pool timeout tuning, and migration
   rollback drills.
