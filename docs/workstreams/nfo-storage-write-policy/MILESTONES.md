# NFO Storage Write Policy Milestones

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream docs define safe local write semantics, non-goals, and validation
  gates.
- `docs/GOALS.md` names M48 as the active implementation goal.
- The design keeps XML preservation separate from VFS write mechanics.

## M1 - VFS Atomic Local Write Boundary

Exit criteria:

- VFS exposes an explicit write request/report path.
- `LocalFsBackend` supports atomic replace using same-directory temp file and
  rename.
- Unsupported backends fail atomic replace explicitly.
- Direct `write_string` compatibility remains intact.

## M2 - NFO Export Uses Safe Write Policy

Exit criteria:

- NFO export writes sidecars through the explicit write policy path.
- Local sidecar export requests atomic replace.
- Export failures carry test-visible diagnostic categories.
- M47 preservation tests continue to pass.

## M3 - Closeout

Exit criteria:

- Focused `taru-vfs` and `taru-nfo` checks and tests pass.
- Workspace checks and nextest pass.
- `git diff --check` has no whitespace errors.
- `docs/GOALS.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` record M48
  completion evidence and recommended follow-ons.
