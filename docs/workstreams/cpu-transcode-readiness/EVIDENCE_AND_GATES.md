# CPU Transcode Readiness - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Gate Commands

- `python -m json.tool docs/workstreams/cpu-transcode-readiness/WORKSTREAM.json`
- `cargo nextest run -p nako-transcode --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Log

### CTR-010

Status: Complete
Date: 2026-05-27

Evidence:

- Added ADR 0047.
- Created workstream docs and task ledger.

Validation:

- `python -m json.tool docs/workstreams/cpu-transcode-readiness/WORKSTREAM.json`

### CTR-020

Status: Complete
Date: 2026-05-27

Evidence:

- Probe-derived CPU capability now records required `libx264` and `aac` encode
  stages.
- Missing required software encoders makes `HardwareAcceleration::None`
  unavailable.
- CPU capability evidence remains bounded to stage capability names and typed
  discovery status.

Validation:

- `cargo nextest run -p nako-transcode --no-fail-fast`

### CTR-030

Status: Complete
Date: 2026-05-27

Evidence:

- Explicit CPU HLS planning rejects an unavailable software pipeline.
- Hardware fallback-to-CPU rejects when the CPU fallback path is unavailable.
- Admin playback readiness maps `software_pipeline_unavailable` and
  `cpu_fallback_unavailable`.

Validation:

- `cargo nextest run -p nako-transcode --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`

### CTR-040

Status: Complete
Date: 2026-05-27

Evidence:

- Server fake FFmpeg probe scripts include `libx264` and `aac` when tests expect
  CPU readiness.
- Admin playback runtime diagnostics still report typed CPU fallback readiness.

Validation:

- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Review Checks

- [x] Probe-derived CPU availability comes from software encoder facts.
- [x] Static test fixtures remain explicit when they bypass FFmpeg probing.
- [x] Hardware fallback-to-CPU no longer hides CPU unavailability.
- [x] Public Client API remains hardware-redacted.
