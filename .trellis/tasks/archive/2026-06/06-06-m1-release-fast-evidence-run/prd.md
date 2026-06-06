# M1 Release Fast Evidence Run

## Goal

Run the technical M1 release-fast gate after the default M1 fast ladder passed,
then route any concrete blocker to the owning lane.

## Requirements

- Execute:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode release-fast`
- Record the date, command, result, and failing delegated step if any.
- If the gate passes, do not open speculative implementation work.
- If the gate fails, classify the failure by delegated release-gate step and
  owner lane:
  - DB or managed-artwork persistence: `nako-db` / storage-vfs.
  - API, SDK, or Admin contract drift: `nako-api` / web-product.
  - Server managed-artwork or self-host smoke: `nako-server` / control-plane or
    owning feature lane.
  - Admin Web generated contract or type-check failure: web-product.
- Keep this task evidence-only unless the gate exposes a concrete blocker that
  needs a focused follow-on implementation task.

## Acceptance Criteria

- [x] Trellis context validation passes.
- [x] M1 ladder `release-fast` mode result is recorded.
- [x] Any failure is classified by delegated gate and owner lane.
- [x] If the gate passes, task evidence states that no blocker implementation
      task was opened from this run.
- [x] Task is archived and committed.

## Technical Notes

Relevant context:

- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/architecture/LANES.md`
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
- `.trellis/spec/nako-api/backend/quality-guidelines.md`
- `.trellis/spec/nako-server/backend/quality-guidelines.md`
