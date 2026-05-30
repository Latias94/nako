# Storage/VFS Resilience And Source Identity — Milestones

Status: Active
Last updated: 2026-05-30

## M0 — Authority Freeze

Exit criteria:

- Workstream docs exist and agree on scope, non-goals, task order, and gates.
- Architecture maps link this workstream as the concrete lane for storage/VFS
  resilience and source identity.

Primary evidence:

- `docs/workstreams/storage-vfs-resilience-and-source-identity/DESIGN.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

## M1 — Source Identity Evidence Policy

Status: Complete as of 2026-05-29.

Exit criteria:

- Scan commits can record or derive layered source identity evidence.
- Evidence confidence is explicit enough to distinguish strong identity from
  duplicate suggestion.
- Normal scan does not require full-file hashing.
- Sensitive evidence is not exposed through public/admin DTOs.

Primary gates:

- `cargo nextest run -p nako-library source_identity scan --no-fail-fast`
- SQLite/PostgreSQL contract tests if persistence changes.

Primary evidence:

- `crates/nako-core/src/media/source.rs`
- `crates/nako-library/src/scan.rs`
- `docs/workstreams/storage-vfs-resilience-and-source-identity/EVIDENCE_AND_GATES.md`

## M2 — Move/Rename Reconciliation

Status: Complete as of 2026-05-29.

Exit criteria:

- Strong-evidence moves or renames preserve relevant **Media Source** state.
- Weak evidence does not merge sources automatically.
- Source tombstones remain correct when files genuinely disappear.
- **Source Duplicate Relationship** remains separate from source identity.

Primary gates:

- `cargo nextest run -p nako-library rename_reconciliation --no-fail-fast`
- `cargo nextest run -p nako-db scan source_duplicate --no-fail-fast`

Primary evidence:

- `crates/nako-library/src/ingestion/source_commit.rs`
- `crates/nako-library/src/index.rs`
- `crates/nako-db/src/sqlite/scan.rs`
- `crates/nako-db/src/postgres/core_catalog.rs`
- `docs/workstreams/storage-vfs-resilience-and-source-identity/EVIDENCE_AND_GATES.md`

## M3 — Storage Failure Classification And Backoff

Status: Complete as of 2026-05-29.

Exit criteria:

- VFS-backed scan/probe/stage paths classify timeout, unavailable, permission,
  rate-limit, stale-cache, and partial-read failures consistently.
- A stuck or slow storage backend cannot block unrelated libraries.
- Classification is available to callers without leaking raw storage details.

Primary gates:

- `cargo nextest run -p nako-vfs --no-fail-fast`
- `cargo nextest run -p nako-server storage --no-fail-fast`

Primary evidence:

- `crates/nako-core/src/error.rs`
- `crates/nako-vfs/src/cache.rs`
- `crates/nako-vfs/src/webdav.rs`
- `crates/nako-library/src/failure.rs`
- `crates/nako-server/src/app/storage.rs`
- `crates/nako-server/src/app/staging.rs`
- `docs/workstreams/storage-vfs-resilience-and-source-identity/EVIDENCE_AND_GATES.md`

## M4 — Diagnostics And Cleanup

Status: Complete as of 2026-05-30.

Exit criteria:

- Admin diagnostics expose safe storage/source-identity pressure summaries.
- Partial staging cleanup and stale-cache conditions are observable.
- DTO and HTTP tests prove redaction of paths, **Source Locators**, raw ETags,
  credentials, and fingerprint values.

Primary gates:

- `cargo nextest run -p nako-server system storage --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` if DTOs change.

Primary evidence:

- `crates/nako-api/src/admin.rs`
- `crates/nako-api/src/admin/storage.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/app/catalog.rs`
- `crates/nako-server/src/app/storage.rs`
- `crates/nako-db/src/sqlite/catalog_governance.rs`
- `crates/nako-db/src/postgres/core_catalog.rs`
- `docs/workstreams/storage-vfs-resilience-and-source-identity/EVIDENCE_AND_GATES.md`

## M5 — Closeout

Exit criteria:

- SVRS-020 through SVRS-050 are complete or split into named follow-ons.
- Relevant architecture docs and workstream evidence are current.
- Final validation evidence is recorded in `EVIDENCE_AND_GATES.md`.
