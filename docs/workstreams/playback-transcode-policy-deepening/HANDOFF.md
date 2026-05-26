# Playback Transcode Policy Deepening - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane is open. PTP-010, PTP-020, PTP-030, and PTP-040 are complete.

Nako already has:

- durable Playback Sessions separate from optional Transcode Sessions;
- direct/remux/HLS playback routes;
- browser playback tickets;
- FFmpeg-backed remux/HLS command planning and runners;
- Admin playback runtime diagnostics;
- redacted playback session lists and support evidence.
- `nako-playback`, which owns playback planning records, client capability
  matching, profile identity, selected source records, and typed internal
  decision reasons.
- Public Client playback contracts with typed `ClientPlaybackDecisionReason`
  and shared `ClientPlaybackCapabilitiesDto`.

PTP-020 added a direct-play characterization test proving that direct playback
creates a durable Playback Session and no fake Transcode Session artifact.
PTP-030 then moved playback selection out of `nako-streaming` into
`nako-playback`; `nako-streaming` now remains direct/range transport mechanics.
PTP-040 promoted public playback reason/capability DTOs and regenerated the
TypeScript/Kotlin SDKs. The existing remux, HLS, browser ticket, redaction,
cancellation, and hardware fallback tests remain green.

Jellyfin reference review found the feature pressures Nako must be ready for:

- device/client profiles;
- playback-info decisions and stream URLs;
- explicit transcode reasons;
- encoding options for hardware, tonemapping, subtitles, throttling, and
  cleanup;
- transcode job pings/progress/cancel;
- user policy for remux/audio transcode/video transcode.

## Active Task

- Task ID: PTP-050
- Status: ready
- Scope: Transcode Policy and Acceleration Plan.

## Decisions

- Nako will not copy Jellyfin code or adopt Jellyfin's DLNA model wholesale.
- Nako will use feature pressure from Jellyfin to deepen its own Playback
  Planner, Transcode Policy, Runtime Inventory, and Transcode Engine Adapter
  seams.
- FFmpeg CLI remains the first engine Adapter.
- Hardware acceleration must be modeled as decode/filter/encode stage selection
  plus fallback policy, not a boolean.
- Public Client playback contracts and Admin diagnostics stay separate.
- `nako-playback` is the planner/profile/reason crate. It was split early
  because deleting selection from `nako-streaming` made transport boundaries
  cleaner and both server app code and API adapters consume the planning
  records.
- Public Client DTOs expose reason as stable wire values. Admin diagnostics may
  carry richer runtime evidence, but should not leak raw host details into
  Public Client responses.

## Blockers

- None.

## Next Action

Run PTP-050. Replace shallow hardware selection with typed transcode policy and
acceleration planning: decode/filter/encode stage choices, fallback policy,
bitrate/output constraints, and subtitle strategy. Keep FFmpeg-specific strings
below the engine/command adapter.
