# Playback Capability Profile And Rendition Planning Handoff

Status: Completed
Last updated: 2026-05-28

## Current State

The lane is closed. PCPR-010 through PCPR-040 are complete.

Shipped implementation:

- Replaced `PlaybackDecision.execution` with `PlaybackDecision.rendition`.
- Deleted duplicate top-level `direct_play`, `transcode_plan`, and
  `transcode_requirement`.
- Moved transcode-profile helpers onto `PlaybackTargetProfile` and deleted
  `PlaybackProfile`.
- Verified planner, API redaction, and server playback gates.

## Active Task

None.

## Important Files

- `crates/nako-playback/src/lib.rs`
- `crates/nako-playback/src/capability.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/selection.rs`
- `docs/adr/0044-playback-capability-profile-planner.md`
- `docs/adr/0049-source-aware-transcode-runtime.md`

## Cautions

- Keep Public Client response redaction intact.
- Do not add adaptive HLS, fMP4, DLNA profile databases, rsmpeg, or remote
  worker behavior in this lane.
- Do not preserve `PlaybackProfile` for compatibility; Nako has no users yet,
  and the richer target-profile identity is the desired request-key shape.
