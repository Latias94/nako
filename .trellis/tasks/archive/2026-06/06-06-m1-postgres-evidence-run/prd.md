# M1 PostgreSQL Evidence Run

## Goal

Run the explicit Product-Operator M1 PostgreSQL evidence gate after the fast,
release-fast, playback, and container evidence runs passed, then classify the
result as passed, failed, or environment-skipped.

## Requirements

- Execute the M1 ladder PostgreSQL mode:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode postgres`
- Record the date, command, result, and failing delegated step if any.
- Check whether the delegated release gate covers only a narrow PostgreSQL
  suite or the documented all-contract PostgreSQL harness.
- If PostgreSQL tooling or a test URL is unavailable, record the missing
  environment dependency as a skip, not as an implementation blocker.
- If the gate fails with a repository-owned schema, migration, adapter, or
  contract issue, classify the blocker under `nako-db` / storage-runtime and
  open a focused follow-on task.
- Keep this task evidence-only unless the gate exposes a concrete blocker that
  needs a focused implementation task.

## Acceptance Criteria

- [x] Trellis context validation passes.
- [x] M1 ladder `postgres` mode result is recorded.
- [x] PostgreSQL harness coverage is classified.
- [x] Any failure or skip is classified by delegated gate and owner lane.
- [x] If the gate passes or is environment-skipped, task evidence states that
      no implementation task was opened from this run.
- [ ] Task is archived and committed.

## Technical Notes

Relevant context:

- `scripts/m1-release-ladder.ps1`
- `scripts/release-gate.ps1`
- `scripts/postgres-contract-harness.ps1`
- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
- `docs/deployment/SELF_HOSTED.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/architecture/LANES.md`
- `.trellis/spec/nako-db/backend/quality-guidelines.md`
