# Storage VFS Cache Repair Diagnostics

## Goal

Add or deepen diagnostics for VFS cache repair behavior without mixing playback
artifact I/O policy or unrelated scan scheduling.

## Requirements

* Preserve ADR 0016 remote storage and VFS cache boundaries.
* Keep diagnostics redacted, bounded, and operator-useful.
* Separate cache repair/source identity behavior from HLS artifact pressure.
* Avoid schema or API expansion until diagnostics requirements are confirmed by
  existing architecture evidence.
* Add tests for repair classification and redaction where behavior changes.

## Acceptance Criteria

* [ ] VFS cache repair diagnostics expose clear state/failure categories.
* [ ] Diagnostics do not leak credentials, raw provider secrets, or unstable
      local paths beyond existing policy.
* [ ] Tests cover at least one success and one failure/repair classification.
* [ ] Storage/VFS architecture notes are updated if diagnostic semantics become
      durable.

## Definition of Done

* Scoped nextest run passes for VFS/library/server areas touched.
* No playback runtime artifact policy is added here.
* No broad storage refactor without a narrower follow-on task.

## Worktree

Suggested path: `E:\Rust\nako-worktrees\01e-vfs-cache-repair-diagnostics`

Suggested branch: `task/01e-vfs-cache-repair-diagnostics`

Conflict note: coordinate with playback only around shared staging/source read
contracts.
