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

### 4. Validation & Error Matrix

- Duplicate membership rows -> result contains one domain record and pagination
  is applied after deduplication.
- Source-only and state-only membership -> both are visible if the domain
  contract says either table grants membership.
- Other-principal playback state -> ignored for current-principal browse.
- Optional sort value is missing -> placed according to the explicit contract,
  not database defaults.
- Unsupported sort/facet text reaches repository SQL -> contract violation;
  parse to enums before the repository call.
- One-to-many child rows are joined before root pagination -> contract
  violation; paginate root records first, then batch-hydrate child collections.

### 5. Good/Base/Bad Cases

- Good: a library browse query joins membership and playback state once, orders
  with enum-owned SQL fragments, and returns a bounded page.
- Good: a source inventory projection pages `Media Source` rows first, then
  batch-loads `Media Item` external IDs and `Media Technical Facts` streams for
  only that page.
- Base: the server app service checks library existence, clamps `PageRequest`,
  calls the repository method, and maps domain records to DTOs.
- Bad: the app service loops over all library items, calls a per-item playback
  getter, sorts in memory, and slices the requested page.
- Bad: the repository joins `media_streams` directly into the source inventory
  page query and lets multiple stream rows change which sources appear on a
  page.

### 6. Tests Required

- Backend-agnostic repository contract tests for source-only membership,
  state-only membership, duplicate membership deduplication, current-principal
  user-state filtering, optional sort NULL ordering, stable tie-breaks, and
  pagination after filtering/order.
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
