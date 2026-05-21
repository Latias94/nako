# Link Apply And Import Promotion — Milestones

Status: Active
Last updated: 2026-05-21

## M0 — Lane Open

Exit criteria:

- Workstream docs exist and agree.
- Mutation boundaries and non-goals are explicit.
- First executable slice is durable acceptance/audit domain work.

Primary evidence:

- `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`
- `docs/workstreams/link-apply-and-import-promotion/TODO.md`

## M1 — Durable Acceptance And Audit Domain

Exit criteria:

- [x] Promotion apply IDs, operation enums, state enums, and accepted plan
  snapshot exist in core.
- [x] Repository trait is explicit.
- [x] SQLite/PostgreSQL migrations preserve backend parity.
- [x] Contract tests round-trip apply/audit records and idempotency keys.

Primary evidence:

- `crates/taru-core/src/managed_import.rs`
- `crates/taru-core/src/repository/managed_import.rs`
- `crates/taru-db/migrations/0032_managed_import_promotion_applies.sql`
- `crates/taru-db/migrations/postgres/0004_managed_import_promotion_applies.sql`
- `crates/taru-db/src/contract_tests.rs`

## M2 — Acceptance And Replay Boundary

Exit criteria:

- Server app service records explicit apply acceptance.
- Matching idempotency keys replay safely.
- Mismatched requests are rejected.
- No storage or Media Source mutation occurs before mutation tasks.

Primary evidence:

- `crates/taru-server/src/app/managed_import.rs`
- `crates/taru-server/src/app/tests/managed_import.rs`

## M3 — VFS Mutation Primitive

Exit criteria:

- Copy/hardlink/symlink apply is storage-mediated.
- Planning safety checks are reused or revalidated.
- Unsupported backends return typed outcomes.
- Server code does not manipulate OS paths directly.

Primary evidence:

- `crates/taru-vfs/src/lib.rs`
- `crates/taru-vfs/src/local.rs`

## M4 — Promotion Apply Orchestration

Exit criteria:

- Apply revalidates plan facts.
- Storage target durability precedes catalog writes.
- Media Source and duplicate relationship commits are consistent.
- Redacted audit outcomes are recorded.

Primary evidence:

- `crates/taru-server/src/app/managed_import.rs`
- `crates/taru-server/src/app/tests/managed_import.rs`

## M5 — Rollback And Cleanup

Exit criteria:

- Injected partial failures are tested.
- Created targets are rolled back or recorded as cleanup-pending.
- Failed applies never mark artifacts promoted.

Primary evidence:

- rollback/cleanup tests in `taru-server` and `taru-vfs`.

## M6 — NFO Sidecar Decision

Exit criteria:

- NFO sidecar mutation is either implemented with backup/audit proof or split.
- NFO import/export is not smuggled into promotion apply.

## M7 — Closeout

Exit criteria:

- Fresh validation is recorded.
- Parent umbrella and workstream index agree on status.
- Follow-ons are explicit.
