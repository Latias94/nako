# Playback Capability Profile Planner

Status: Completed
Last updated: 2026-05-27

## Why This Lane Exists

Nako's **Playback Runtime** has the right high-level seams, but playback
compatibility still uses a shallow client shape. That makes the next decoding
and transcode work risky: hardware decode, subtitle burn-in, HDR tone mapping,
adaptive HLS, desktop-native playback, and renderer adapters all need a richer
decision model before FFmpeg parameters are expanded.

This lane turns playback compatibility into a deep Module owned by
`nako-playback`.

## Relevant Authority

- ADRs:
  - `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
  - `docs/adr/0039-playback-policy-and-renderer-target-boundary.md`
  - `docs/adr/0040-casting-as-renderer-session-adapter.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/workstreams/playback-transcode-policy-deepening/`
  - `docs/workstreams/playback-policy-and-renderer-targets/`
  - `docs/workstreams/external-casting-adapter-boundary/`
- Reference behavior:
  - `repo-ref/jellyfin/MediaBrowser.Model/Dlna/DeviceProfile.cs`
  - `repo-ref/jellyfin/MediaBrowser.Model/Dlna/StreamBuilder.cs`
  - `repo-ref/jellyfin/MediaBrowser.Model/Configuration/EncodingOptions.cs`
  - `repo-ref/jellyfin/MediaBrowser.Controller/MediaEncoding/EncodingHelper.cs`

## Problem

Current `nako-playback` decisions can choose direct play, remux, or HLS
transcode, but compatibility is mostly:

- file-name container inference;
- probed video/audio codec name checks;
- a direct-play boolean;
- requested remux or transcode output container;
- broad playback permission checks.

This is not enough for Jellyfin-class decoding and playback behavior. The
planner cannot yet explain conditions such as bitrate limit, profile/level,
resolution, frame-rate, bit depth, HDR range, interlace, subtitle delivery, or
audio-channel mismatch. It also cannot describe a transcode requirement deeply
enough for a future hardware pipeline planner.

## Target State

When this lane closes:

- `nako-playback` owns a profile-driven playback capability model.
- `PlaybackPlanner` returns a decision report with typed compatibility reasons.
- Direct play, remux, and transcode decisions are derived from profile
  capabilities and **Media Technical Facts**, not ad hoc route branching.
- The profile model has stable identity keys for playback cache/session reuse.
- Existing browser/native default behavior is represented as default profiles,
  not special-case code.
- `nako-server` remains an adapter: load facts, resolve policy, construct a
  profile, call planner, execute returned plan.
- FFmpeg command planning remains in `nako-transcode`.

## In Scope

- Workstream and ADR.
- Characterization tests for current direct/remux/HLS decisions.
- `PlaybackTargetProfile` and profile identity records in `nako-playback`.
- Typed compatibility conditions and decision report records.
- Planner migration from codec-list branching to profile capability matching.
- Default profile builders for browser, native, and renderer targets.
- Server adapter changes required to build planner profiles.
- Focused tests proving direct play, remux, transcode, denial, and reason
  behavior.

## Out Of Scope

- Frontend UI.
- Mobile client implementation.
- Tauri/desktop player implementation.
- New hardware acceleration backends.
- HDR tone mapping execution.
- Subtitle burn-in execution.
- Adaptive HLS ladders.
- Optimized Versions.
- DLNA profile import/export compatibility.
- Copying reference project source, schemas, tests, or comments.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `nako-playback` is the right home for pure compatibility planning. | High | ADR 0038 and the existing crate already own `PlaybackPlanner`. | If dependency pressure grows, move pure records to `nako-core` and keep planning in `nako-playback`. |
| Output breadth can stay narrow while the model gets deeper. | High | Current product only executes direct/remux/single-variant HLS. | If clients require new outputs during this lane, split an output-breadth follow-on. |
| Public DTO compatibility is not a hard constraint yet. | High | The project is pre-production and the user approved fearless refactoring. | Public Client API and SDK tests must be updated in the same commit slice. |
| Jellyfin is a capability checklist, not a model to copy. | High | AGENTS.md repo-ref licensing rule and ADR 0044. | If exact Jellyfin compatibility becomes a goal, open a separate ADR and compatibility lane. |
| Hardware decode should follow the profile planner. | High | Hardware decisions need a transcode requirement and reasons first. | If an urgent hardware bug appears, keep it a focused patch without widening this lane. |

## Architecture Direction

### Playback Target Profile

Introduce a Nako-owned `PlaybackTargetProfile`:

```text
PlaybackTargetProfile
  direct_play
  direct_play_profiles
  remux_profiles
  transcode_profiles
  compatibility_rules
  subtitle_capabilities
  output_constraints
  identity
```

This profile is smaller than Jellyfin's DLNA model but can express the same
classes of feature pressure.

### Decision Report

Planner output should include a report:

```text
PlaybackDecisionReport
  evaluated source
  direct play reasons
  remux reasons
  transcode reasons
  selected mode
  policy denial
```

The report is the test surface. Routes and DTO mappers must not infer reasons.

### Compatibility Conditions

Conditions should be typed and stable:

- container not supported;
- video codec not supported;
- audio codec not supported;
- video profile/level/bitrate/resolution/frame-rate unsupported;
- audio channels/sample-rate/bitrate unsupported;
- subtitle delivery unsupported;
- HDR/range unsupported;
- requested output requires transcode;
- policy denial.

The first implementation can only populate conditions backed by existing
**Media Technical Facts**, while preserving enum space and tests for future
fields.

### Transcode Requirement

The planner should return a playback-shaped transcode requirement, not FFmpeg
arguments:

```text
TranscodeRequirement
  output container
  video output codec
  audio output codec
  selected streams
  output constraints
  subtitle strategy hint
  reasons
```

`nako-transcode` later turns this into decode/filter/encode pipeline and
FFmpeg command plans.

## Closeout Condition

This lane can close when:

- workstream docs and ADR 0044 reflect shipped behavior;
- `nako-playback` has profile-driven planner records and tests;
- shallow codec-list-only compatibility is removed or made an adapter;
- server playback code constructs profiles and consumes decision reports;
- focused `nako-playback` and `nako-server` tests pass;
- remaining decode, subtitle, HDR, and HLS breadth work is split into follow-on
  workstreams.
