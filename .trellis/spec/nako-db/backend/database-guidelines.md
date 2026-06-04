# Database Guidelines

Use SQLx directly in adapter modules. Repository traits come from `nako-core`;
database-specific SQL and row mapping stay inside `nako-db`.

## Query Patterns

- Use parameter binding, not interpolated values, for row data.
- String-built SQL is allowed only for controlled identifiers such as table or
  owner-column names in shared helper functions. Do not interpolate user input.
- Convert all SQLx errors through backend helper functions such as
  `database_error`.
- Keep row mapping close to the adapter that owns the query. Use shared codec
  helpers for repeated enum, ID, JSON, and numeric conversions.
- Use transactions for multi-table commits that must be atomic, such as catalog
  graph replacement, provider mapping commits, artwork lifecycle writes, and
  durable job state transitions.

## Migrations

- Add SQLite schema changes under `crates/nako-db/migrations/`.
- Add PostgreSQL schema changes under `crates/nako-db/migrations/postgres/`.
- Register every new migration in both:
  - `crates/nako-db/src/sqlite/migrations.rs`
  - `crates/nako-db/src/postgres.rs`
- Keep version numbers and descriptions aligned across SQLite and Postgres.
- When an incremental SQLite migration must enforce a cross-column invariant
  that cannot be added through `ALTER TABLE ... ADD COLUMN`, add the SQLite-side
  enforcement in the migration itself (for example trigger-based guards) so the
  migrated-store contract matches PostgreSQL checks.
- Keep baseline migrations as direct schema shape. Do not replay historical
  `ALTER TABLE` fragments in baseline SQL.
- After a schema change, update row mappers, insert/update SQL, list filters,
  and contract tests in the same task.

## Scenario: Baseline Plus Incremental Migration Replay

### 1. Scope / Trigger

- Trigger: adding a database schema field through a new migration.
- Scope: `crates/nako-db/migrations/baseline.sql`, incremental migration files,
  `sqlite/migrations.rs`, `postgres.rs`, adapter row mappers, and migration
  tests.

### 2. Signatures

- SQLite migrator registration:
  `const MIGRATIONS: &[(i64, &str, &str)]`
- PostgreSQL migrator registration:
  `const MIGRATIONS: &[(i64, &str, &str)]`
- New SQLite migration path:
  `crates/nako-db/migrations/<version>_<description>.sql`
- New PostgreSQL migration path:
  `crates/nako-db/migrations/postgres/<version>_<description>.sql`

### 3. Contracts

- A fresh store applies `baseline.sql` first, then every registered incremental
  migration in order.
- Do not add the same new column/index to both `baseline.sql` and the new
  incremental migration unless the incremental migration is explicitly
  idempotent for already-baselined stores.
- For the current repository migration model, prefer putting newly added fields
  in the incremental migration only; keep baseline as version-1 direct schema.
- Update expected applied migration versions when a new version is registered.

### 4. Validation & Error Matrix

- New column exists in baseline and `ALTER TABLE ... ADD COLUMN` migration ->
  fresh SQLite migration fails with duplicate column.
- Migration file exists but is not registered -> migrated stores miss the new
  schema shape.
- Row mapper selects a new column before migration registration -> migrated
  stores fail at runtime.
- PostgreSQL uses `IF NOT EXISTS` but SQLite does not -> SQLite still needs a
  replay-safe shape or baseline must not include the new column.

### 5. Good/Base/Bad Cases

- Good: `0004_example.sql` adds `example_column`, both migrators register
  version 4, row mappers include the field, and migration tests expect
  `[1, 2, 3, 4]`.
- Base: `baseline.sql` remains the version-1 direct schema without historical
  `ALTER TABLE` fragments.
- Bad: `baseline.sql` includes `example_column` and `0004_example.sql` also
  runs `ALTER TABLE ... ADD COLUMN example_column`, causing fresh SQLite
  migration failure.

### 6. Tests Required

- SQLite migration test asserting the full applied version list.
- SQLite or repository round-trip test that calls `migrate()` on a fresh
  in-memory store before using the new field.
- Contract test asserting the new field round-trips through repository traits.
- PostgreSQL migration registration or baseline-shape tests when a PostgreSQL
  migration is added.

### 7. Wrong vs Correct

#### Wrong

```sql
-- baseline.sql
CREATE TABLE vfs_cache_failures (
    uri TEXT NOT NULL,
    library_id TEXT
);

-- 0004_vfs_cache_failure_authority.sql
ALTER TABLE vfs_cache_failures
    ADD COLUMN library_id TEXT;
```

#### Correct

```sql
-- baseline.sql remains the version-1 table shape.
CREATE TABLE vfs_cache_failures (
    uri TEXT NOT NULL
);

-- 0004_vfs_cache_failure_authority.sql owns the new field.
ALTER TABLE vfs_cache_failures
    ADD COLUMN library_id TEXT;
```

## Naming Conventions

- Table and column names use `snake_case`.
- ID columns store strong IDs as text strings unless an existing table uses a
  different established shape.
- JSON payload columns use `*_json` names.
- Timestamp columns use the existing `*_at` or `*_at_ms` convention from the
  surrounding table; do not mix both styles inside one table without a reason.
- Index names should describe table plus key or access pattern.

## Scenario: Durable Job Priority Policy

### 1. Scope / Trigger

- Trigger: durable job rows gained a scheduler-visible `priority` column.
- Scope: `nako-core` job structs, SQLite/Postgres migrations, SQLite/Postgres
  job adapters, retry enqueue behavior, lease claiming, and DB contract tests.
- Boundary: priority is a generic durable-job policy. Do not encode provider,
  metadata review, addon, scan, or Admin-Web semantics in the scheduler.

### 2. Signatures

- `NewJob { priority: JobPriority, ... }`
- `Job { priority: JobPriority, ... }`
- `JobLeaseRepository::list_claimable_jobs_for_lease(filter, page) ->
  Result<Vec<Job>>`
- `JobPriority::{Low, Normal, High}` maps to persisted scores
  `0`, `50`, and `100`.
- `jobs.priority` is `INTEGER NOT NULL DEFAULT 50` on SQLite and
  `bigint NOT NULL DEFAULT 50` on Postgres.

### 3. Contracts

- Every enqueue path must set a priority explicitly. Existing work should use
  `JobPriority::Normal` unless a generic durable-job policy says otherwise.
- `enqueue_job_retry` must copy the source job priority to the retry row.
- `list_claimable_jobs_for_lease` must use the same filter semantics and aged
  fairness / priority / FIFO ordering as `claim_next_job_lease`, but it must
  not mutate job state.
- `claim_next_job_lease` orders eligible queued jobs by aged fairness first,
  then priority, then FIFO tie-breakers.
- API/Admin diagnostics should not expose a priority field unless a separate
  read-only diagnostic follow-on adds that surface deliberately.

### 4. Validation & Error Matrix

- Unknown persisted score -> return `NakoError::Database`.
- Missing migration registration -> migrated stores lack `jobs.priority` and
  job repository tests must fail.
- Retry row priority differs from source -> contract violation.
- Claimable preview ordering differs from lease claim ordering -> scheduler
  fairness contract violation.
- Business-specific scheduler branch -> architecture violation against ADR 0053.

### 5. Good/Base/Bad Cases

- Good: a high-priority generic job claims before a fresh low-priority job in
  the same filter/resource class.
- Base: old jobs migrated without an explicit score become normal priority.
- Bad: an endless stream of fresh high-priority rows prevents aged low-priority
  rows from ever being claimed.

### 6. Tests Required

- Contract test for priority ordering.
- Contract test for starvation guard/fairness.
- Contract test that claimable preview ordering matches later lease claim
  ordering.
- Contract test that retry and lease recovery preserve priority.
- Migration tests must assert the new migration version is applied.

### 7. Wrong vs Correct

#### Wrong

```rust
NewJob {
    kind: JobKind::MetadataCandidateReviewBatchApply,
    resource_class: "metadata.candidate_review.apply".to_owned(),
    priority: JobPriority::High, // business-specific scheduler shortcut
    // ...
}
```

#### Correct

```rust
NewJob {
    kind,
    resource_class,
    priority: JobPriority::Normal, // default generic durable-job policy
    // ...
}
```

## Common Mistakes

- Adding a column to SQLite only.
- Registering migration files but forgetting the `MIGRATIONS` array.
- Adding a PostgreSQL `CHECK` for a new persisted invariant but leaving
  migrated SQLite stores able to write the illegal combination.
- Updating row inserts but not row readers.
- Adding a repository trait method without a contract test.
- Returning a raw SQLx error type through a `nako-core` repository trait.
