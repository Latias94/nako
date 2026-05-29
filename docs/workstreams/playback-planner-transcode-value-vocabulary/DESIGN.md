# Design

## Problem

`nako-playback` currently imports `nako-transcode` for planner-facing value
types:

- remux output container;
- transcode output container;
- HLS variant policy and segment container;
- HLS output requirement;
- transcode plan;
- track selection;
- output constraints;
- subtitle strategy.

Those values appear in public playback planner records, so transcode execution
vocabulary leaks upward into planner ownership. This makes future planner work
look like a transcode execution change even when FFmpeg behavior does not
change.

## Target State

- `nako-playback` owns playback planner value objects.
- `nako-server` translates planner values into `nako-transcode` execution
  requests close to orchestration boundaries.
- Request identity strings remain byte-for-byte compatible unless a test proves
  a deliberate change is required.
- Public Client DTO output remains stable.
- `cargo tree -p nako-playback --depth 1` shows no direct `nako-transcode`
  dependency.

## Scope

- `crates/nako-playback`
- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/http/renderer.rs`
- server playback tests that construct planner values
- `docs/workstreams/playback-planner-transcode-value-vocabulary`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/README.md`

## Non-Goals

- No HLS feature behavior change.
- No FFmpeg command planning change.
- No new media engine, remote worker, or scheduler behavior.
- No Public Client or Admin wire field rename.
- No database schema migration.

## Boundary Plan

`nako-playback` should expose values with playback names:

```text
PlaybackRemuxContainer
PlaybackTranscodeContainer
PlaybackHlsVariantPolicy
PlaybackHlsSegmentContainer
PlaybackHlsOutputRequirement
PlaybackTranscodePlan
PlaybackTrackSelection
PlaybackOutputConstraints
PlaybackSubtitleStrategy
```

`nako-server` should own adapters:

```text
PlaybackRemuxContainer -> nako_transcode::RemuxContainer
PlaybackTranscodeContainer -> nako_transcode::OutputContainer
PlaybackHlsOutputRequirement -> nako_transcode::HlsOutputRequirement
PlaybackTrackSelection -> nako_transcode::TranscodeTrackSelection
PlaybackOutputConstraints -> nako_transcode::TranscodeOutputConstraints
```

Do not put these conversions in `nako-api`; API DTO mapping should stay
decoupled from transcode execution types.

`PlaybackSubtitleStrategy` remains planner-owned intent. The server runtime
still derives its final execution subtitle strategy close to HLS media rendition
planning, where sidecar subtitle overrides are available.

## Risk Plan

- Request identity strings depend on `as_str()` and container extension helpers.
  New playback-owned types must keep the same string values.
- Server playback runtime currently passes planner values directly into
  transcode profile builders. Missing adapters will compile-fail quickly but can
  affect many tests.
- `TranscodeSubtitleStrategy::SidecarSelected` appears in server runtime policy,
  not only planner output. The playback-owned strategy should cover planner
  intent; server-only execution overrides may remain transcode-owned.
