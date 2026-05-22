# Link Apply And Import Promotion — Milestones

Status: Complete
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

- `crates/nako-core/src/managed_import.rs`
- `crates/nako-core/src/repository/managed_import.rs`
- `crates/nako-db/migrations/0032_managed_import_promotion_applies.sql`
- `crates/nako-db/migrations/postgres/0004_managed_import_promotion_applies.sql`
- `crates/nako-db/src/contract_tests.rs`

## M2 — Acceptance And Replay Boundary

Exit criteria:

- [x] Server app service records explicit apply acceptance.
- [x] Matching idempotency keys replay safely.
- [x] Mismatched requests are rejected.
- [x] No storage or Media Source mutation occurs before mutation tasks.

Primary evidence:

- `crates/nako-server/src/app/managed_import.rs`
- `crates/nako-server/src/app/tests/managed_import.rs`

## M3 — VFS Mutation Primitive

Exit criteria:

- [x] Copy/hardlink/symlink apply is storage-mediated.
- [x] Planning safety checks are reused or revalidated.
- [x] Unsupported backends return typed outcomes.
- [x] Server code does not manipulate OS paths directly.

Primary evidence:

- `crates/nako-vfs/src/lib.rs`
- `crates/nako-vfs/src/local.rs`

## M4 — Promotion Apply Orchestration

Exit criteria:

- [x] Apply revalidates plan facts.
- [x] Storage target durability precedes catalog writes.
- [x] Media Source and duplicate relationship commits are consistent.
- [x] Redacted audit outcomes are recorded.

Primary evidence:

- `crates/nako-server/src/app/managed_import.rs`
- `crates/nako-server/src/app/tests/managed_import.rs`

## M5 — Rollback And Cleanup

Exit criteria:

- [x] Injected partial failures are tested.
- [x] Created targets are cleaned up or recorded as cleanup-pending.
- [x] Failed applies never mark artifacts promoted.

Primary evidence:

- VFS cleanup primitives and tests in `crates/nako-vfs/src/lib.rs`,
  `crates/nako-vfs/src/local.rs`, and `crates/nako-vfs/src/cache.rs`.
- Promotion apply cleanup-complete / cleanup-pending tests in
  `crates/nako-server/src/app/tests/managed_import.rs`.
- Cleanup audit orchestration in `crates/nako-server/src/app/managed_import.rs`.

## M6 — NFO Sidecar Decision

Exit criteria:

- [x] NFO sidecar mutation is either implemented with backup/audit proof or split.
- [x] NFO import/export is not smuggled into promotion apply.

Primary evidence:

- Split decision in `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`.
- Follow-on workstream in `docs/workstreams/nfo-sidecar-promotion-apply`.

## M7 — Closeout

Exit criteria:

- [x] Fresh validation is recorded.
- [x] Parent umbrella and workstream index agree on status.
- [x] Follow-ons are explicit.

Primary evidence:

- Closeout evidence in
  `docs/workstreams/link-apply-and-import-promotion/EVIDENCE_AND_GATES.md`.
- Follow-on lane in `docs/workstreams/nfo-sidecar-promotion-apply`.
- Parent umbrella handoff in `docs/workstreams/post-rpd-product-hardening`.
