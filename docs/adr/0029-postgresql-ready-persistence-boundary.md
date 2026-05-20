# 0029: Use a PostgreSQL-Ready Persistence Boundary

## Status

Accepted.

## Context

Taru currently uses SQLite through `taru-db::SqliteStore`. That shape was
appropriate for the MVP, but the project now needs a persistence architecture
that can support PostgreSQL later without carrying SQLite assumptions through
server, metadata, catalog, playback, Addon, and admin code.

Current friction:

- `SqliteStore` implements nearly every repository trait.
- SQL row codecs, migrations, transaction helpers, and repository adapters are
  concentrated in one concrete crate shape.
- Server app services frequently hold `SqliteStore` directly.
- The original migration/lifecycle trait was named `TransactionManager`, but it
  only exposed migration behavior.
- Tests mostly prove SQLite behavior by constructing `SqliteStore` directly,
  rather than proving backend-neutral persistence contracts.

Taru is not live and has no stable production compatibility burden, so the
correct move is to reshape the persistence seam now instead of preserving the
MVP adapter shape.

## Decision

Taru will move toward a PostgreSQL-ready persistence boundary:

- Domain records, IDs, and repository contracts remain in `taru-core` unless a
  later ADR splits a dedicated persistence-contract crate.
- `taru-db` becomes the database facade used by `taru-server` and tests. The
  facade owns backend selection, database lifecycle, migration entrypoints, and
  re-exported test helpers.
- SQLite implementation details move behind a SQLite-owned adapter module or
  implementation crate. The clean long-term target is a small facade plus
  backend-specific adapters:
  - `taru-db` as facade;
  - `taru-sqlite` for SQLite SQLx implementation;
  - future `taru-postgres` for PostgreSQL SQLx implementation.
- During migration, an internal `taru-db::sqlite` module split may happen first
  to reduce risk, but production services should depend on the facade handle,
  not on `SqliteStore`.
- Repository behavior must be covered by backend-neutral contract tests. SQLite
  is the first adapter required to pass those contracts; PostgreSQL must pass
  the same contract suite when added.
- Multi-record workflow writes should be expressed as workflow-shaped commit or
  unit-of-work methods. Callers should not learn SQL write ordering.
- Migration/lifecycle naming must remain honest. The public trait for database
  startup lifecycle is `DatabaseLifecycle`; it currently exposes migration
  behavior and must not grow transaction APIs by accident.
- SQL dialect differences stay in backend adapters: time expressions, JSON
  storage, lock/lease acquisition, `ON CONFLICT` behavior, row codecs, and
  migration files must not leak into application services.

## Consequences

- PostgreSQL can be added as an adapter instead of as a rewrite of every app
  service.
- The first implementation work is mostly architectural: contract tests, facade
  handle, SQLite module split, and deletion of direct `SqliteStore`
  dependencies from server services.
- Some crate/module churn is expected and accepted because Taru is
  pre-compatibility.
- Contract tests become the authority for persistence behavior, while
  backend-specific tests remain useful for migrations and dialect edge cases.
- The facade may temporarily delegate many repository traits to SQLite during
  the transition. This is acceptable only as a migration step with a deletion
  gate; SQL and transaction logic must live in backend-specific adapters.

## Alternatives Considered

- Keep `SqliteStore` as the permanent concrete store and add PostgreSQL later
  by copying it. Rejected because it preserves the god-adapter shape and would
  duplicate persistence logic without contract tests.
- Move all repository traits from `taru-core` into a new crate immediately.
  Deferred because current repository contracts are already established in
  `taru-core`, and a facade-plus-adapter split gives most of the PostgreSQL
  readiness benefit with less immediate churn.
- Introduce an ORM. Rejected because Taru relies on explicit workflow commits,
  durable jobs, leases, redaction-sensitive read models, and SQL-specific
  behavior that should remain visible in adapter code.
- Implement full PostgreSQL now. Deferred until the contract suite and facade
  shape are in place; otherwise PostgreSQL work would hard-code today's seams.

## Related Workstreams

- [0030: Define PostgreSQL-Ready SQL Dialect And Migration Policy](0030-postgresql-ready-sql-dialect-and-migration-policy.md)
- `docs/workstreams/future-ready-architecture-refactor/`
- `docs/workstreams/core-architecture-deepening/`
- `docs/workstreams/repository-seam-deepening/`
- `docs/workstreams/runtime-foundation/`
