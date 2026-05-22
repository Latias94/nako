# Managed Import Staging — Milestones

Status: Complete
Last updated: 2026-05-21

## M0 — Lane Open

Exit criteria:

- Workstream docs exist and agree.
- Scope excludes generic downloaders and direct library writes.
- First executable slice is durable domain/schema work.

Primary evidence:

- `docs/workstreams/managed-import-staging/DESIGN.md`
- `docs/workstreams/managed-import-staging/TODO.md`

## M1 — Durable Import Artifact Domain

Exit criteria:

- [x] Core domain records and state enums exist.
- [x] Repository trait is explicit.
- [x] SQLite/PostgreSQL migrations preserve backend parity.
- [x] Contract tests round-trip Managed Import artifacts.

Primary evidence:

- `crates/nako-core/src/managed_import.rs`
- `crates/nako-core/src/repository/managed_import.rs`
- `crates/nako-db/migrations/0031_managed_import_artifacts.sql`
- `crates/nako-db/migrations/postgres/0003_managed_import_artifacts.sql`
- `crates/nako-db/src/contract_tests.rs`

## M2 — App Service Diagnostics

Exit criteria:

- [x] Server service can create/list staged import artifact diagnostics.
- [x] Diagnostics are redacted and library-scoped.
- [x] No external fetch or library write is performed.

Primary evidence:

- `crates/nako-server/src/app/managed_import.rs`
- `crates/nako-server/src/app/tests/managed_import.rs`

## M3 — Promotion Plan Preview

Exit criteria:

- [x] Promotion planning is non-mutating.
- [x] Plan includes destination, duplicate/link, metadata/NFO, and blocked
  reasons.
- [x] Tests prove library roots are unchanged by planning.

Primary evidence:

- `crates/nako-core/src/managed_import.rs`
- `crates/nako-server/src/app/managed_import.rs`
- `crates/nako-server/src/app/tests/managed_import.rs`

## M4 — Apply/Follow-On Decision

Exit criteria:

- [x] Apply is either implemented with rollback/audit proof or explicitly
  split.
- [x] Hardlink/symlink mutation is not smuggled into planning.
- [x] Follow-on workstream records operator confirmation, rollback, cleanup,
  audit, and storage mutation gates.

Primary evidence:

- `docs/workstreams/managed-import-staging/DESIGN.md`
- `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`

## M5 — Closeout

Exit criteria:

- [x] Fresh validation is recorded.
- [x] Parent umbrella and workstream index agree on status.
- [x] Follow-ons are explicit.

Primary evidence:

- `docs/workstreams/managed-import-staging/EVIDENCE_AND_GATES.md`
- `docs/workstreams/managed-import-staging/HANDOFF.md`
- `docs/workstreams/post-rpd-product-hardening/HANDOFF.md`
