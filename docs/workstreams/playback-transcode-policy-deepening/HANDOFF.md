# Playback Transcode Policy Deepening - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane is open. PTP-010 and PTP-020 are complete.

Nako already has:

- durable Playback Sessions separate from optional Transcode Sessions;
- direct/remux/HLS playback routes;
- browser playback tickets;
- FFmpeg-backed remux/HLS command planning and runners;
- Admin playback runtime diagnostics;
- redacted playback session lists and support evidence.

PTP-020 added a direct-play characterization test proving that direct playback
creates a durable Playback Session and no fake Transcode Session artifact. The
existing remux, HLS, browser ticket, redaction, cancellation, and hardware
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

- Task ID: PTP-030
- Status: ready
- Scope: Playback Planner records and Module.

## Decisions

- Nako will not copy Jellyfin code or adopt Jellyfin's DLNA model wholesale.
- Nako will use feature pressure from Jellyfin to deepen its own Playback
  Planner, Transcode Policy, Runtime Inventory, and Transcode Engine Adapter
  seams.
- FFmpeg CLI remains the first engine Adapter.
- Hardware acceleration must be modeled as decode/filter/encode stage selection
  plus fallback policy, not a boolean.
- Public Client playback contracts and Admin diagnostics stay separate.
- A new crate is deferred until reuse pressure proves it is deeper than app
  Modules plus core records.

## Blockers

- None.

## Next Action

Run PTP-030. Add a Playback Planner Module that returns direct/remux/HLS plans
and typed decision reasons from source facts, client capabilities, runtime
facts, and policy. Keep HTTP routes as adapters over planner output.
