# PostgreSQL Production Readiness Design

Status: Completed
Last updated: 2026-05-20

Closeout: M62 is complete for the supported PostgreSQL backend scope. Managed
Artwork PostgreSQL parity is intentionally excluded from this lane and tracked
as `docs/workstreams/managed-artwork-postgresql-parity/`.

## Why This Lane Exists

M61 closed the broad architecture refactor and created a PostgreSQL-ready
persistence seam:

- `taru-db` exposes a `TaruDatabase` facade;
- SQLite implementation details live under `taru-db::sqlite`;
- backend-neutral job lease contract tests run against SQLite;
- ignored PostgreSQL job lease contract tests can run when
  `TARU_TEST_POSTGRES_URL` is set;
- ADR 0029 and ADR 0030 define the persistence boundary, migration ownership,
  SQL dialect policy, row-codec policy, and test fixture policy.

At the start of M62, that was not yet production PostgreSQL support. The
PostgreSQL code was a proof harness for libraries, jobs, and job leases. The
server still constructed SQLite through `TaruDatabase::connect*`, and most
repository behavior was proven only by SQLite tests.

Closeout note: this paragraph documents the M61 starting point. PGR-120
promoted PostgreSQL into the runtime connection path and expanded the
backend-neutral contract matrix across the supported backend scope.

M62 exists to turn the proof into a production-ready backend shape without
reintroducing a god adapter, fake portability layer, or hidden SQLite
assumptions.

## Authority

- `CONTEXT.md`
- `docs/adr/0029-postgresql-ready-persistence-boundary.md`
- `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
- `docs/workstreams/future-ready-architecture-refactor/`
- `crates/taru-db/src/contract_tests.rs`
- `crates/taru-db/src/facade.rs`
- `crates/taru-db/src/postgres.rs`
- `crates/taru-db/src/sqlite/`
- `crates/taru-server/src/config.rs`
- `crates/taru-server/src/app/composition.rs`

## Problem

### P0 — Contract Coverage Is Too Narrow

The only backend-neutral PostgreSQL proof is the job lease lifecycle. It proves
locking, fencing, cancellation, recovery, basic libraries, and jobs, but not
the data paths that make Taru a media server:

- Media Library and Media Source identity;
- Media Item hierarchy and Library Item State;
- scan commit units and ingestion failures;
- metadata/provider mapping/catalog graph/search projection commits;
- user playback state;
- transcode session lifecycle;
- event outbox/webhook/addon/automation state;
- Managed Artwork state.

### P1 — Schema Parity Is Not Yet Designed As A Matrix

PostgreSQL currently has one migration for contract jobs. Production readiness
needs a clear table-by-table parity plan so future SQL work is not an
unreviewed translation of SQLite migrations.

### P2 — Runtime Backend Selection Does Not Exist

`TaruDatabase` is still structurally SQLite-backed in production. Server config
accepts a database URL but does not model backend kind, PostgreSQL pool/runtime
policy, connection validation, migration behavior, or safe diagnostics.

### P3 — SQLite Assumptions May Still Leak Above The Adapter

M61 isolated many SQLite modules, but M62 must actively remove assumptions from
facade/server/tests:

- `sqlite::memory:` as the only test/runtime construction path;
- SQLite row-codec helpers used by facade-level tests;
- text timestamp expectations that may not match PostgreSQL `timestamptz`;
- JSON text vs native `jsonb`;
- SQLite-specific conflict behavior and transaction ordering.

### P4 — PostgreSQL Verification Must Be Repeatable

PostgreSQL tests cannot rely on a developer remembering ad hoc commands. M62
needs documented opt-in commands and a future CI path that can run PostgreSQL
contract tests safely.

## Target State

When M62 closes:

- `TaruDatabase` can select SQLite or PostgreSQL through an explicit
  production configuration path.
- SQLite remains the default and always-on test backend.
- PostgreSQL has production-shaped lifecycle, migration, pool/runtime policy,
  and safe diagnostics.
- Backend-neutral contract tests cover the core repository/workflow seams that
  must behave identically across SQLite and PostgreSQL.
- PostgreSQL migrations cover all tables required by the production-ready
  backend scope, with intentional differences documented by ADR-backed policy.
- Server startup can migrate and run against PostgreSQL for the supported
  scope without importing SQLite adapter details.
- SQLite-specific assumptions either stay inside `taru-db::sqlite` or are
  named as temporary follow-ons with owners and expiry gates.
- Local and CI verification commands are documented.

## In Scope

- Backend-neutral contract-test harness expansion.
- PostgreSQL migration/schema parity planning and implementation.
- `TaruDatabase` backend selection and backend kind modeling.
- Server configuration for selecting SQLite or PostgreSQL.
- PostgreSQL lifecycle/migration/pool/runtime diagnostics needed for production
  operation.
- SQLite assumption audit and cleanup where it blocks backend selection.
- Documentation for local PostgreSQL test setup and optional CI gates.

## Out Of Scope

- Database sharding, read replicas, multi-tenant schemas, or online zero-downtime
  migration tooling.
- Replacing SQLite as the default local backend.
- Full query performance tuning before functional parity is proven.
- Non-database feature expansion such as Network Tunnel Provider, AI runtime,
  or new Admin UI pages except for database diagnostics needed by this lane.
- Copying schemas, migrations, tests, comments, or generated code from
  Jellyfin, Plex, or other reference projects.

## Starting State Inventory

| Area | Current state | M62 implication |
| --- | --- | --- |
| Facade | `TaruDatabase` owns only a SQLite field and delegates all repository traits to SQLite. | Introduce backend kind/enum or adapter dispatch without leaking concrete stores to callers. |
| SQLite | Implementation lives under `crates/taru-db/src/sqlite/`. | Keep SQLite as the reference always-on backend. |
| PostgreSQL | `PostgresStore` is `#[cfg(test)]`, supports lifecycle, libraries, jobs, and job leases. | Promote only when enough production contracts and migrations exist. |
| Contracts | `contract_tests.rs` covers job leases, with SQLite active and PostgreSQL ignored/opt-in. | Generalize harness by contract family and add vertical workflow contracts. |
| Server config | Uses `database_url` and many tests use `sqlite::memory:`. | Add explicit database backend selection and safe config diagnostics. |
| Migrations | SQLite has broad migrations; PostgreSQL has `0001_contract_jobs.sql`. | Create a parity matrix and implement by contract slice. |

Current PGR-120 audit update: the table above records the M61 starting point.
By PGR-120, `PostgresStore` has been promoted out of test-only compilation for
the supported backend scope, `TaruDatabase` dispatches through an internal
backend adapter trait, and PostgreSQL runtime selection enters the real
connection path. Managed Artwork remains the intentionally split runtime gap.

## Contract-Test Matrix

Each family must state whether it is required for M62 closeout, deferred, or
intentionally SQLite-only.

| Priority | Contract family | Required behavior | Current proof | M62 target |
| --- | --- | --- | --- | --- |
| P0 | Database lifecycle | connect, migrate, idempotent migrate, safe failure, backend identity | SQLite broad; PostgreSQL proof | Required |
| P0 | Library | upsert/get/list Media Library with roots/options/domain/preset | Job proof seeds Library | Required |
| P0 | Jobs and leases | enqueue/list/start/fail/recover, lease claim/heartbeat/fence/cancel/recover | SQLite + PostgreSQL job lease contracts | Required |
| P0 | Media Source and Media Item | Media Source identity scoped by Media Library; Media Item hierarchy; source links | SQLite tests only | Required |
| P0 | Scan commit unit | commit source state, Local Inference Evidence, Media Technical Facts, ingestion failures, search projection atomically | SQLite tests only | Required |
| P0 | Metadata/Catalog commit | provider mapping, raw response/attempt, Catalog Item Graph, Search Projection rollback behavior | SQLite tests only | Required or explicitly split if too broad |
| P1 | Search projection | semantic Browse Facets, aliases, provider IDs, exact facet matching | SQLite tests only | Required for production search parity or split to search-backend lane |
| P1 | User Playback State | principal-scoped progress/watched/continue watching | SQLite tests only | Required |
| P1 | Transcode sessions | session lifecycle, stale startup recovery, list/filter | SQLite tests only | Required |
| P1 | Event outbox and webhooks | idempotent event enqueue, list/filter, webhook attempts | SQLite tests only | Required |
| P2 | Addons/Automation | registration, tokens/grants, side effects, automation providers/artifacts | SQLite tests only | Required if Addon/Automation remain enabled with PostgreSQL |
| P2 | Managed Artwork | candidates, ingest, artifacts, selected artwork, lifecycle/remediation/gallery | SQLite tests only at M62 closeout; later covered by MAPG contracts | Split by PGR-090 to `docs/workstreams/managed-artwork-postgresql-parity/`; MAPG later landed PostgreSQL parity and runtime capability enablement |
| P3 | VFS cache/staging | cache records and staging manifest/lease behavior | SQLite + PostgreSQL `VfsStaging` contracts added in PGR-120 | Required; completed for supported runtime scope |

## Architecture Direction

### Backend Selection

Prefer an explicit backend selection interface over URL guessing:

- `DatabaseBackendKind::{Sqlite, Postgres}` in `taru-db` or server config;
- `TaruDatabase::connect(config)` or equivalent production constructor;
- `TaruDatabase` dispatches to the active adapter internally;
- server/app code continues to depend on repository traits and `TaruDatabase`,
  not `SqliteStore` or `PostgresStore`.

URL scheme inference may remain a convenience, but it must not be the only
source of truth for operational behavior.

### Contract Families Before SQL Copying

Every PostgreSQL expansion should start with a backend-neutral contract that
SQLite already passes. PostgreSQL then implements the minimum schema and SQL
needed to pass the same contract.

### Schema Parity

PostgreSQL migrations should be backend-owned and native:

- UUID columns use `uuid`;
- structured JSON uses `jsonb` when the adapter needs JSON behavior;
- booleans use `boolean`;
- timestamps use `timestamptz`;
- job lease claiming uses PostgreSQL locking semantics such as
  `FOR UPDATE SKIP LOCKED`;
- SQL clock functions stay adapter-owned.

Do not create one lowest-common-denominator SQL layer.

### Runtime Policy

Production PostgreSQL support needs explicit:

- pool size and timeout defaults;
- migration-on-startup behavior;
- connection validation and error redaction;
- config diagnostics that show backend kind but not credentials;
- optional test schema/database isolation.

## Closeout Condition

M62 can close only when:

- `TaruDatabase` supports explicit SQLite/PostgreSQL backend selection in
  production code;
- PostgreSQL is no longer only `#[cfg(test)]` proof code for the supported
  backend scope;
- required contract families either pass against SQLite and PostgreSQL, or are
  consciously split into named follow-on lanes with expiry gates;
- PostgreSQL migrations cover the supported production backend scope;
- server startup, migration, and safe diagnostics work for PostgreSQL;
- local/CI verification commands are documented;
- final validation gates pass with SQLite always-on and PostgreSQL opt-in gate
  evidence recorded.
