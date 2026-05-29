# HLS Seek Restart Lifecycle - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Smallest Current Repro

```bash
cargo nextest run -p nako-transcode hls_request_variant --no-fail-fast
```

This proves HLS request variant identity before adding playback-generation
components.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-transcode hls_request_variant --no-fail-fast
cargo nextest run -p nako-server hls_source_request_identity --no-fail-fast
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

### Runtime Gate

```bash
cargo nextest run -p nako-server hls_source --no-fail-fast
```

### Closeout Gate

```bash
cargo fmt --all -- --check
git diff --check
python3 -m json.tool docs/workstreams/hls-seek-restart-lifecycle/WORKSTREAM.json
```

## Evidence Anchors

- `crates/nako-transcode/src/artifact.rs`
- `crates/nako-transcode/src/lib.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/tests/playback.rs`
- `docs/workstreams/hls-seek-restart-lifecycle/HANDOFF.md`

## Evidence Log

- 2026-05-29 HSRL-010: Opened the workstream and scoped the first slice to
  request identity/generation modeling before runtime cancellation or FFmpeg
  seek flags.
- 2026-05-29 HSRL-020: Added `HlsPlaybackGeneration` to
  `HlsRequestVariantPlan` and threaded it through internal `HlsSourceRequest`
  handling. Default `0 ms` generation does not alter existing request keys;
  non-zero generation changes `TranscodeRequestIdentity` and HLS staging
  layout. Validation passed with nextest run
  `4e8d3628-c39f-42dd-9f6c-5a0eb32fe057`: `cargo nextest run -p
  nako-transcode hls_request_variant --no-fail-fast` (3 passed, 72 skipped),
  nextest run `637f6a62-7779-4d1f-80aa-f7b4f646d5d0`: `cargo nextest run -p
  nako-server hls_source_request_identity --no-fail-fast` (3 passed, 440
  skipped), and runtime gate run `5e035fe0-51c8-4259-aaf0-efb643ba8908`:
  `cargo nextest run -p nako-server hls_source --no-fail-fast` (14 passed, 429
  skipped, 1 nextest leak warning).
- 2026-05-29 HSRL-030: Made HLS request admission explicit in
  `HlsAppService`: same-generation active duplicates still conflict,
  same-generation finished outputs still reuse, and a different request key for
  the same source supersedes active HLS sessions by requesting cancellation
  before starting the new session. Red test run
  `a691deb6-fafc-448b-8b45-da092cb9db79` failed before implementation because
  the prior generation remained `running`; green targeted run
  `868111ce-4767-4969-b16e-1339438e274e`: `cargo nextest run -p nako-server
  hls_source_seek_generation_supersedes_active_prior_generation
  --no-fail-fast` (1 passed, 446 skipped). Runtime gate run
  `4be7e479-c87e-4f07-9571-69c7e6adfa16`: `cargo nextest run -p nako-server
  hls_source --no-fail-fast` (15 passed, 432 skipped, 1 nextest leak warning).
  Closeout gates passed: `cargo fmt --all -- --check`, `git diff --check`,
  and `python3 -m json.tool
  docs/workstreams/hls-seek-restart-lifecycle/WORKSTREAM.json >/dev/null`.
- 2026-05-29 HSRL-040: Threaded `HlsPlaybackGeneration` from server playback
  into `HlsRequest` command planning. Non-default generations now emit input
  `-ss` before `-i`, `-avoid_negative_ts make_zero`,
  `-force_key_frames expr:gte(t,n_forced*segment_time)`, and
  `-hls_flags independent_segments`; default generation preserves existing HLS
  argv. Red test failed before implementation with `E0560` because
  `HlsRequest` did not expose `playback_generation`. Focused green runs:
  `7347848c-1b85-476a-89a8-05104fbd651c`: `cargo nextest run -p
  nako-transcode ffmpeg_builder_plans_hls_seek_generation_input_and_segment_flags
  --no-fail-fast` (1 passed, 75 skipped), and
  `acbd8c0b-fe50-4714-8239-5bd0a85c893b`: `cargo nextest run -p nako-server
  hls_source_seek_generation_reaches_ffmpeg_command --no-fail-fast` (1 passed,
  447 skipped). Task gates passed with `9eef82dd-750f-4d87-804e-1628f327bbf4`:
  `cargo nextest run -p nako-transcode hls --no-fail-fast` (36 passed, 40
  skipped), and `393f53b6-2dfb-43ef-8d2b-84a4df493189`: `cargo nextest run -p
  nako-server hls --no-fail-fast` (48 passed, 400 skipped).
  Closeout gates passed: `cargo fmt --all -- --check`, `git diff --check`,
  and `python3 -m json.tool
  docs/workstreams/hls-seek-restart-lifecycle/WORKSTREAM.json >/dev/null`.
