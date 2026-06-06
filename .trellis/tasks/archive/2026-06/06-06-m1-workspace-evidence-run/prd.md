# M1 Workspace Evidence Run

## Goal

Run the explicit Product-Operator M1 workspace evidence gate after fast,
release-fast, playback, container, and PostgreSQL evidence passed, then
classify the result as passed, failed, or environment-skipped.

## Requirements

- Execute:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode workspace`
- Record the date, command, result, and failing delegated step if any.
- If the gate fails with a repository-owned compile or test regression,
  classify the first failure by owning crate/lane and fix it in this task when
  the change is a narrow release-gate stability repair.
- Keep this task evidence-focused unless the gate exposes a concrete blocker
  that needs a narrow implementation fix.
- Do not commit raw workspace test logs that include local absolute paths; keep
  task evidence to public summaries.

## Acceptance Criteria

- [x] Trellis context validation passes.
- [x] M1 ladder `workspace` mode result is recorded.
- [x] Any failure or skip is classified by delegated gate and owner lane.
- [x] If the initial gate fails with release-gate instability, the focused fix
      and verification are recorded.
- [x] If the final gate passes, task evidence states that no separate
      implementation task was opened from this run.
- [ ] Task is archived and committed.

## Technical Notes

Relevant context:

- `scripts/m1-release-ladder.ps1`
- `scripts/release-gate.ps1`
- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
- `docs/deployment/SELF_HOSTED.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/architecture/LANES.md`
- `.trellis/spec/nako/backend/quality-guidelines.md`
- `.trellis/spec/nako-server/backend/quality-guidelines.md`
- `.trellis/spec/nako-transcode/backend/quality-guidelines.md`
