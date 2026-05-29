# Playback Planner Transcode Seam Deepening - Design

Status: Completed
Last updated: 2026-05-29

## Problem

The planner currently owns too much transcode execution vocabulary:

- `PlaybackTargetProfile` builds `RemuxTranscodeProfile` and
  `HlsTranscodeProfile`.
- `nako-playback` imports `TranscodeProfile`, `TranscodeExecutionPolicy`, and
  transcode validators.
- Server playback orchestration calls profile builders on the playback target
  profile before starting remux/HLS runtime work.

This is behaviorally correct today, but it makes the next playback features add
more transcode execution details to the planner crate.

## Desired Boundary

```text
nako-playback
  -> source/client/policy/user preference compatibility
  -> PlaybackRenditionPlan
  -> selected tracks, output constraints, HLS output requirement

nako-transcode
  -> playback transcode profile request
  -> TranscodeProfile validation
  -> TranscodeProfile identity
  -> FFmpeg/runtime-facing policy vocabulary

nako-server
  -> compose planner output with runtime policy
  -> call transcode-owned profile builders
  -> bind source identity and staging/runtime artifacts
```

## Implementation Shape

This lane added transcode-owned playback profile request builders:

- remux request: output container, selected tracks, remote input flag, playback
  profile key;
- HLS request: validated `TranscodePlan`, execution policy, HLS output
  requirement, selected tracks, remote input flag, playback profile key.

The profile-building methods were deleted from `PlaybackTargetProfile`. The
planner keeps lightweight accessors for selected tracks, output constraints,
HLS output, and identity key because those are planning facts, not execution
profiles.

## Risk

- Request identity may change if builder input ordering changes. Preserve the
  existing `TranscodeProfile` persisted request key material.
- Server tests that construct expected request identities must move to the new
  builder instead of relying on playback methods.
- This lane should not alter runtime behavior, HLS playlists, session reuse, or
  FFmpeg command planning. Focused playback gates passed to cover this.

## Follow-Ons

- HLS seek/restart lifecycle.
- HDR tone mapping policy and FFmpeg filter planning.
- Audio downmix/normalization policy and execution filters.
- Runtime resource scheduler and per-host transcode capacity diagnostics.
