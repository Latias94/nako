# Managed Import Staging — Milestones

Status: Active
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

- `crates/taru-core/src/managed_import.rs`
- `crates/taru-core/src/repository/managed_import.rs`
- `crates/taru-db/migrations/0031_managed_import_artifacts.sql`
- `crates/taru-db/migrations/postgres/0003_managed_import_artifacts.sql`
- `crates/taru-db/src/contract_tests.rs`

## M2 — App Service Diagnostics

Exit criteria:

- [x] Server service can create/list staged import artifact diagnostics.
- [x] Diagnostics are redacted and library-scoped.
- [x] No external fetch or library write is performed.

Primary evidence:

- `crates/taru-server/src/app/managed_import.rs`
- `crates/taru-server/src/app/tests/managed_import.rs`

## M3 — Promotion Plan Preview

Exit criteria:

- Promotion planning is non-mutating.
- Plan includes destination, duplicate/link, metadata/NFO, and blocked reasons.
- Tests prove library roots are unchanged by planning.

## M4 — Apply/Follow-On Decision

Exit criteria:

- Apply is either implemented with rollback/audit proof or explicitly split.
- Hardlink/symlink mutation is not smuggled into planning.

## M5 — Closeout

Exit criteria:

- Fresh validation is recorded.
- Parent umbrella and workstream index agree on status.
- Follow-ons are explicit.
