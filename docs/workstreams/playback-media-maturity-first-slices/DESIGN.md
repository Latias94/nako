# Playback Media Maturity First Slices - Design

Status: Completed
Last updated: 2026-05-28

## Problem

Nako now has source-aware media facts, typed playback rendition plans, and a
runtime boundary for HLS, remux, support evidence, and diagnostics. The next
risk is that client capability input is still too shallow: callers can mostly
send direct-play, container, video codec, audio codec, and output container
preferences.

That shape is not deep enough for a self-hosted media server that must handle
browser, desktop, mobile, renderer, Chromecast-like, DLNA-like, and future
remote-worker playback pressure. Jellyfin shows the breadth of behavior Nako
must eventually support, but Nako should keep its own smaller profile model and
avoid copying Jellyfin's DLNA object graph.

## Intent

Deepen the existing capability-profile and rendition-planning boundary without
starting a new media engine or changing runtime execution before the planner can
describe the desired output.

This lane is intentionally a first-slice refactor, not full productization of
adaptive bitrate ladders or CMAF/fMP4 serving.

## Scope

- `crates/nako-playback`: richer `ClientPlaybackCapabilities`,
  `PlaybackTargetProfile` mapping, compatibility reasons, and transcode
  requirements.
- `crates/nako-client-protocol`: Public Client DTOs for browser playback
  capability input and renderer/client capability records.
- `crates/nako-api`: DTO mapping and OpenAPI schema updates.
- `crates/nako-server`: HTTP query/body adapters for browser playback tickets,
  direct playback decisions, and renderer capability registration.
- `crates/nako-transcode`: planning vocabulary for HLS variant policy and
  segment container, carried as requirements before runtime execution expands.
- `docs/workstreams`: task ledger, evidence, and handoff.

## Non-Goals

- Do not implement full adaptive bitrate ladder generation.
- Do not make FFmpeg emit fMP4/CMAF segments in this lane.
- Do not add DLNA device profiles or copy Jellyfin profile structures.
- Do not introduce rsmpeg as an execution adapter.
- Do not add database migrations unless a task proves they are needed.

## Boundary Direction

The planner should receive a capability profile that can express:

- direct-play limits for bitrate, resolution, HDR, audio channels, and subtitle
  delivery;
- HLS rendition preferences for single-variant versus adaptive planning;
- HLS segment container preference for MPEG-TS versus fMP4;
- output constraints that can be passed into transcode requirements without
  leaking host paths or FFmpeg command details.

Target flow:

```text
Public Client capability input
  -> ClientPlaybackCapabilities
  -> PlaybackTargetProfile
  -> PlaybackDecision.rendition
  -> TranscodeRequirement
  -> Transcode runtime planning
```

Runtime should keep serving existing HLS MPEG-TS unless and until a future lane
implements executable adaptive/fMP4 output. The first-slice value is a stable
planning contract and explicit reasons.

## Testing Plan

- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-client-protocol --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo nextest run -p nako-server renderer --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python3 -m json.tool docs/workstreams/playback-media-maturity-first-slices/WORKSTREAM.json`

## Risk Plan

- **Wire churn:** Nako has no compatibility-bound users yet, but defaults should
  preserve current behavior where fields are absent.
- **Runtime overclaiming:** fMP4/adaptive fields describe planner intent only in
  this lane. Public evidence must not claim executable fMP4 support until the
  runtime implements it.
- **Reason drift:** public decision reports should expose typed compatibility
  reasons without leaking `TranscodeRequirement`, source locators, host paths,
  or command strings.
- **Profile sprawl:** keep the model smaller than Jellyfin DLNA profiles and
  split DLNA-specific profile work into a follow-on lane.

## Closeout Condition

This lane can close when:

- Public Client and browser playback inputs can express the first richer
  capability profile fields.
- `PlaybackTargetProfile` consumes those fields for direct-play and transcode
  requirement decisions.
- HLS requirement planning can distinguish single/adaptive and MPEG-TS/fMP4
  intent without changing runtime output behavior.
- Planner tests cover bitrate, resolution, audio-channel, HDR, and subtitle
  constraints.
- API/server/protocol mappings and OpenAPI schemas are updated and verified.

## Closeout Summary

Completed on 2026-05-28. Public Client capability input now carries direct-play
limits for bitrate, resolution, audio channels, HDR, subtitles, plus HLS
single/adaptive and MPEG-TS/fMP4 planning preferences.

`PlaybackTargetProfile` consumes those fields for direct-play evaluation,
profile identity, output constraints, and source-aware transcode requirements.
The runtime still serves existing HLS MPEG-TS output; executable adaptive
ladder and fMP4/CMAF output remain split follow-ons.
