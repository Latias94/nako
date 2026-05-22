# NFO Sidecar Promotion Apply — Milestones

Status: Complete
Last updated: 2026-05-21

## M0 — Lane Open

Exit criteria:

- [x] Workstream docs exist and agree.
- [x] Sidecar apply boundary is split from Managed Import promotion.
- [x] First executable slice is durable acceptance/audit domain work.

Primary evidence:

- `docs/workstreams/nfo-sidecar-promotion-apply/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/TODO.md`

## M1 — Durable Acceptance And Audit Domain

Exit criteria:

- [x] Sidecar apply IDs, operation/state enums, and accepted preview snapshot
  exist in core.
- [x] Repository trait is explicit.
- [x] SQLite/PostgreSQL migrations preserve backend parity.
- [x] Contract tests round-trip sidecar apply/audit records and idempotency
  keys.

## M2 — Acceptance And Replay Boundary

Exit criteria:

- [x] Server app service records explicit sidecar apply acceptance.
- [x] Matching idempotency keys replay safely.
- [x] Mismatched/stale requests are rejected.
- [x] No sidecar file write or canonical metadata mutation occurs before apply
  tasks.

## M3 — Export Apply

Exit criteria:

- [x] Export apply uses `taru-nfo` round-trip preservation.
- [x] Sidecar writes are mediated by VFS storage APIs.
- [x] Backup, atomic replace, and retention diagnostics are recorded.
- [x] Operator-facing diagnostics redact raw paths and raw XML.

## M4 — Import Authority Apply

Exit criteria:

- [x] NFO import applies local authority through canonical metadata boundaries.
- [x] User-locked fields are respected.
- [x] Accepted field, skipped field, and conflict outcomes are auditable.
- [x] Hierarchy confirmation is explicit and stale-safe.

## M5 — Rollback And Repair

Exit criteria:

- [x] Injected partial failures are tested.
- [x] Failed-before-mutation, rollback-complete, and repair-pending outcomes are
  distinguishable.
- [x] No failure path claims a false committed state.

## M6 — Closeout

Exit criteria:

- [x] Fresh validation is recorded.
- [x] Parent umbrella and follow-on split decisions agree.
- [x] API/UI/addon exposure work is either opened or explicitly deferred.
