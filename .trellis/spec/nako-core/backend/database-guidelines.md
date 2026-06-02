# Database Guidelines

`nako-core` defines persistence contracts, not persistence implementation.

## Scope

- Repository traits live in `crates/nako-core/src/repository/*.rs`.
- Durable records, IDs, filters, and commit structs live in `nako-core`.
- SQLite/Postgres SQL, migrations, row mappers, and contract tests live in
  `nako-db`.

## Contract Rules

- Adding a repository method is a cross-crate contract change. Update:
  `nako-core` trait, SQLite adapter, Postgres adapter when implemented,
  contract tests, and any server app service that calls it.
- Use strong ID types instead of raw `String`/`Uuid` parameters in repository
  traits.
- Use `PageRequest` for list surfaces. Do not add unbounded list methods for
  Admin/Public/API scale surfaces.
- Persisted enums must expose parse helpers that return `NakoError::Database`
  for unknown stored values. This turns database drift into a visible adapter
  error.
- Store domain state in neutral terms from `CONTEXT.md`; do not encode provider
  payload ownership into core records unless the concept is explicitly a
  Provider Subject or Provider Mapping.

## Wrong vs Correct

### Wrong

```rust
async fn list_jobs_raw_sql(&self, sql: &str) -> Result<Vec<Job>>;
```

### Correct

```rust
async fn list_jobs(&self, filter: JobListFilter, page: PageRequest) -> Result<Vec<Job>>;
```

Repository traits describe domain queries. SQL shape stays in `nako-db`.

## Evidence

- `crates/nako-core/src/repository/jobs.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-core/src/repository/pagination.rs`
- `crates/nako-db/src/contract_tests.rs`
