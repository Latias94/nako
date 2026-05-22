# Android Playback Residual Rust Core Routes — Milestones

Status: Closed
Last updated: 2026-05-21

## M0 — Scope Freeze

Exit criteria:

- Scope targets residual playback route construction only.
- Non-goals keep DTO decode, transport, UI, and server API shape out of scope.
- Cleanup target is limited to confirmed-dead compatibility helpers.

## M1 — Rust Core Residual Playback Builders

Exit criteria:

- `nako-client-core` has explicit builders for source probe, playback session
  inspect, and playback session cancel.
- Builders produce `CoreHttpRequest` with auth, safe preview, method, and path
  encoding.
- Core tests cover representative read/write residual playback routes.

## M2 — UniFFI Residual Playback Surface

Exit criteria:

- `nako-client-uniffi` exposes thin residual playback request builder bindings.
- Boundary guard still passes.
- UniFFI tests cover source probe and session cancel.

## M3 — Android Playback Migration And Cleanup

Exit criteria:

- `NakoPlaybackClient` residual runtime route construction uses
  `PlaybackCore`/Rust core.
- Generated SDK DTO decode and Android diagnostics remain unchanged.
- Confirmed dead compatibility helpers are removed.
- Playback JVM tests pass.

## M4 — Integration Verification And Docs

Exit criteria:

- README/workstream docs explain the completed playback runtime route ownership.
- Combined Rust, UniFFI, guard, route-owner scan, dead-helper scan, and Android
  playback gates pass.

## M5 — Closeout

Exit criteria:

- Closeout records final gates, residual risks, and follow-ons.
- Workstream JSON and markdown agree on closed state.
