# HRLB-020 Lifecycle Tests - 2026-05-31

## Scope

Task: `HRLB-020`

Goal: add focused HLS lifecycle invariant tests and only introduce a
behavior-preserving server-local lifecycle coordinator/facade if HRLB-010
justified it.

## Changes

- Added `hls_source_timeout_fails_session_and_cleans_visible_output` for HLS
  runner timeout mapping, persisted `Timeout` failure category, operator
  message, and serve-visible output cleanup.
- Added `app_startup_marks_stale_hls_transcode_sessions_failed` for HLS-specific
  startup stale-session recovery.
- Added HLS remote staged-input release coverage across:
  - successful HLS completion;
  - runner failure;
  - playlist admission rejection before background start.
- Added local test helpers for remote WebDAV HLS app setup and released
  `FfmpegInput` staging manifest assertions.

No coordinator/facade was added. The existing server-local HLS lifecycle
surface was sufficient for the behavior-preserving coverage slice.

## Validation

```text
cargo nextest run -p nako-server hls --no-fail-fast
```

Result: initial run was blocked before tests ran because
`crates/nako-server/src/api_mapping.rs:134` did not cover
`HardwarePipelineStage::ToneMap` and
`HardwarePipelineStage::SubtitleBurnIn`. Planner accepted a narrow scope-out
fix to map those stages through `AdminHardwarePipelineStage`.

After the mapping fix, one full HLS run reached 69/70 and hit an existing
load-sensitive progressive-readiness timeout in
`http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`.
That test passed individually, then the final full HLS rerun passed 70/70.

```text
cargo fmt --all -- --check
git diff --check
```

Result: passed. `git diff --check` emitted only Git line-ending normalization
warnings for touched files.

## Handoff

Status: DONE_WITH_CONCERNS

Next: assign `HRLB-030` to split PAIP artifact I/O pressure, resource admission
unification, remote workers, LL-HLS/CMAF, player UX, and HLS test stability
into bounded follow-ons or explicit deferrals.
