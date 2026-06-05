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
  `managed-artwork`, `storage-runtime`, `source-identity`, `all-contracts`.

### 3. Contracts

- Focused suites must map to explicit nextest filters for their risk area.
- `all-contracts` is the broad escape hatch and should remain the only suite
  that uses the generic `postgres_` filter.
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
