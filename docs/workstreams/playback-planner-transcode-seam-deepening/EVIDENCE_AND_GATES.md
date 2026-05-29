# Playback Planner Transcode Seam Deepening - Evidence And Gates

Status: Completed
Last updated: 2026-05-29

## Smallest Current Repro

```bash
rg -n "try_hls_transcode_profile|hls_transcode_profile|remux_transcode_profile" crates
```

The current repro shows playback-owned profile builders and server/test call
sites that should move to transcode-owned builders.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-transcode profile --no-fail-fast
```

### Server Behavior Gate

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

### Closeout Gate

```bash
cargo fmt --all -- --check
git diff --check
python3 -m json.tool docs/workstreams/playback-planner-transcode-seam-deepening/WORKSTREAM.json
```

## Evidence Anchors

- `crates/nako-playback/src/capability.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-server/src/app/playback`
- `docs/workstreams/playback-planner-transcode-seam-deepening/TODO.md`
- `docs/workstreams/playback-planner-transcode-seam-deepening/HANDOFF.md`

## Evidence Log

- 2026-05-29 PPTS-010: Opened the workstream around the playback/transcode
  profile-builder seam. The first target is deleting playback-owned
  `TranscodeProfile` construction while preserving request identity.
- 2026-05-29 PPTS-020: Added `PlaybackRemuxProfileRequest`,
  `PlaybackHlsProfileRequest`, `build_playback_remux_profile`, and
  `build_playback_hls_profile` to `nako-transcode`. Deleted
  `PlaybackTargetProfile::{remux_transcode_profile,try_remux_transcode_profile,hls_transcode_profile,try_hls_transcode_profile}`.
  The HLS builder now rejects non-HLS transcode plans at the seam. Validation
  passed with nextest run `3f3364e3-7707-4a76-ab1c-cb3ec94753b5`: `cargo
  nextest run -p nako-transcode profile --no-fail-fast` (13 passed, 60
  skipped), and nextest run `389d7aa2-b07d-4609-bacb-23c053458a78`: `cargo
  nextest run -p nako-playback --no-fail-fast` (19 passed).
- 2026-05-29 PPTS-030: Updated playback app remux/HLS composition and
  app/http test identity helpers to call transcode-owned profile builders.
  Validation passed with nextest run
  `d576c630-975d-45a8-97cc-85904db8eeae`: `cargo nextest run -p nako-server
  playback --no-fail-fast` (118 passed, 324 skipped).
- 2026-05-29 PPTS-040: Closed the lane after final non-test checks and JSON
  validation passed: `python3 -m json.tool
  docs/workstreams/playback-planner-transcode-seam-deepening/WORKSTREAM.json`,
  `cargo fmt --all -- --check`, and `git diff --check`. Follow-on media
  features stay split into dedicated lanes.
