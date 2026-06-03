# Current release gate shape

- Query: smallest useful playback release-gate mode.
- Scope: release gate scripts and deployment docs.
- Date: 2026-06-04.

## Findings

- `scripts/release-gate.ps1` and `scripts/release-gate.sh` both support modes
  `docs`, `fast`, `db`, `api`, `postgres`, `container`, `workspace`, and `all`.
- Both scripts run `cargo fmt --all -- --check`, `git diff --check`, and an
  optional redaction inventory before mode-specific work.
- The API/fast path already includes `cargo nextest run -p nako-server
  self_host_smoke --no-fail-fast`, but that path also runs broad API/SDK checks.
- Neither script has a focused playback release mode or explicit FFmpeg/FFprobe
  presence check.
- `docs/architecture/OPERATIONS_RELEASE.md` calls out playback release hardware
  matrix work, FFmpeg/ffprobe presence, CPU fallback smoke, and optional
  hardware diagnostics.
- `docs/deployment/SELF_HOSTED.md` and `docs/deployment/RELEASE_CHECKLIST.md`
  already explain FFmpeg/FFprobe as runtime prerequisites but do not name a
  focused playback release gate.

## Recommended first slice

Add `playback` mode to both release-gate scripts:

- require `ffmpeg -version` and `ffprobe -version`;
- run focused transcode HLS coverage;
- run focused server playback/self-host smoke coverage;
- keep GPU hardware smoke optional and documented as a follow-on;
- leave default `fast` behavior unchanged.

## Verification candidates

- PowerShell playback gate with redaction inventory skipped.
- Bash script syntax parse or help/mode validation on Windows.
- Rust focused tests invoked by the gate.
- `git diff --check` and Trellis validate.
