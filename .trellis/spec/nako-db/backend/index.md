# nako-db Backend Development Guidelines

These specs document the current persistence boundary in `crates/nako-db`.
SQLite is the default runtime target, PostgreSQL parity is preserved by adapter
shape and contract tests.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding adapter
  modules or repository implementations.
- Read [Database Guidelines](./database-guidelines.md) before adding schema,
  migrations, SQL, or row mapping.
- Read [Error Handling](./error-handling.md) before converting SQLx, migration,
  parse, or transaction errors.
- Read [Quality Guidelines](./quality-guidelines.md) before changing repository
  contracts or selecting validation gates.
- Read [Logging Guidelines](./logging-guidelines.md) before adding database
  diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | SQLite/Postgres adapter layout and contract-test ownership | Filled from code |
| [Database Guidelines](./database-guidelines.md) | SQLx, migrations, row mapping, durable job priority contract | Filled from code and ADRs |
| [Error Handling](./error-handling.md) | SQLx conversion, stored enum drift, transaction failures | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Contract tests, migration gates, SQLite/Postgres parity | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | Redaction-safe DB diagnostics | Filled as constrained boundary |

## Authority / Evidence

- ADR 0029: PostgreSQL-ready persistence boundary.
- ADR 0030: PostgreSQL-ready SQL dialect and migration policy.
- ADR 0053: bounded API scale and durable jobs.
- `crates/nako-core/src/repository/*.rs`
- `crates/nako-db/src/sqlite/*.rs`
- `crates/nako-db/src/postgres*.rs`
- `crates/nako-db/src/contract_tests.rs`
- `crates/nako-db/migrations/`
