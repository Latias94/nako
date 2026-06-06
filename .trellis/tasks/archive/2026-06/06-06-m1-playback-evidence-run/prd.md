# M1 Playback Evidence Run

## Goal

Run the explicit Product-Operator M1 playback gate after the default `fast` and
technical `release-fast` gates passed, then route any concrete playback blocker
to the playback/transcode lane.

## Requirements

- Execute:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode playback`
- Record the date, command, result, and failing delegated step if any.
- If the gate passes, do not open speculative playback implementation work.
- If the gate fails, classify the failure by delegated playback step:
  - FFmpeg/FFprobe availability or version failure: operations-release plus
    playback-transcode packaging/diagnostics.
  - `nako-transcode` compile or hardware tests: playback-transcode.
  - hardware report generation: playback-transcode plus operations-release.
  - HLS tests: playback-transcode.
  - server self-host smoke: control-plane or owning server feature lane.
- Keep this task evidence-only unless the gate exposes a concrete blocker that
  needs a focused follow-on implementation task.

## Acceptance Criteria

- [x] Trellis context validation passes.
- [x] M1 ladder `playback` mode result is recorded.
- [x] Any failure is classified by delegated gate and owner lane.
- [x] If the gate passes, task evidence states that no blocker implementation
      task was opened from this run.
- [x] Task is archived and committed.

## Technical Notes

Relevant context:

- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/LANES.md`
- `.trellis/spec/nako-transcode/backend/index.md`
- `.trellis/spec/nako-server/backend/quality-guidelines.md`
