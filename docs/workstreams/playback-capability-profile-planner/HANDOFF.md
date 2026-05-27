# Playback Capability Profile Planner - Handoff

Status: Completed
Last updated: 2026-05-27

## Current State

The profile-driven **Playback Capability Planner** is implemented.

Shipped state:

- ADR 0044 records the decision.
- `crates/nako-playback/src/capability.rs` owns target profiles,
  compatibility evaluations, and decision reports.
- `PlaybackPlanner` evaluates direct play, remux, and transcode through
  profile capability reports.
- Public Client API exposes safe decision reports without Source Locators or
  FFmpeg details.
- Kotlin and TypeScript SDK package entries were regenerated.
- Server playback decision routes assert report visibility and redaction.

## Completed Gates

Final gates passed on 2026-05-27:

- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

Additional confidence gates also passed:

- `cargo nextest run -p nako-client-protocol --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo check -p nako-client`
- `cargo check -p nako-client-core`
- `cargo check -p nako-client-uniffi`

## Guardrails

- Do not copy Jellyfin code, comments, tests, or schema shapes.
- Keep FFmpeg command planning in `nako-transcode`.
- Keep server playback orchestration as an adapter around the pure planner.
- Public Client DTOs must not expose local Source Locators, FFmpeg command
  strings, or runtime host paths.

## Open Risks

- `ClientPlaybackCapabilities` remains as the narrow Public Client request
  shape, but it is now an adapter into `PlaybackTargetProfile`.
- Concrete FFmpeg hardware decode, subtitle/HDR, and HLS breadth work is split
  into `FOLLOW_ONS.md`.
