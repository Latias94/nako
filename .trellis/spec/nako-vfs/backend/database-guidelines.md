# Database Guidelines

`nako-vfs` does not own database adapters, but it does produce facts that may be
persisted by `nako-core` repository contracts and `nako-db` adapters.

## Rules

- Keep VFS persistence records in `nako-core` (`vfs_cache.rs`,
  `storage_health.rs`, staging records).
- Keep SQLite/Postgres persistence in `nako-db` (`sqlite/vfs_cache.rs`,
  `sqlite/vfs_health.rs`, `postgres/vfs_health.rs`, `postgres/vfs_staging.rs`).
- `nako-vfs` may call cache/storage abstractions passed into it, but should not
  import `nako-db` or `sqlx`.
- Persist redaction-safe cache and health facts, not raw provider errors that
  contain credentials or host-local details.

## Scenario: Staging Attribution Authority

### 1. Scope / Trigger

- Trigger: staging manifest records drive storage pressure, scan admission, and
  Admin diagnostics across VFS, DB, server, and API layers.
- Scope: `nako-core` staging structs, SQLite/Postgres staging migrations and
  adapters, server staging-budget policy slices, and Admin staging DTOs.

### 2. Signatures

- `NewStagingManifestRecord { attribution: StagingAttribution, ... }`
- `StagingManifestRecord { attribution: StagingAttribution, ... }`
- `StagingAttribution::{attributed(LibraryId), ambiguous(), unknown()}`
- DB columns:
  - `staging_manifest_records.attribution_kind`
  - `staging_manifest_records.attributed_library_id`

### 3. Contracts

- `attributed` requires `attributed_library_id`.
- `ambiguous` and `unknown` must not carry a library id.
- Server staging policy may count `attributed(library_id)` records toward the
  matching library slice and the backend slice.
- `ambiguous` and `unknown` records count only toward the backend slice.
- Admin diagnostics may expose `attribution_kind` and
  `attributed_library_id`, but must not expose `source_uri`, `local_path`,
  fingerprints, etags, credentials, raw backend errors, or host-local paths.

### 4. Validation & Error Matrix

- Unknown stored attribution kind -> `NakoError::Database`.
- Stored `attributed` without a library id -> `NakoError::Database`.
- Stored `ambiguous` or `unknown` with a library id -> `NakoError::Database`.
- Missing SQLite or PostgreSQL migration registration -> migrated stores must
  fail staging attribution contract tests.
- Admin response contains raw source/path/fingerprint/error details -> redaction
  contract violation.

### 5. Good/Base/Bad Cases

- Good: a probe staged from a known library writes
  `StagingAttribution::attributed(library_id)` and blocks that library when its
  slice is critical.
- Base: old or hand-written records default to `unknown` and only affect the
  backend aggregate.
- Bad: same-root WebDAV records are assigned to a library by matching
  `source_uri` path prefixes.

### 6. Tests Required

- SQLite and PostgreSQL contract tests round-trip `attributed`, `ambiguous`,
  and `unknown`, including updates from attributed to ambiguous.
- Server/Admin tests prove ambiguous same-root or multi-endpoint records do not
  increase any per-library policy slice.
- Redaction tests assert Admin staging records and policy slices do not include
  raw source locators, local paths, fingerprints, etags, credentials, or raw
  errors.

### 7. Wrong vs Correct

#### Wrong

```rust
if storage_path_matches_root(record.source_uri.as_str(), library_root) {
    library_policy.record(record);
}
```

#### Correct

```rust
if record.attribution.is_attributed_to(library_id) {
    library_policy.record(record);
}
```

## Review Checklist

- Is the fact a VFS runtime result or a durable storage health/cache record?
- Does the durable record live in `nako-core`?
- Are DB adapters updated in `nako-db` if the durable shape changed?
- Is the diagnostic safe for Admin/Public exposure?
