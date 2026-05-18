# NFO Sidecar Backup Policy Milestones

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream docs define local backup semantics, non-goals, and validation
  gates.
- `docs/GOALS.md` names M49 as the active implementation goal.
- The design keeps XML preservation separate from storage backup mechanics.

## M1 - VFS Local Backup Boundary

Exit criteria:

- VFS write requests can explicitly ask for existing-file backup.
- `LocalFsBackend` creates a same-directory backup before replacing an existing
  file.
- Unsupported backends fail backup requests explicitly.
- Existing direct and atomic write behavior remains compatible.

## M2 - NFO Export Backup Diagnostics

Exit criteria:

- Forced export over an existing sidecar requests backup.
- Fresh sidecar export does not request backup.
- Export summaries expose internal/test-visible backup counts and backup
  reports.
- Backup failure prevents final sidecar replacement.

## M3 - Closeout

Exit criteria:

- Focused `taru-vfs` and `taru-nfo` checks and tests pass.
- Workspace checks and nextest pass.
- `git diff --check` has no whitespace errors.
- `docs/GOALS.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` record M49
  completion evidence and recommended follow-ons.
