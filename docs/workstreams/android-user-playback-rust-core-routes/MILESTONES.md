# Android User Playback Rust Core Routes — Milestones

Status: Closed
Last updated: 2026-05-21

## M0 — Scope Freeze

Exit criteria:

- Scope targets User Playback State route construction only.
- Non-goals keep DTO decode, body DTO ownership, transport, UI, and server API
  shape out of scope.
- ADR 0028 and ADR 0032 remain authoritative.

## M1 — Rust Core User Playback Builders

Exit criteria:

- `nako-client-core` has explicit builders for the four Android user-playback
  routes.
- Builders produce `CoreHttpRequest` with method, auth, safe preview, item ID
  encoding, pagination, JSON content type on writes, and optional body passthrough.
- Core tests cover stable URLs and headers for representative read/write routes.

## M2 — UniFFI User Playback Surface

Exit criteria:

- `nako-client-uniffi` exposes thin user-playback request builder bindings.
- Boundary guard still passes.
- UniFFI tests cover at least continue watching and a write route.

## M3 — Android User Playback Migration

Exit criteria:

- `NakoUserPlaybackClient` runtime route construction uses
  `UserPlaybackCore`/Rust core.
- Generated SDK DTO/body mapping and Android diagnostics remain unchanged.
- User-playback JVM tests pass.

## M4 — Integration Verification And Docs

Exit criteria:

- README/workstream docs explain the new route ownership.
- Combined Rust, UniFFI, guard, route-owner scan, and Android user-playback
  gates pass.

## M5 — Closeout

Exit criteria:

- Closeout records final gates, residual risks, and follow-ons.
- Workstream JSON and markdown agree on closed state.
