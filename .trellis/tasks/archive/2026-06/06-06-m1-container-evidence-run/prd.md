# M1 Container Evidence Run

## Goal

Run the explicit Product-Operator M1 container/config gate after `fast`,
`release-fast`, and `playback` evidence passed, then classify the result as
passed, failed, or environment-skipped.

## Requirements

- Execute:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode container`
- Record the date, command, result, and failing delegated step if any.
- If Docker/Compose is unavailable, record the missing environment dependency
  as a skip, not as an implementation blocker.
- If the gate fails with a repo-owned config or compose issue, classify the
  blocker under operations-release and open a focused follow-on task.
- Keep this task evidence-only unless the gate exposes a concrete blocker that
  needs a focused implementation task.

## Acceptance Criteria

- [x] Trellis context validation passes.
- [x] M1 ladder `container` mode result is recorded.
- [x] Any failure or skip is classified by delegated gate and owner lane.
- [x] If the gate passes or is environment-skipped, task evidence states that
      no implementation task was opened from this run.
- [x] Task is archived and committed.

## Technical Notes

Relevant context:

- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
- `docs/deployment/RELEASE_CHECKLIST.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/LANES.md`
- `.trellis/spec/nako-server/backend/quality-guidelines.md`
