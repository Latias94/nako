# Directory Structure

`nako-db` implements `nako-core` repository traits for database backends. It
does not own domain records, HTTP DTOs, app services, or provider runtimes.

## Current Layout

```text
crates/nako-db/src/
├── lib.rs                 # public database facade exports
├── backend.rs             # backend kind and connect options
├── facade.rs              # NakoDatabase wrapper and capabilities
├── sqlite.rs              # SQLite store root
├── sqlite/                # SQLite repository adapters and helpers
├── postgres.rs            # PostgreSQL store root, migrations, helpers
├── postgres/              # PostgreSQL repository adapters
├── contract_tests.rs      # backend-agnostic repository contracts
└── tests.rs               # focused database tests

crates/nako-db/migrations/
├── baseline.sql
├── 0002_*.sql
└── postgres/
    ├── baseline.sql
    └── 0002_*.sql
```

## Module Rules

- Add SQLite feature adapters under `src/sqlite/<feature>.rs`.
- Add PostgreSQL feature adapters under `src/postgres/<feature>.rs` when the
  contract is expected to support PostgreSQL parity.
- Keep shared SQLite conversion helpers in `src/sqlite/codec.rs`.
- Keep PostgreSQL root helpers near `src/postgres.rs` unless a focused module
  needs its own mapper.
- Keep database facade behavior in `facade.rs`; do not spread backend dispatch
  across feature modules.
- Keep backend-agnostic behavior in `contract_tests.rs` so SQLite and Postgres
  can run the same contract cases.

## Forbidden Placement

- Do not define new domain records here. Add them to `nako-core`.
- Do not return `sqlx` rows or adapter-private structs through repository
  traits.
- Do not add HTTP/Admin DTO shaping here. Use `nako-api` and `nako-server`.
- Do not make SQLite-only behavior the repository contract unless the task and
  ADR explicitly accept no PostgreSQL parity.

## Examples

- `sqlite/jobs.rs` and `postgres/jobs.rs`: same durable job contract across
  backend adapters.
- `sqlite/migrations.rs` and `postgres.rs`: versioned migration registration.
- `contract_tests.rs`: shared families for repository behavior.
