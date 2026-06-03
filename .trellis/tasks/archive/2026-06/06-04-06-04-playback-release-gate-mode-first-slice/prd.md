# Playback release gate mode first slice

## Goal

Add an explicit playback release-gate mode so operators and release builders can
verify FFmpeg/FFprobe availability plus the existing playback/transcode smoke
coverage before claiming a host or package is playback-ready.

## What I Already Know

* `docs/architecture/OPERATIONS_RELEASE.md` lists playback release hardware
  matrix work as a next lane.
* `scripts/release-gate.ps1` and `scripts/release-gate.sh` currently support
  `docs`, `fast`, `db`, `api`, `postgres`, `container`, `workspace`, and `all`.
* The fast/API gate already runs `nako-server self_host_smoke`, but there is no
  focused playback mode or direct FFmpeg/FFprobe presence check.
* This machine has `ffmpeg` and `ffprobe` available, so the new mode can be
  smoke-tested locally.

## Assumptions

* The first slice should be script-level and documentation-level. It should not
  add server routes, Admin DTOs, generated SDK output, or runtime behavior.
* Hardware GPU smoke remains optional and operator-run; the gate should not
  require VAAPI/NVENC/QSV devices.
* The gate should fail clearly if FFmpeg or FFprobe is missing, because playback
  release readiness depends on those tools.

## Requirements

* Add `playback` as a supported mode to both release-gate scripts.
* In playback mode, check FFmpeg and FFprobe availability before Rust playback
  gates run.
* Reuse existing focused gates rather than inventing new live media fixtures.
* Keep docs mode unchanged and keep fast/all behavior compatible.
* Update release docs to show how to run the playback release gate.
* Add or update evidence so the task proves the scripts parse and run locally.

## Acceptance Criteria

* [x] `scripts/release-gate.ps1 -Mode playback -SkipRedactionInventory` runs the
  playback gate on Windows.
* [x] `scripts/release-gate.sh --mode playback --skip-redaction-inventory` is
  syntactically valid and documents the new mode.
* [x] FFmpeg/FFprobe presence is checked in the playback mode.
* [x] Playback mode runs focused playback/transcode smoke tests already present
  in the repo.
* [x] Release docs mention the playback gate.
* [x] `cargo fmt --all -- --check`, `git diff --check`, Trellis validate, and
  focused script checks pass.

## Definition Of Done

* Code/docs are committed with a Conventional Commit message.
* Verification evidence is persisted in this task directory.
* Relevant architecture or release docs are updated.
* Task is archived and the developer journal is recorded.

## Out Of Scope

* No live GPU smoke requirement.
* No Docker hardware pass-through implementation.
* No server/Admin/Public API or generated SDK change.
* No new media fixture or runtime playback route.
* No broad workspace release gate rewrite.

## Technical Approach

* Add `playback` to mode validation in both scripts.
* Add a small playback gate function in both scripts:
  * `ffmpeg -version`;
  * `ffprobe -version`;
  * `cargo check -p nako-transcode -p nako-server --tests`;
  * focused `cargo nextest` for transcode HLS and server playback smoke.
* Wire the function to `playback` and `all`, leaving `fast` unchanged.
* Update `docs/deployment/RELEASE_CHECKLIST.md`,
  `docs/deployment/SELF_HOSTED.md`, and
  `docs/architecture/OPERATIONS_RELEASE.md`.

## Research References

* [`research/current-release-gate-shape.md`](research/current-release-gate-shape.md)
  - current script modes and bounded playback mode recommendation.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/index.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/nako-transcode/backend/index.md`
  * `.trellis/spec/nako-transcode/backend/quality-guidelines.md`
