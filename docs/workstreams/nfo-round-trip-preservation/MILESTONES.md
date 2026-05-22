# NFO Round Trip Preservation Milestones

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream docs define preservation semantics, non-goals, and validation
  gates.
- `docs/GOALS.md` names M47 as the active implementation goal.
- NFO preservation remains separate from VFS link/write policy.

## M1 - Preservation-Aware Codec

Exit criteria:

- `nako-nfo` exposes a preservation-aware update path for existing movie NFO
  XML.
- Nako-owned fields are rendered from `NfoDocument`.
- Unknown top-level XML elements survive the update.
- Duplicate or alias-owned fields are reported as conflicts in a structured
  report.
- Existing parse/render behavior for newly generated movie NFO still passes.

## M2 - Forced Export Uses Preservation

Exit criteria:

- `NfoService::export_source` reads the existing sidecar when `force` is true
  and the sidecar already exists.
- Existing sidecar update uses the preservation-aware codec path.
- Creating a missing sidecar still uses deterministic fresh rendering.
- Export failures remain item-level failures in the existing summary model.

## M3 - Closeout

Exit criteria:

- Focused `nako-nfo` checks and tests pass.
- Workspace checks and nextest pass.
- `git diff --check` has no whitespace errors.
- `docs/GOALS.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` record M47
  completion evidence and recommended follow-ons.

Completion notes:

- M0 through M3 are complete.
- `nako-nfo` focused checks and workspace checks passed.
- Remaining NFO compatibility and storage write-policy work is intentionally
  outside M47.
