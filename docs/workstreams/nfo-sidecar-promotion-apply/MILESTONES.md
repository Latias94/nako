# NFO Sidecar Promotion Apply — Milestones

Status: Active
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

- [ ] Sidecar apply IDs, operation/state enums, and accepted preview snapshot
  exist in core.
- [ ] Repository trait is explicit.
- [ ] SQLite/PostgreSQL migrations preserve backend parity.
- [ ] Contract tests round-trip sidecar apply/audit records and idempotency
  keys.

## M2 — Acceptance And Replay Boundary

Exit criteria:

- [ ] Server app service records explicit sidecar apply acceptance.
- [ ] Matching idempotency keys replay safely.
- [ ] Mismatched/stale requests are rejected.
- [ ] No sidecar file write or canonical metadata mutation occurs before apply
  tasks.

## M3 — Export Apply

Exit criteria:

- [ ] Export apply uses `taru-nfo` round-trip preservation.
- [ ] Sidecar writes are mediated by VFS storage APIs.
- [ ] Backup, atomic replace, and retention diagnostics are recorded.
- [ ] Operator-facing diagnostics redact raw paths and raw XML.

## M4 — Import Authority Apply

Exit criteria:

- [ ] NFO import applies local authority through canonical metadata boundaries.
- [ ] User-locked fields are respected.
- [ ] Accepted field, skipped field, and conflict outcomes are auditable.
- [ ] Hierarchy confirmation is explicit and stale-safe.

## M5 — Rollback And Repair

Exit criteria:

- [ ] Injected partial failures are tested.
- [ ] Failed-before-mutation, rollback-complete, and repair-pending outcomes are
  distinguishable.
- [ ] No failure path claims a false committed state.

## M6 — Closeout

Exit criteria:

- [ ] Fresh validation is recorded.
- [ ] Parent umbrella and follow-on split decisions agree.
- [ ] API/UI/addon exposure work is either opened or explicitly deferred.
