# Playback Release Gate Mode Evidence

Date: 2026-06-04

## Scope

- Added explicit `playback` mode to `scripts/release-gate.ps1` and
  `scripts/release-gate.sh`.
- Playback mode checks FFmpeg and FFprobe availability.
- Playback mode reuses existing transcode HLS and server self-host playback
  smoke gates.
- No server runtime, public API, Admin DTO, generated SDK, schema, or live GPU
  smoke requirement was added.

## Verification

- `ffmpeg -version`: passed on local host, version
  `5.1.2-essentials_build-www.gyan.dev`.
- `ffprobe -version`: passed on local host, version
  `5.1.2-essentials_build-www.gyan.dev`.
- `bash -n scripts/release-gate.sh`: passed after preserving LF line endings.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs -SkipRedactionInventory`:
  passed.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode playback -SkipRedactionInventory`:
  passed.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed with LF/CRLF normalization warnings only.
  - `cargo check -p nako-transcode -p nako-server --tests`: passed.
  - `cargo nextest run -p nako-transcode hls --no-fail-fast`: passed, 73
    tests.
  - `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`: passed,
    1 test.
- `python .\.trellis\scripts\task.py validate 06-04-06-04-playback-release-gate-mode-first-slice`:
  passed.

## Documentation Sync

- `docs/deployment/RELEASE_CHECKLIST.md` documents the playback release gate.
- `docs/deployment/SELF_HOSTED.md` lists playback mode beside fast/postgres
  local release confidence checks.
- `docs/architecture/OPERATIONS_RELEASE.md` records the shipped playback mode
  foundation and leaves GPU hardware matrix diagnostics as a follow-on.
