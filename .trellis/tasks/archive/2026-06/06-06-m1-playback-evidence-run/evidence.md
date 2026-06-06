# Evidence

## Summary

- Ran Product-Operator M1 explicit playback gate against `main` after the
  default `fast` and technical `release-fast` gates passed.
- `playback` passed, so no playback/transcode blocker implementation task was
  opened from this run.
- The run generated
  `target/release-gate/playback-hardware-report.json`; this target artifact is
  local release evidence output, not committed task state.

## Verification

- Date: 2026-06-06 20:30 Asia/Shanghai.
- Command:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode playback`
- Result: passed.
- Delegated gate:
  `scripts/release-gate.ps1 -Mode playback`
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.
  - redaction inventory scan wrote 7250 matches to
    `target/release-gate/redaction-inventory.txt`.
  - `ffmpeg -version` passed with FFmpeg 5.1.2 essentials build.
  - `ffprobe -version` passed with FFprobe 5.1.2 essentials build.
  - `cargo check -p nako-transcode -p nako-server --tests` passed.
  - `cargo nextest run -p nako-transcode hardware --no-fail-fast`
    passed: 16 tests passed, 106 skipped.
  - `cargo run -p nako-transcode --example hardware-report -- --ffmpeg ffmpeg --output target/release-gate/playback-hardware-report.json`
    passed and wrote the hardware report.
  - `cargo nextest run -p nako-transcode hls --no-fail-fast`
    passed: 75 tests passed, 47 skipped.
  - `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`
    passed: 1 test passed, 654 skipped.

## Decision

The explicit M1 playback gate is green. Because no FFmpeg/FFprobe,
transcode-hardware, hardware-report, HLS, or server self-host blocker was
exposed, this task does not open a follow-on implementation slice. The next
evidence-driven step is an environment/config gate such as `container` or
`postgres`.
