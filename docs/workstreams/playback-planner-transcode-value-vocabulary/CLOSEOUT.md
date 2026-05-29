# Closeout

Status: Completed
Closed: 2026-05-29

## Result

The lane achieved its target: `nako-playback` no longer directly depends on
`nako-transcode`.

`nako-playback` now owns planner-facing values for remux output containers,
transcode output containers, HLS output requirements, track selection, output
constraints, transcode plans, and subtitle strategy. `nako-server` owns the
adapters that turn those planner values into `nako-transcode` execution types
at playback orchestration boundaries.

Existing Public Client playback decision DTOs, HLS/remux request identity
strings, and playback runtime behavior were kept compatible.

## Verification

- `cargo check -p nako-playback --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo tree -p nako-playback --depth 1`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python -m json.tool docs/workstreams/playback-planner-transcode-value-vocabulary/WORKSTREAM.json`

## Follow-On

No immediate follow-on is required. The remaining playback/transcode boundary is
intentional: `nako-server` and `nako-transcode` still own FFmpeg execution,
runtime policy, HLS artifact layout, and hardware pipeline planning.
