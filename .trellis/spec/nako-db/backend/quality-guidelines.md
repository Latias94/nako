# Quality Guidelines

Persistence work must prove repository behavior, not only compile.

## Required Patterns

- Add or extend `contract_tests.rs` for every repository behavior that should
  hold across SQLite and Postgres.
- Include both fresh-store and migrated-store cases when the schema change
  affects migration behavior.
- Use `NAKO_TEST_POSTGRES_URL` for optional Postgres contract runs. Do not make
  local development require Postgres for SQLite-only checks.
- Keep SQLite and Postgres row shape comparable even when SQL dialect differs.
- Keep list methods bounded with `PageRequest`.

## Gate Selection

- Narrow DB compile:
  `cargo check -p nako-db --tests`
- Focused DB behavior:
  `cargo nextest run -p nako-db <filter> --no-fail-fast`
- Cross-crate repository contract:
  `cargo nextest run -p nako-core -p nako-db <filter> --no-fail-fast`
- Docs-only spec update:
  `git diff --check`

## Scenario: SQLite Playback Write Pressure Regression

### 1. Scope / Trigger

- Trigger: changing SQLite runtime options, playback heartbeat persistence,
  transcode runtime metrics, or any write path that can contend with the
  on-disk default database.
- Scope: `crates/nako-db/src/sqlite/runtime.rs`,
  `crates/nako-db/src/sqlite/playback.rs`,
  `crates/nako-db/src/contract_tests.rs`, and any docs that state the operator
  expectation for playback write pressure.
- Boundary: prove the existing on-disk policy is still acceptable under
  lock contention; do not add a queueing layer or new write worker just to hide
  pressure.

### 2. Signatures

- SQLite runtime policy:
  `SqliteRuntimeOptions::on_disk() -> SqliteRuntimeOptions`
- Playback write paths:
  `record_playback_session_heartbeat(PlaybackSessionHeartbeat) -> Result<...>`
  and `update_transcode_session_runtime_metrics(TranscodeSessionId, TranscodeSessionRuntimeMetrics) -> Result<...>`

### 3. Contracts

- On-disk SQLite keeps WAL enabled, uses the configured busy timeout, and
  keeps a bounded connection pool.
- A held write lock should make playback heartbeat and transcode metric writes
  wait rather than fail spuriously.
- When the lock is released, both writes must complete successfully and update
  the expected rows.
- In-memory SQLite remains single-connection and is not the target of this
  regression.

### 4. Validation & Error Matrix

- WAL or busy-timeout policy regressed -> pressure test or policy test fails.
- Playback write returns lock error while the lock is held -> regression in
  SQLite runtime policy or repository behavior.
- Playback write succeeds before the held lock is released -> regression in
  pool or transaction handling.
- Lock release does not unblock the writes -> regression in connection cleanup
  or test setup.

### 5. Good/Base/Bad Cases

- Good: a deterministic `BEGIN IMMEDIATE` test holds the SQLite write lock,
  starts heartbeat and metrics writes in parallel, confirms they are pending,
  then releases the lock and observes both writes complete.
- Base: on-disk runtime still reports WAL, the configured busy timeout, and
  the expected pool size.
- Bad: adding a retry queue or background write worker only to paper over the
  lock contention.

### 6. Tests Required

- `cargo nextest run -p nako-db on_disk_runtime_uses_wal_and_busy_timeout --no-fail-fast`
- `cargo nextest run -p nako-db on_disk_runtime_keeps_playback_writes_pending_while_a_write_lock_is_held --no-fail-fast`
- `cargo check -p nako-db --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

### 7. Wrong vs Correct

#### Wrong

```rust
// Treat a lock wait as a hard failure and add a new queue to hide it.
```

#### Correct

```rust
let mut lock_conn = store.pool().acquire().await.unwrap();
sqlx::query("BEGIN IMMEDIATE").execute(&mut *lock_conn).await.unwrap();
// Playback writes stay pending until the lock is released.
```

## Scenario: Repository-Backed Browse Queries

### 1. Scope / Trigger

- Trigger: adding or changing a repository method that powers a Public/Admin
  browse or list surface from multiple persistence tables.
- Scope: `nako-core` repository trait signature, SQLite/PostgreSQL adapters,
  `NakoDatabase` facade forwarding, app-service delegation, and shared
  `contract_tests.rs` coverage.

### 2. Signatures

- Repository method shape:
  `fn list_*_for_browse(..., page-or-query) -> Result<Vec<DomainRecord>>`.
- Query records should use domain enums and `PageRequest`; do not expose SQL,
  adapter-private row structs, or raw query strings through `nako-core`.

### 3. Contracts

- Filtering, deduplication, ordering, and `LIMIT/OFFSET` happen in the
  repository query, not by loading every row into an app service.
- If membership can come from several tables, aggregate to one row per domain
  record before applying pagination. Multi-source items must not duplicate
  results or shift page boundaries.
- User-specific state joins must bind the current principal and must not let
  another principal's state affect filtering or ordering.
- Dynamic `ORDER BY` fragments may be generated only from trusted domain enum
  branches. User-provided values must be parsed before the repository call and
  never interpolated into SQL.
- Optional sort keys must define NULL ordering explicitly in both SQLite and
  PostgreSQL. Preserve existing semantics when replacing app-layer sorting.
- Stable tie-breaks belong in the query contract and must remain independent of
  ascending/descending primary sort direction.
- When a list projection hydrates one-to-many child records such as media
  streams or external IDs, apply `LIMIT/OFFSET` to the root domain record first,
  then batch-load child records for that bounded page. Joining child rows before
  pagination shifts page boundaries and duplicates root records.
- Public Catalog item-list projections that enforce Library Access must apply
  access filtering, duplicate-source deduplication, stable ordering, and
  `LIMIT/OFFSET` inside the repository query. Reuse a shared adapter-local
  access predicate helper when several projections need the same `Media Item`
  visibility rule. Administrator principals may keep source-less item semantics
  only when the existing access contract requires it; ordinary principals must
  see an item only through an accessible `Media Source` and `Media Library`.
- Public Catalog search projections that enforce Library Access must filter the
  `search_documents` candidate set before calling `nako-search` evaluation and
  before applying `SearchQuery` pagination. Keep access predicates in
  `nako-db`; do not add principals or Library Access concepts to `nako-search`.

### 4. Validation & Error Matrix

- Duplicate membership rows -> result contains one domain record and pagination
  is applied after deduplication.
- Source-only and state-only membership -> both are visible if the domain
  contract says either table grants membership.
- Other-principal playback state -> ignored for current-principal browse.
- If the schema already enforces parent existence with a foreign key, do not
  invent an orphan-row fixture for list-projection coverage; use access holes
  or source-less admin rows instead.
- Optional sort value is missing -> placed according to the explicit contract,
  not database defaults.
- Unsupported sort/facet text reaches repository SQL -> contract violation;
  parse to enums before the repository call.
- One-to-many child rows are joined before root pagination -> contract
  violation; paginate root records first, then batch-hydrate child collections.
- Inaccessible `Media Item` rows consume page slots before Library Access is
  applied -> contract violation; access must be part of the root query.
- Inaccessible `search_documents` consume search page slots before Library
  Access is applied -> contract violation; filter candidates before scoring and
  pagination.
- A `Media Item` with multiple accessible `Media Source` rows appears more than
  once or shifts page boundaries -> contract violation; deduplicate before
  ordering and pagination.
- Ordinary principal sees source-less `Media Item` rows -> contract violation.
- Administrator principal cannot see source-less rows on a surface whose
  existing access contract allows them -> contract regression.

### 5. Good/Base/Bad Cases

- Good: a library browse query joins membership and playback state once, orders
  with enum-owned SQL fragments, and returns a bounded page.
- Good: a source inventory projection pages `Media Source` rows first, then
  batch-loads `Media Item` external IDs and `Media Technical Facts` streams for
  only that page.
- Good: Public Catalog `/items` and relation item lists (`Person`, `Tag`,
  `Genre`) use repository methods that include Library Access predicates before
  `ORDER BY title ASC, id ASC`, `LIMIT`, and `OFFSET`.
- Good: Public Catalog `/search` asks `nako-db` for accessible search hits;
  adapters read only Library-Access-visible `search_documents`, then reuse
  `nako-search` for deterministic scoring and `SearchQuery` pagination.
- Base: the server app service checks library existence, clamps `PageRequest`,
  calls the repository method, and maps domain records to DTOs.
- Bad: the app service loops over all library items, calls a per-item playback
  getter, sorts in memory, and slices the requested page.
- Bad: the repository joins `media_streams` directly into the source inventory
  page query and lets multiple stream rows change which sources appear on a
  page.
- Bad: HTTP receives a page of catalog items and then loops over it with
  `item_has_access`, because inaccessible rows have already consumed page
  capacity.
- Bad: the server loops over global search pages and batches access checks to
  fill holes left by inaccessible high-scoring search hits.

### 6. Tests Required

- Backend-agnostic repository contract tests for source-only membership,
  state-only membership, duplicate membership deduplication, current-principal
  user-state filtering, optional sort NULL ordering, stable tie-breaks, and
  pagination after filtering/order.
- Public Catalog Library Access projections need backend-agnostic contracts for
  ordinary user policy, role policy, duplicate-source deduplication, admin
  source-less semantics, batch-by-id ordering, and page-hole regression cases.
- Public Catalog search access projections need a backend-agnostic hidden-hit
  page-hole contract proving a hidden high-scoring search document does not
  consume the first visible search page.
- Focused SQLite gate:
  `cargo nextest run -p nako-db <browse-contract-filter> --no-fail-fast`.
- PostgreSQL ignored contract must compile and should be run with
  `NAKO_TEST_POSTGRES_URL` when local tooling is available.
- Server route tests should remain focused on query parsing, access checks, and
  DTO shape; do not duplicate every repository ordering case in HTTP tests.

### 7. Wrong vs Correct

#### Wrong

```rust
let mut rows = store.list_items(page_all).await?;
for row in &mut rows {
    row.state = store.get_user_state(principal, row.id).await?;
}
rows.sort_by(...);
Ok(rows[offset..end].to_vec())
```

#### Correct

```rust
let page = query.page.clamped();
let rows = store
    .list_library_items_for_browse(library_id, principal, &LibraryItemBrowseQuery { page, ..query })
    .await?;
```

## Scenario: PostgreSQL Contract Harness Suite Selection

### 1. Scope / Trigger

- Trigger: adding or changing backend-neutral repository contracts that should
  run against PostgreSQL, or changing the PostgreSQL contract harness command
  surface.
- Scope: `crates/nako-db/src/contract_tests.rs`,
  `scripts/postgres-contract-harness.ps1`,
  `scripts/postgres-contract-harness.sh`, and durable docs that enumerate
  harness suites.

### 2. Signatures

- PowerShell:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite <suite>`
- Bash:
  `bash scripts/postgres-contract-harness.sh --suite <suite>`
- Current suites:
  `managed-artwork`, `storage-runtime`, `source-identity`, `job-runtime`,
  `storage-source-parity`, `all-contracts`.

### 3. Contracts

- Focused suites must map to explicit nextest filters for their risk area.
- `all-contracts` is the broad escape hatch and should remain the only suite
  that uses the generic `postgres_` filter.
- `storage-source-parity` is the combined M2 storage-VFS reliability suite;
  it should stay explicit rather than silently replacing the focused
  `storage-runtime` and `source-identity` suites.
- `job-runtime` is the focused durable job runtime suite. It covers job lease,
  cancellation, retry/backoff, queue-pressure, priority, and lease recovery
  contracts without pulling every ignored PostgreSQL contract through
  `all-contracts`.
- PowerShell and Bash suite names must stay in parity.
- Harness behavior for caller-provided database URLs, temporary local clusters,
  safe skip, `RequireTooling` / `--require-tooling`, and cleanup must remain
  unchanged when adding a suite.

### 4. Validation & Error Matrix

- Suite added only to one shell -> cross-platform release gate drift.
- Focused suite uses `postgres_` -> loses the intended narrow risk boundary.
- Suite filter typo -> nextest runs zero or the wrong ignored contracts.
- Harness cleanup change -> risk of leaving local PostgreSQL state or deleting
  outside `target/postgres-contract/`.

### 5. Good/Base/Bad Cases

- Good: source identity work runs `source-identity` and proves the existing
  PostgreSQL ignored contracts for media source identity, scan source-unit
  writes, source duplicate relationships, and VFS attribution.
- Good: control-plane or storage repair runtime work runs `job-runtime` and
  proves the existing PostgreSQL ignored contracts for durable job lease,
  cancellation, retry, queue-pressure, priority, and recovery behavior.
- Base: release-critical Managed Artwork parity keeps using
  `managed-artwork`.
- Bad: a new persistence contract is documented as "run all contracts" without
  considering whether a focused suite should cover the risk area.

### 6. Tests Required

- Parse/check both harness scripts after command-surface changes.
- Run matching SQLite contract filters for the selected risk area.
- Run the PowerShell PostgreSQL harness suite when local tooling or
  `NAKO_TEST_POSTGRES_URL` is available; otherwise record the safe skip.
- Run `git diff --check` after script or docs changes.

### 7. Wrong vs Correct

#### Wrong

```powershell
pwsh -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
```

#### Correct

```powershell
pwsh -File scripts/postgres-contract-harness.ps1 -Suite source-identity
```

## Forbidden Patterns

- Do not skip Postgres adapter updates by hiding new behavior behind SQLite
  helper methods.
- Do not use live external services in contract tests other than the optional
  Postgres URL path already modeled in the crate.
- Do not add unbounded scans or list queries for Admin/Public surfaces.
- Do not store raw secrets, playback tickets, or raw local paths in diagnostic
  columns unless an ADR and redaction contract explicitly require it.

## Review Checklist

- Migration file exists for SQLite and Postgres.
- Both migration arrays include the same version.
- Inserts, updates, row readers, list filters, and contract tests all know about
  the new field.
- Stored enum conversions reject unknown values.
- The new SQL preserves existing pagination, ordering, and transaction rules.
