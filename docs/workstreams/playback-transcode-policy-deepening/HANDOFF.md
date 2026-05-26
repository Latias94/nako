# Playback Transcode Policy Deepening - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane is open. PTP-010, PTP-020, PTP-030, PTP-040, and PTP-050 are
complete.

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
- `TranscodeExecutionPolicy`, which carries decode/filter/encode acceleration
  stage choices, fallback evidence, output constraints, and subtitle strategy
  into HLS profile identity and FFmpeg request planning.

PTP-020 added a direct-play characterization test proving that direct playback
creates a durable Playback Session and no fake Transcode Session artifact.
PTP-030 then moved playback selection out of `nako-streaming` into
`nako-playback`; `nako-streaming` now remains direct/range transport mechanics.
PTP-040 promoted public playback reason/capability DTOs and regenerated the
TypeScript/Kotlin SDKs. The existing remux, HLS, browser ticket, redaction,
cancellation, and hardware fallback tests remain green.
PTP-050 replaced the single HLS hardware field with stage-based transcode
policy, removed Public Client exposure of server hardware selection, and kept
FFmpeg encoder/filter strings inside the FFmpeg adapter.

Jellyfin reference review found the feature pressures Nako must be ready for:

- device/client profiles;
- playback-info decisions and stream URLs;
- explicit transcode reasons;
- encoding options for hardware, tonemapping, subtitles, throttling, and
  cleanup;
- transcode job pings/progress/cancel;
- user policy for remux/audio transcode/video transcode.

## Active Task

- Task ID: PTP-060
- Status: ready
- Scope: Runtime Inventory and Transcode Engine Adapter.

## Decisions

- Nako will not copy Jellyfin code or adopt Jellyfin's DLNA model wholesale.
- Nako will use feature pressure from Jellyfin to deepen its own Playback
  Planner, Transcode Policy, Runtime Inventory, and Transcode Engine Adapter
  seams.
- FFmpeg CLI remains the first engine Adapter.
- Hardware acceleration must be modeled as decode/filter/encode stage selection
  plus fallback policy, not a boolean.
- Public Client transcode plans should describe requested output container and
  codecs only; hardware acceleration selection is service-side runtime/admin
  evidence.
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

Run PTP-060. Bind the transcode policy to a redaction-safe runtime inventory
and start moving FFmpeg CLI execution behind a typed engine Adapter. Preserve
the existing command runner behavior while making start/cancel/progress and
capability evidence engine-shaped instead of route-shaped.
