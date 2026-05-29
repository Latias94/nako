# Evidence And Gates

## Required Gates

- `python -m json.tool docs/workstreams/playback-api-transcode-boundary-cleanup/WORKSTREAM.json`
- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo tree -p nako-api --depth 1`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Log

### PATB-010

- `python -m json.tool docs/workstreams/playback-api-transcode-boundary-cleanup/WORKSTREAM.json`
  passed on 2026-05-29.
- `git diff --check -- docs/workstreams/playback-api-transcode-boundary-cleanup docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`
  passed on 2026-05-29. Git reported only line-ending normalization warnings
  for existing tracked docs.

### PATB-020

- `cargo check -p nako-api --tests` passed on 2026-05-29.
- `cargo check -p nako-server --tests` passed on 2026-05-29.
- `cargo nextest run -p nako-api --no-fail-fast` passed on 2026-05-29:
  69 passed, 0 skipped.
- `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast` passed
  on 2026-05-29: 9 passed, 442 skipped by filter.
- `cargo tree -p nako-api --depth 1` showed direct dependencies on
  `nako-addon-protocol`, `nako-client-protocol`, `nako-core`,
  `nako-playback`, `nako-streaming`, `serde`, `serde_json`, `sha2`, and
  `time`; no direct `nako-transcode` edge remained.
- `cargo fmt --all -- --check` passed on 2026-05-29.
- `git diff --check` passed on 2026-05-29. Git reported only line-ending
  normalization warnings for tracked files.

### PATB-030

- `rg -n "nako_transcode|TranscodePlan|OutputContainer|HlsVariantPolicy|HlsSegmentContainer|RemuxContainer|HlsOutputRequirement|TranscodeOutputConstraints|TranscodeTrackSelection|TranscodeSubtitleStrategy" crates/nako-playback crates/nako-api -g "*.rs"`
  passed on 2026-05-29.
- `cargo tree -p nako-playback --depth 1` showed `nako-playback` still has a
  direct `nako-transcode` dependency.
- Inventory result: `nako-playback` publicly exposes transcode-owned values in
  planner and target types, including `TranscodePlan`, `OutputContainer`,
  `RemuxContainer`, `HlsOutputRequirement`, `HlsVariantPolicy`,
  `HlsSegmentContainer`, `TranscodeOutputConstraints`,
  `TranscodeTrackSelection`, and `TranscodeSubtitleStrategy`.
- Decision: split the remaining planner/runtime boundary cleanup to a separate
  follow-on, proposed slug `playback-planner-transcode-value-vocabulary`.
  Recommended shape is playback-owned planner value objects plus server-side
  adapters into `nako-transcode` execution types.

### PATB-040

- `cargo nextest run -p nako-server playback --no-fail-fast` passed on
  2026-05-29: 122 passed, 329 skipped by filter.
- Closeout records that the API cleanup target is complete and the remaining
  transcode-shaped playback planner surface is deferred to the follow-on lane.
