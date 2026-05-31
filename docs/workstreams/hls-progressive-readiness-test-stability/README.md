# HLS Progressive Readiness Test Stability

Status: Closed
Last updated: 2026-05-31

This workstream owns the HLS progressive readiness gate instability split from
`docs/workstreams/hls-runtime-lifecycle-boundary/HRLB-040`.

The immediate problem is not PAIP, LL-HLS/CMAF, remote workers, player UX, DTO
shape, schema, or VFS behavior. The problem is that the full HLS nextest gate
fails under default suite concurrency on progressive readiness tests that pass
individually.

Current task: none. The workstream closed after `HPRTS-030`.

Closeout summary:

- `HPRTS-020` classified the instability as Windows process-backed test timing
  under default nextest concurrency.
- The fix is test-only: the two target progressive readiness tests now use a
  named readiness timeout that is 180s on Windows and 60s elsewhere.
- `HPRTS-030` reran the default full HLS gate successfully, unblocking
  `HRLB-040` closeout.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
