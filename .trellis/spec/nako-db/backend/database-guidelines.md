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
- PostgreSQL `SELECT DISTINCT` queries must order only by expressions present in
  the selected projection. If a query does not join a duplicating relation,
  prefer removing redundant `DISTINCT`; if `DISTINCT` is required, select a
  stable typed sort projection and order by that selected expression. SQLite may
  accept looser `DISTINCT ... ORDER BY` SQL that PostgreSQL rejects, so run the
  ignored PostgreSQL contract when changing shared access projections.

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

## Scenario: Compatibility Key Normalization Migration

### 1. Scope / Trigger

- Trigger: persisted identity keys change shape, such as replacing a legacy
  file-derived key with a canonical domain key.
- Scope: SQLite/PostgreSQL migration files, migration registration tests,
  repository or app behavior that previously had runtime fallback lookup, and
  focused behavior tests for the owning app flow.

### 2. Signatures

- New migration:
  `crates/nako-db/migrations/<version>_<normalization>.sql`
- PostgreSQL parity migration:
  `crates/nako-db/migrations/postgres/<version>_<normalization>.sql`
- Registration:
  `MIGRATIONS: &[(i64, &str, &str)]` in both SQLite and PostgreSQL adapters.

### 3. Contracts

- Normalize stored data before deleting runtime compatibility lookup.
- When a legacy row and canonical row represent the same domain identity,
  collapse to one canonical row before updating the unique key.
- Define winner priority explicitly in SQL. For acquisition intake candidates,
  accepted or managed-import-linked rows outrank unlinked transient rows, then
  newer observations win.
- Merge meaningful nullable fields from loser rows before deletion. Preserve
  linked artifact IDs, diagnostics, display/intended locator facts, size,
  fingerprint, earliest `first_seen_at_ms`, latest `last_seen_at_ms`, and
  latest `updated_at_ms` where applicable.
- Keep the migration idempotent for empty or already-normalized databases.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Legacy-only row exists | Rewrite its key to the canonical form |
| Legacy and canonical rows conflict under a unique key | Merge fields, delete loser rows, then update the winner key |
| Accepted linked row conflicts with newer transient row | Keep the accepted/linked row and merge the newer observation timestamp |
| Already-normalized database | Migration performs no data change and succeeds |
| Runtime fallback removed before migration coverage exists | Treat as a compatibility regression |

### 5. Good/Base/Bad Cases

- Good: a migration builds a bounded temporary normalization set, updates the
  winner, deletes losers, then changes the unique key value.
- Base: an empty fresh store applies the migration after baseline without
  touching any row.
- Bad: app code keeps probing legacy keys forever after data is normalized, or
  a migration updates legacy rows directly and hits the canonical unique key.

### 6. Tests Required

- SQLite migration test for a legacy-only row.
- SQLite migration test for duplicate legacy/canonical collapse.
- PostgreSQL migration registration or live PostgreSQL migration coverage when
  tooling is available.
- Focused app test proving the runtime path now uses only canonical lookup and
  still avoids duplicate rows.

### 7. Wrong vs Correct

#### Wrong

```sql
UPDATE acquisition_intake_candidates
SET source_key = 'watch_folder:' || source_uri
WHERE source_kind = 'watch_folder';
```

This can violate the unique key when both legacy and canonical rows already
exist for the same candidate identity.

#### Correct

```sql
CREATE TEMP TABLE normalization AS
SELECT winner_id, canonical_source_key
FROM ranked_candidates
WHERE candidate_rank = 1;

DELETE FROM acquisition_intake_candidates
WHERE id IN (SELECT loser_id FROM ...);

UPDATE acquisition_intake_candidates
SET source_key = normalization.canonical_source_key
FROM normalization
WHERE id = normalization.winner_id;
```

The migration selects and merges a winner before changing the unique key value.

## Scenario: User Playback Profile Preference Persistence

### 1. Scope / Trigger

- Trigger: adding or changing persisted current-user playback profile
  preferences.
- Scope: `UserPlaybackProfilePreferenceRepository`,
  `UserPlaybackProfilePreference`,
  `UserPlaybackProfilePreferenceWrite`,
  SQLite/PostgreSQL `user_playback_profile_preferences` migrations,
  adapter CRUD SQL, `NakoDatabase` facade forwarding, and repository contract
  tests.
- Boundary: persistence stores a resolved effective capability payload. It does
  not resolve playback profiles, read presets, apply playback policies, or
  manage multiple named device profiles.

### 2. Signatures

- Table: `user_playback_profile_preferences`.
- Identity: `principal_id` primary key.
- Columns:
  `principal_id`, `capabilities_json`, `updated_at_ms`, `version`,
  `created_at`, and `updated_at`.
- Repository:
  `upsert_user_playback_profile_preference(write) ->
  Result<UserPlaybackProfilePreference>`.
- Repository:
  `get_user_playback_profile_preference(principal_id) ->
  Result<Option<UserPlaybackProfilePreference>>`.
- Repository:
  `delete_user_playback_profile_preference(principal_id) -> Result<bool>`.

### 3. Contracts

- `nako-core` records store `capabilities_json: String` so `nako-core` does not
  depend on `nako-playback` or Public Client protocol DTOs.
- SQLite stores `capabilities_json` as `TEXT NOT NULL` with `json_valid`.
  PostgreSQL stores it as `jsonb NOT NULL`; upsert SQL must cast the bound text
  parameter with `$2::jsonb`.
- Upsert identity is `principal_id`; replacing a preference increments
  `version`, updates `updated_at_ms`, and keeps exactly one row for the
  principal.
- Delete is idempotent and returns whether a row was removed.
- Repository methods do not accept arbitrary user ids from HTTP. The server
  supplies the authenticated principal id after auth resolution.
- This table is separate from `user_playback_states`; do not widen item
  progress rows with device/profile preference fields.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Missing row | `get` returns `Ok(None)` |
| `delete` on missing row | Returns `false` |
| First upsert for a principal | Creates version `1` row |
| Second upsert for same principal | Replaces JSON, increments version, preserves one row |
| Upsert for another principal | Does not affect the first principal |
| SQLite JSON is invalid | Schema rejects the write |
| PostgreSQL JSON text is bound without `::jsonb` | Treat as adapter bug; live PostgreSQL may fail |
| Migration file exists but migrator registration is missing | Migration contract failure |

### 5. Good/Base/Bad Cases

- Good: store the resolved capability JSON emitted by the server playback
  profile resolver and return it unchanged for API mapping.
- Base: a fresh migrated SQLite store applies the new version and can round
  trip absent/upsert/replace/delete behavior.
- Bad: storing unresolved user request JSON and relying on future API code to
  re-resolve it.
- Bad: adding a `playback_profile` column to `user_playback_states`.

### 6. Tests Required

- SQLite migration version registration test.
- PostgreSQL migration registration test.
- Backend-agnostic repository contract for absent, upsert, replace,
  other-principal isolation, and delete idempotence.
- Focused gate:
  `cargo nextest run -p nako-db user_playback_profile_preference --no-fail-fast`.
- Compile gate:
  `cargo check -p nako-db --tests`.

### 7. Wrong vs Correct

#### Wrong

```sql
INSERT INTO user_playback_profile_preferences (principal_id, capabilities_json)
VALUES ($1, $2)
```

#### Correct

```sql
INSERT INTO user_playback_profile_preferences (principal_id, capabilities_json)
VALUES ($1, $2::jsonb)
```

The Postgres adapter binds resolved capabilities as text from the neutral core
record and explicitly casts it to `jsonb` at the SQL boundary.

## Scenario: Source Duplicate Relationship Pair Identity

### 1. Scope / Trigger

- Trigger: changing `SourceDuplicateRelationship` persistence, source identity
  reconciliation, source fingerprint evidence, or any writer that may suggest
  duplicate media sources.
- Scope: `SourceDuplicateRepository`, SQLite/PostgreSQL source duplicate
  adapters, migrations, scan commit persistence, and repository contract tests.
- Boundary: this contract records reviewable duplicate evidence. It must not
  merge `MediaSource` or `MediaItem` identity.

### 2. Signatures

- `SourceDuplicateRelationship { id, source_id, duplicate_source_id,
  evidence_kind, evidence_value, status, confidence_milli }`
- `SourceDuplicateRelationship::canonicalized()` orders the source pair before
  persistence.
- `SourceDuplicateRepository::upsert_source_duplicate_relationship(
  relationship
) -> Result<()>`
- `SourceDuplicateRepository::get_source_duplicate_relationship_by_pair(
  source_id, duplicate_source_id
) -> Result<Option<SourceDuplicateRelationship>>`
- `MediaRepository::list_media_sources_by_fingerprint(
  library_id, fingerprint, exclude_source_id, PageRequest
) -> Result<Vec<MediaSourceFingerprintMatch>>`
- SQLite stores source ids as `TEXT`; PostgreSQL stores source ids as `uuid`.

### 3. Contracts

- `(source_id, duplicate_source_id)` is the repository identity for an upsert.
  The generated `SourceDuplicateRelationshipId` is stable once the canonical
  pair exists.
- Writers must canonicalize the pair before persistence and must reject or fail
  non-distinct source ids through schema constraints.
- Re-upserting the same canonical pair with a different relationship id updates
  only mutable payload fields: `evidence_kind`, `evidence_kind_key`,
  `evidence_value`, `status`, `confidence_milli`, and `updated_at`.
- Re-upserting the same canonical pair must not update the stored `id`,
  `source_id`, or `duplicate_source_id`.
- SQLite and PostgreSQL must both enforce unique canonical pairs through schema
  and use `ON CONFLICT(source_id, duplicate_source_id)` for upsert behavior.
- Pair lookup must canonicalize input order before querying so callers can
  check existing relationships without knowing stored order.
- Fingerprint match queries must be same-library and bounded by `PageRequest`.
  When used for reconciliation planning, they must exclude the target source
  before applying `LIMIT/OFFSET` so pagination is candidate-oriented. They may
  project redaction-safe stale state from `source_states.tombstoned` but must
  not return raw source locators beyond the existing internal `MediaSource`
  repository record.

### 4. Validation & Error Matrix

- Same canonical pair with different id -> update existing row payload and keep
  original id.
- Reversed pair input -> canonicalize to the same stored row.
- Pair lookup with reversed input -> return the stored canonical row.
- Fingerprint match query with a cross-library matching fingerprint -> exclude
  the other-library source.
- Fingerprint match query with an excluded target source -> exclude it before
  pagination.
- Fingerprint match query sees a tombstoned source state -> mark the candidate
  `stale = true`.
- Same source used twice -> database constraint failure; do not silently create
  a self-duplicate relationship.
- Missing PostgreSQL pair unique index -> `ON CONFLICT(source_id,
  duplicate_source_id)` cannot be the durable upsert contract.
- SQLite/PostgreSQL migration version drift -> contract parity failure.

### 5. Good/Base/Bad Cases

- Good: source hash reconciliation retries the same pair and refreshes evidence
  without creating duplicate rows.
- Good: read-only reconciliation finds same-library fingerprint matches through
  `list_media_sources_by_fingerprint`, reads existing pair status through
  `get_source_duplicate_relationship_by_pair`, and leaves duplicate rows
  unchanged.
- Base: a manually suggested duplicate pair remains addressable by its original
  relationship id after stronger evidence arrives.
- Bad: a writer conflicts only on `id`, so retries with new ids create
  duplicate rows or fail on a pair unique constraint.

### 6. Tests Required

- Backend-agnostic contract test under `ContractFamily::SourceDuplicate` proving
  pair-idempotent upsert, reversed input canonicalization, latest payload, one
  listed row, and stable original id.
- Backend-agnostic contract test under `ContractFamily::SourceDuplicate` proving
  bounded same-library fingerprint matching, pagination, stale source-state
  projection, and canonical pair lookup.
- SQLite focused gate:
  `cargo nextest run -p nako-db source_duplicate --no-fail-fast`
- PostgreSQL ignored contract must use the same contract under
  `NAKO_TEST_POSTGRES_URL`.
- Migration tests must assert the source duplicate pair identity migration is
  registered and that fresh SQLite stores apply the new version list.

### 7. Wrong vs Correct

#### Wrong

```sql
ON CONFLICT(id) DO UPDATE SET
    source_id = excluded.source_id,
    duplicate_source_id = excluded.duplicate_source_id,
    evidence_kind = excluded.evidence_kind;
```

#### Correct

```sql
ON CONFLICT(source_id, duplicate_source_id) DO UPDATE SET
    evidence_kind = excluded.evidence_kind,
    evidence_kind_key = excluded.evidence_kind_key,
    evidence_value = excluded.evidence_value,
    status = excluded.status,
    confidence_milli = excluded.confidence_milli;
```

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
