# Android Active Playback Session Cancellation - Handoff

Status: Closed
Last updated: 2026-05-19

## Final State

APSC-020, APSC-030, and APSC-040 are complete. Android smoke has a
`profile-active-remux` state that proves player exit cancels an active remux
session through Public Client API readback.

## Key Constraints

- Android evidence must use Public Client API only.
- Do not use admin diagnostics, server logs, local output paths, or bearer token
  contents as smoke truth.
- Do not change normal Android default playback capabilities just to satisfy
  smoke.
- Dedicated debug fixture state is allowed.

## Evidence Anchors

- `apps/android/build/smoke/20260519-223623-profile-active-remux-emulator-5554/report.md`
- `apps/android/build/smoke/20260519-223623-profile-active-remux-emulator-5554/profile-active-remux-session-cancelled.txt`
- `cargo test -p nako-server remux_stream_route -- --nocapture`
- `cargo test -p nako-transcode remux_runner_kills_and_cleans_temp_output_on_cancel -- --nocapture`

## Residual Follow-Ons

- HLS active cancellation can reuse this pattern when playlist startup becomes
  asynchronous enough to require a dedicated fixture.
- A future regression wrapper can opt into `profile-active-remux` for heavier
  local gates, but the default stable smoke set remains unchanged for speed.
