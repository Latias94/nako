# Playback Release Hardware Map Reconciliation

## Goal

Reconcile active playback architecture maps after the playback release hardware
report task shipped host hardware report evidence.

## Requirements

- Update `docs/architecture/PLAYBACK.md` so release/packaging no longer points
  at the already shipped FFmpeg/hardware matrix report as the next lane.
- Narrow remaining playback hardware follow-ons to true future work:
  hardware tone-map execution, HEVC/AV1 FFmpeg execution, one-frame GPU smoke,
  container device pass-through, and broader player/client packaging evidence.
- Update `docs/architecture/LANES.md` if needed so lane guidance no longer
  implies Admin/release hardware reporting is still open.
- Do not change Rust code, release scripts, deployment docs, or behavior.

## Acceptance Criteria

- [x] Active architecture docs reference the shipped hardware report evidence.
- [x] Active architecture docs keep true follow-ons visible and narrow.
- [x] Focused grep finds no stale `FFmpeg/hardware matrix packaging gate` or
      `Admin/release reporting` wording in active architecture maps.
- [x] `git diff --check` and Trellis task validation pass.

## Definition Of Done

- Docs are updated with exact shipped/follow-on boundaries.
- No code or script changes are included.
- Task is validated, committed, and archived.

## Technical Notes

- Shipped evidence commits:
  - `0ff33be8 feat(release): emit playback hardware report evidence`
  - `9ac17116 chore(task): archive 06-05-06-05-playback-release-hardware-report`
- Likely files:
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/LANES.md`
