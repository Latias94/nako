# Phase 7.4: Remote Playback Resource Budgets

Status: completed for M7.

## Goal

Give remote direct-play streaming and remote input staging independent resource
budgets so remote playback cannot consume the generic scan/probe/transcode
limits.

## Implemented Shape

- Added `[playback].remote_stream_concurrency`.
- Added `[playback].remote_stage_concurrency`.
- `NakoApp` owns independent semaphores for remote streams and remote staging.
- Remote direct-play acquires a stream permit before opening the backend body
  stream and holds it until the response body plan is dropped.
- Probe and FFmpeg input staging acquire a stage permit around the staging
  operation.

## Validation

- `cargo nextest run -p nako-server direct_play_holds_remote_stream_budget_until_body_is_dropped manifest_recording_backend_waits_for_stage_budget config_round_trips_from_toml config_uses_default_runtime_settings direct_stream_response_proxies_vfs_body_stream`

## Remaining Gaps

- Route-level stress tests should cover concurrent HTTP direct streams and
  concurrent remux/HLS staging requests as post-M7 hardening.
- Cleanup still does not have a separate resource class because M7 cleanup is
  startup-driven rather than a concurrent worker.
