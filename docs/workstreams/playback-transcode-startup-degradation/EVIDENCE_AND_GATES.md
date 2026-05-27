# Playback Transcode Startup Degradation - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Gate Commands

- `python -m json.tool docs/workstreams/playback-transcode-startup-degradation/WORKSTREAM.json`
- `cargo nextest run -p nako-server hls_service --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_playback_runtime --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Log

### PTSD-010

Status: Complete
Date: 2026-05-27

Evidence:

- Added ADR 0048.
- Created workstream docs and task ledger.

Validation:

- `python -m json.tool docs/workstreams/playback-transcode-startup-degradation/WORKSTREAM.json`

### PTSD-020

Status: Complete
Date: 2026-05-27

Evidence:

- `HlsAppService` stores `TranscodePipelineReadiness` and an optional
  `TranscodePipelinePlan`.
- Startup succeeds when the configured HLS pipeline is unavailable.
- HLS execution policy planning still rejects unavailable transcode before
  FFmpeg spawn.

Validation:

- `cargo nextest run -p nako-server hls_service --no-fail-fast`

### PTSD-030

Status: Complete
Date: 2026-05-27

Evidence:

- Admin playback runtime diagnostics report unavailable HLS readiness without
  requiring an executable startup plan.
- `selected_hls_slots` is zero when no executable HLS plan exists.
- Selected fallback readiness is unavailable when the pipeline is unavailable.

Validation:

- `cargo nextest run -p nako-server admin_v1_playback_runtime --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`

### PTSD-040

Status: Complete
Date: 2026-05-27

Evidence:

- Focused playback gate passed after the startup degradation refactor.

Validation:

- `cargo fmt --all -- --check`
- `git diff --check`

## Review Checks

- [x] HLS unavailability does not block admin/runtime startup.
- [x] HLS execution does not silently fall back when no executable plan exists.
- [x] Admin diagnostics keep typed readiness and bounded evidence.
- [x] Direct play/remux semantics remain untouched.
