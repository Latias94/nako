# Architecture Map Staleness Notes

## Findings

- `docs/architecture/LANES.md` still says the storage-vfs lane should consider
  "source fingerprint hash operator/Admin diagnostics" even though source hash
  Admin overview diagnostics, Jobs drill-down filters, Admin manual enqueue,
  and source-hash-specific retry/requeue have shipped.
- `docs/architecture/CONTROL_PLANE.md` source fingerprint hash follow-ons still
  bundle "operator/API surfaces" too broadly. The remaining work is policy and
  automation: scan-originated enqueue, automatic reconciliation, duplicate
  suggestion/mutation policy, and broader scheduler migration. Read-only Admin
  diagnostics, manual enqueue, and source-hash-specific retry/requeue are no
  longer missing.
- `docs/architecture/STORAGE_VFS.md` was updated by the VFS cache remediation
  slice to record the shipped read-only plan. This task should verify its task
  link points at the archived task path and its follow-on lane is durable repair
  queues/workers rather than first-slice remediation planning.

## Source Evidence

- `.trellis/tasks/archive/2026-06/06-05-06-05-cross-lane-architecture-audit/research/storage-library-control-plane.md`
  lists source hash Admin diagnostics as already shipped and calls out
  `LANES.md` staleness.
- `.trellis/tasks/archive/2026-06/06-05-06-05-cross-lane-architecture-audit/research/next-lane-synthesis.md`
  ranks Public Client playback parity as the top serial gate but also allows
  low-conflict architecture-map reconciliation.
- `bf56c38c feat(storage): add vfs cache remediation plan` shipped read-only
  VFS cache remediation planning and archived the Trellis task at
  `.trellis/tasks/archive/2026-06/06-06-vfs-cache-repair-non-destructive-remediation-plan-first-slice/`.
- `.trellis/tasks/archive/2026-06/06-06-admin-source-fingerprint-hash-trigger-first-slice/`
  shipped Admin manual source hash enqueue.
- `.trellis/tasks/archive/2026-06/06-06-source-hash-retry-requeue-admin-command/`
  shipped source-hash-specific Admin retry/requeue.

## Docs-only Boundary

Do not edit production Rust, TypeScript, generated contracts, schema, config,
or API docs in this task. This is a planning-map accuracy slice only.
