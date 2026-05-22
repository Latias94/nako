# NFO Link Authority — Handoff

Status: Complete
Last updated: 2026-05-21

## Current State

The lane is active as the next post-RPD mainline after metadata provider
breadth. Existing NFO preservation/write/backup/retention work is treated as
baseline. Non-destructive VFS link planning is implemented. Filesystem-link
evidence can be recorded as suggested Source Duplicate Relationship
diagnostics without merging Media Sources or Media Items. NFO authority preview
now explains import/export decisions without writing sidecars.

## Final State

- Task ID: LNA-060
- Owner: planner
- Files: `docs/workstreams/nfo-link-authority`
- Validation: evidence gates are fresh and follow-ons are explicit
- Status: DONE
- Evidence: LNA-020 link planning, LNA-030 source duplicate diagnostics, and
  LNA-040 NFO authority preview are complete

## Decisions

- Link behavior belongs in VFS/storage, not in `nako-nfo`.
- Link dry-run must not create targets or modify existing files.
- Hard/soft link apply is deliberately deferred until apply/rollback/audit
  semantics are designed.
- Source duplicate evidence must not merge Media Sources or Media Items.
- VFS link planning returns typed diagnostics only; it does not create links.
- Filesystem-link duplicate evidence stores a redacted diagnostic string
  (`scheme`, `kind`, `status`) instead of raw OS paths.
- NFO authority preview is intentionally non-mutating: it computes create,
  skip, update, backup-required, policy-rejected, and failure decisions before
  sidecar writes.

## Blockers

- None.

## Follow-Ons

- `managed-import-staging`: design import/download promotion state before any
  library mutation.
- `link-apply-and-import-promotion` or equivalent sub-lane: implement actual
  hardlink/symlink creation only after promotion, rollback, cleanup, redacted
  audit reports, and source duplicate confirmation semantics are designed.
- Admin read model: expose NFO authority preview and Source Duplicate
  Relationship diagnostics when operator review UX is ready.

## Next Recommended Action

- Return to `post-rpd-product-hardening` and choose the next mainline lane.
  Recommended: `managed-import-staging`, because local authority diagnostics
  are now strong enough to design download/import promotion without unsafe
  writes.
