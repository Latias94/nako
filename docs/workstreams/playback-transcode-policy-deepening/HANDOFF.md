# Playback Transcode Policy Deepening - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane is open. PTP-010, PTP-020, and PTP-030 are complete.

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

PTP-020 added a direct-play characterization test proving that direct playback
creates a durable Playback Session and no fake Transcode Session artifact.
PTP-030 then moved playback selection out of `nako-streaming` into
`nako-playback`; `nako-streaming` now remains direct/range transport mechanics.
The existing remux, HLS, browser ticket, redaction, cancellation, and hardware
fallback tests remain green.

Jellyfin reference review found the feature pressures Nako must be ready for:

- device/client profiles;
- playback-info decisions and stream URLs;
- explicit transcode reasons;
- encoding options for hardware, tonemapping, subtitles, throttling, and
  cleanup;
- transcode job pings/progress/cancel;
- user policy for remux/audio transcode/video transcode.

## Active Task

- Task ID: PTP-040
- Status: ready
- Scope: stable Client Playback Capabilities and Playback Decision Reasons.

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
- Public Client DTOs still expose reason as a safe string. PTP-040 should
  decide the stable protocol reason vocabulary before richer admin diagnostics
  rely on it.

## Blockers

- None.

## Next Action

Run PTP-040. Promote typed Client Playback Capabilities and Playback Decision
Reasons into stable protocol/API shapes without copying Jellyfin's DLNA profile
model. Keep richer operator diagnostics out of Public Client DTOs.
