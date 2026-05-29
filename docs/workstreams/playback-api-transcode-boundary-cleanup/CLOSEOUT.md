# Closeout

Status: Completed
Closed: 2026-05-29

## Result

The lane achieved its target: `nako-api` no longer directly depends on
`nako-transcode`.

`nako-api` now owns Admin hardware acceleration and transcode pipeline readiness
DTOs. `nako-server` maps transcode runtime/config values into those DTOs before
returning Admin responses or persisted playback runtime settings. Public Client
playback decision conversion still hides source locators and internal rendition
state without importing `nako_transcode`.

## Verification

- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo tree -p nako-api --depth 1`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python -m json.tool docs/workstreams/playback-api-transcode-boundary-cleanup/WORKSTREAM.json`

## Follow-On

PATB-030 found that `nako-playback` still exposes transcode-owned planner value
objects. That should become a separate lane, proposed slug
`playback-planner-transcode-value-vocabulary`.

Recommended target:

- `nako-playback` owns planner-facing values for remux/transcode output,
  HLS output shape, track selection, output constraints, and subtitle strategy.
- `nako-server` maps playback planner values to `nako-transcode` execution
  values.
- Existing playback decision JSON, request identity, and HLS/remux behavior stay
  unchanged.
