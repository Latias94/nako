# Design

## Problem

`nako-api` currently imports `nako-transcode` for DTO conversion and Admin
diagnostics fields. That makes API ownership depend on execution-layer types
such as `TranscodePlan`, `HardwareAccelerationPolicy`, and
`TranscodePipelineReadiness`.

The dependency direction is backwards for a self-hosted media server that needs
stable clients. API crates should own wire contracts; server composition should
translate runtime facts into those contracts.

## Target State

- Public Client DTO conversion reads playback decisions without naming
  transcode execution types in `nako-api`.
- Admin playback/config DTOs expose API-local hardware and readiness enums with
  the same serialized values as the previous transcode-backed fields.
- `nako-server` owns the conversion from `nako-transcode` runtime/config facts
  to API DTOs.
- `cargo tree -p nako-api` shows no direct `nako-transcode` dependency.

## Final State

The target state is complete. `nako-api` owns the Admin hardware/readiness DTOs,
`nako-server` maps transcode runtime/config values into those DTOs, and
`nako-api` no longer has a direct `nako-transcode` dependency.

PATB-030 found that removing `nako-playback -> nako-transcode` is a different
planner/runtime value-vocabulary refactor. That work should not be hidden in
this API cleanup lane.

## Scope

- `crates/nako-api`
- `crates/nako-server`
- `docs/workstreams/playback-api-transcode-boundary-cleanup`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/README.md`

## Non-Goals

- No wire field rename, schema migration, route change, or generated SDK route
  inventory change.
- No removal of `nako-playback -> nako-transcode` in the first slice.
- No FFmpeg command, hardware probing, HLS generation, or runtime scheduler
  behavior change.
- No frontend behavior change.

## Boundary Direction

`nako-api` stays contract-focused:

```text
nako-client-protocol + nako-core + DTO helpers
```

`nako-server` composes runtime facts:

```text
nako-server -> nako-playback
nako-server -> nako-transcode
nako-server -> nako-api DTO adapters
```

The API crate may still consume `nako-playback` decision records in this lane
because the existing Public Client mapping functions already use playback
planner output. The next boundary review decides whether that mapping should
move entirely into server-side adapters or into a client-protocol mapper crate.

## Compatibility Plan

Admin hardware enums must preserve previous `serde(rename_all = "snake_case")`
values:

- `none`, `vaapi`, `nvenc`, `quick_sync`, `amf`, `video_toolbox`
- `cpu`, `fail`
- readiness and hardware-stage values already emitted by transcode runtime

Public Client transcode plan fields keep the existing `ClientTranscodePlan`
shape.
