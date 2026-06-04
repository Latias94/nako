# Evidence

## Changes

- Added `crates/nako-transcode/examples/hardware-report.rs`, a thin release
  evidence producer that serializes the existing
  `FfmpegHardwareAccelerationDetector` report to JSON.
- Updated `scripts/release-gate.sh` and `scripts/release-gate.ps1` so playback
  mode writes `target/release-gate/playback-hardware-report.json`.
- Updated `docs/architecture/OPERATIONS_RELEASE.md`,
  `docs/deployment/RELEASE_CHECKLIST.md`, and
  `docs/deployment/SELF_HOSTED.md` to describe the shipped report baseline and
  keep container device pass-through as a follow-on.
- Updated `.trellis/spec/nako-transcode/backend/directory-structure.md` to
  record the `hardware-report` example boundary.

## Validation

- `cargo fmt --all`
- `cargo check -p nako-transcode --examples --tests`
- `cargo run -p nako-transcode --example hardware-report -- --ffmpeg ffmpeg --output target/release-gate/playback-hardware-report.json`
- `rg -n "[A-Za-z]:\\\\|/Users/|/home/|token|secret|password|managed-artwork://|source_uri|cache_uri|storage_uri" target/release-gate/playback-hardware-report.json`
  returned no matches.
- `cargo nextest run -p nako-transcode hardware --no-fail-fast` passed: 16
  tests.
- `bash -n scripts/release-gate.sh`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode playback -SkipRedactionInventory`
  passed. It exercised `cargo check -p nako-transcode -p nako-server --tests`,
  `cargo nextest run -p nako-transcode hardware --no-fail-fast`, the new
  hardware-report example step, `cargo nextest run -p nako-transcode hls
  --no-fail-fast`, and `cargo nextest run -p nako-server self_host_smoke
  --no-fail-fast`.

## Notes

- The generated report is under `target/`, which is gitignored.
- The default playback release gate still does not require GPU devices. It
  records host FFmpeg capability evidence and leaves one-frame GPU smoke and
  container device pass-through as explicit follow-ons.
