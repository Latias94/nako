# Error Handling

Database adapters translate SQLx, migration, row parsing, and stored-value drift
into `nako_core::NakoError`.

## Required Patterns

- Use adapter helper functions such as `database_error` for SQLx errors.
- Use core parse helpers for persisted enum values when they exist.
- Unknown stored values return `NakoError::Database`, not a default enum case.
- Targeted missing rows return `Ok(None)` from repository getters, then the app
  service decides whether that is `NotFound`.
- Transaction failures must abort the whole operation; do not partially commit
  multi-table domain updates.

## Validation Matrix

| Condition | Error / behavior |
|-----------|------------------|
| SQLx execution or decode failure | `NakoError::Database` via helper |
| Unknown stored enum string or score | `NakoError::Database` |
| Optional lookup missing | `Ok(None)` |
| Command update affects no row where conflict is meaningful | `NakoError::Conflict` or `Ok(None)` according to existing repository contract |
| Invalid isolated Postgres test schema name | `NakoError::InvalidInput` |

## Wrong vs Correct

### Wrong

```rust
let status = JobStatus::Queued; // silently hides database drift
```

### Correct

```rust
let status = JobStatus::parse(row_get::<String>(&row, "status")?)?;
```

## Evidence

- `crates/nako-db/src/sqlite/codec.rs`
- `crates/nako-db/src/postgres.rs`
- `crates/nako-core/src/job.rs`
