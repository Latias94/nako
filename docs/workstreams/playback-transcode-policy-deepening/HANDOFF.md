# Playback Transcode Policy Deepening - Handoff

Status: Completed
Last updated: 2026-05-27

## Current State

The lane is closed. PTP-010 through PTP-080 are complete.

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
- `TranscodeRuntimeInventory`, which gives Admin diagnostics a redaction-safe
  FFmpeg/runtime capability summary.
- `TranscodeEngineAdapter`, which makes FFmpeg remux/HLS execution expose typed
  start outcomes and progress snapshots.
- persisted Admin playback runtime settings with restart-effect reporting.
- artifact lifecycle and throttle diagnostics in Admin playback runtime and
  support evidence.
- startup cleanup for expired terminal remux/HLS artifacts under the configured
  transcode root, with security-skip accounting for paths outside that root.
- thin playback HTTP routes over app-service orchestration for direct stream,
  remux stream, HLS playlist, and HLS segment target resolution.

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
PTP-060 added the runtime inventory and made remux/HLS runners the first
FFmpeg CLI engine adapters consumed by server playback orchestration.
PTP-070 added persisted playback runtime settings, lifecycle/throttle
diagnostics, generated Admin TypeScript contract updates, and startup artifact
cleanup.
PTP-080 moved route-owned direct/remux/HLS playback orchestration into
`PlaybackAppService`, preserved browser ticket behavior, and added explicit
coverage for legacy HLS segment URLs by Transcode Session id.

Jellyfin reference review found the feature pressures Nako must be ready for:

- device/client profiles;
- playback-info decisions and stream URLs;
- explicit transcode reasons;
- encoding options for hardware, tonemapping, subtitles, throttling, and
  cleanup;
- transcode job pings/progress/cancel;
- user policy for remux/audio transcode/video transcode.

## Active Task

- Task ID: none
- Status: completed
- Scope: Playback Transcode Policy Deepening is closed.

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
- Runtime capability summaries should flow through `TranscodeRuntimeInventory`
  instead of being recomputed separately in Admin HTTP adapters.
- FFmpeg process details stay behind `TranscodeEngineAdapter`; server playback
  orchestration should talk in terms of typed artifact outcomes and progress.
- Admin playback runtime settings are persisted as typed JSON payloads in a
  generic Admin settings document. The server applies them during startup and
  reports `requires_restart` until the running config matches the persisted
  payload.
- Playback artifact cleanup is rooted under the configured transcode staging
  root; terminal artifacts outside that root are counted as security skips
  rather than deleted.

## Blockers

- None.

## Next Action

Open focused follow-on lanes for adaptive HLS ladders, optimized versions,
remote transcode workers, desktop/native player capability negotiation, subtitle
delivery, HDR tonemapping, SyncPlay, DLNA, and per-user playback permission
policy.
