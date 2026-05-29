# Handoff

Status: Completed
Last updated: 2026-05-29

Current task: None

## Current State

PATB-010 opened the workstream and linked it from the playback/transcode
architecture index. Existing playback seam workstreams are completed and do not
cover the API crate dependency.

PATB-020 removed the direct `nako-api -> nako-transcode` dependency. API-local
Admin hardware/readiness DTOs now preserve the previous wire values, while
`nako-server` maps transcode runtime/config facts into those DTOs.

PATB-030 found that `nako-playback` still directly exposes transcode-owned
planner values. That is a planner/runtime value-vocabulary problem and should
be split from this completed API cleanup lane.

PATB-040 verified and closed the lane.

## Follow-On

Open `playback-planner-transcode-value-vocabulary` when ready:

- introduce playback-owned planner value objects for remux container,
  transcode output container, HLS output shape, track selection, output
  constraints, and subtitle strategy;
- map those values to `nako-transcode` execution types in `nako-server`;
- preserve current playback decision JSON and request identity behavior.

## Risks

- The API cleanup target is complete, but `nako-api` still consumes
  `nako-playback` decision records. If a future lane removes
  `nako-playback -> nako-transcode`, re-check `nako-api` conversion helpers
  because they currently read planner fields through playback types.
- Persisted playback runtime settings rely on hardware enum strings. The new
  API-local enums intentionally preserve those values.
