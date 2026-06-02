# Directory Structure

`nako-playback` is a planner crate. It transforms domain facts into a typed
`PlaybackDecision` and `TranscodeRequirement`.

## Current Layout

```text
crates/nako-playback/src/
├── lib.rs          # planner entrypoint, decision records, public exports
├── capability.rs   # target profiles and compatibility evaluation
└── values.rs       # audio, color, HLS, remux, transcode value types
```

## Module Rules

- Keep decision records, selected source, planning request, and rendition plans
  in `lib.rs` unless a focused module is justified.
- Keep client capability normalization and Direct Play/Remux/Transcode
  evaluation in `capability.rs`.
- Keep reusable output value types in `values.rs`.
- Re-export core playback policy and renderer target records from
  `nako-core` only when they are needed by planner callers.

## Forbidden Placement

- Do not execute FFmpeg here. FFmpeg command planning and runtime belong to
  `nako-transcode` and `nako-server`.
- Do not serve HTTP byte ranges, HLS playlists, or tickets here. Transport
  belongs to `nako-streaming` and `nako-server`.
- Do not read storage backends directly. Use `PlaybackStorageContext` facts from
  callers.
- Do not write playback sessions or database state here.
- Do not implement resource admission here. Admission lives in server app
  runtime helpers such as `crates/nako-server/src/app/playback/resource.rs`.

## Examples

- `PlaybackPlanningRequest`: source, probe, target, policy, and context inputs.
- `PlaybackTargetProfile::identity`: stable profile identity key for request
  facts.
- `TranscodeRequirement`: planner output consumed by transcode/runtime code.
