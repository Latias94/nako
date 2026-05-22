# Phase 25.1: Transcode Runtime Productization Slice

## Summary

This slice moves M25 from a proposed workstream into runtime code. Playback
orchestration is split into focused app modules, HLS hardware selection now uses
FFmpeg encoder capability evidence, and selected acceleration determines the
runtime budget used by HLS sessions.

The implementation keeps Nako's current single-variant HLS behavior. It does
not add an adaptive bitrate ladder, client UI, or direct remote FFmpeg input.

## Code Changes

- Replaced the monolithic `crates/nako-server/src/app/playback.rs` with the
  `crates/nako-server/src/app/playback/` module tree.
- Moved direct-play planning and VFS streaming response bodies into
  `playback/direct.rs`.
- Moved FFmpeg input resolution, remote staging, manifest recording, and staging
  lease release into `playback/input.rs`.
- Moved remux session admission, persisted reuse, process execution, failure
  persistence, cancellation, and finished-session events into
  `playback/remux.rs`.
- Moved HLS session admission, selected acceleration, runtime guard creation,
  HLS command planning, failure persistence, cancellation, and finished-session
  events into `playback/hls.rs`.
- Added `LibraryStorageBackend::clone_backend` so FFmpeg input staging can wrap
  the selected storage backend with manifest-recording behavior without
  re-resolving library identity.
- Added `FfmpegHardwareAccelerationDetector` in `nako-transcode` to probe
  `ffmpeg -hide_banner -encoders` and build a hardware capability report.
- Added FFmpeg encoder parsing for VAAPI (`h264_vaapi`), NVENC (`h264_nvenc`),
  and QuickSync/QSV (`h264_qsv`).
- HLS service construction now selects hardware acceleration from the FFmpeg
  report and chooses CPU or GPU transcode slots from
  `TranscodeResourceBudget::slots_for`.

## Runtime Contract

HLS and remux requests now share the same explicit session lifecycle:

- `Planned`: a persisted transcode session has been admitted.
- `Running`: FFmpeg execution has started.
- `Finished`: output exists and can be reused by later matching requests.
- `Failed`: planning, runner, timeout, stale startup, or storage failure was
  persisted with a failure category and message.
- `Cancelled`: the runner observed cancellation and persisted cancellation.

Admission rules:

- A persisted active remux or HLS session for the same request key rejects a
  duplicate request with `Conflict`.
- A finished session with the expected output path is reused.
- Startup marks stale active transcode sessions as failed with the `Stale`
  category.
- HLS segments are served only after the owning HLS session is `Finished`.

Hardware rules:

- `hardware_acceleration = none` selects CPU without requiring GPU capability.
- A requested GPU accelerator selects that accelerator only when the FFmpeg
  encoder report marks it available.
- `hardware_fallback = cpu` falls back to CPU when the requested accelerator is
  unavailable.
- `hardware_fallback = fail` rejects app startup when the requested accelerator
  is unavailable.
- A failed FFmpeg probe keeps CPU available and marks GPU accelerators
  unavailable with a diagnostic reason.

## Validation Evidence

Validation run for this slice:

```powershell
cargo check -p nako-transcode --tests
cargo check -p nako-server --tests
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-server app::tests::playback --no-fail-fast
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run -p nako-server --no-fail-fast
cargo nextest run --workspace --no-fail-fast
git diff --check
```

Results:

- `cargo nextest run -p nako-transcode --no-fail-fast`: 21 tests passed.
- `cargo nextest run -p nako-server --no-fail-fast`: 90 tests passed.
- `cargo nextest run --workspace --no-fail-fast`: 231 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Known Follow-Ups

- HLS remains single-variant. Adaptive bitrate ladder planning belongs after
  the runtime contract is stable.
- Cancellation is implemented in the process runner and persisted by app
  services, but there is no public cancellation HTTP endpoint yet.
- Hardware probing currently uses encoder names as capability evidence. Device
  initialization checks can be added later if operators need stricter
  diagnostics.
- HTTP API shapes did not change in this slice, so public API docs do not need
  new DTO sections yet.
