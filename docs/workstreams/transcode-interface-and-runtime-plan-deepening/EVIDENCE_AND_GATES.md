# Transcode Interface And Runtime Plan Deepening - Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Required Gates

```text
python -m json.tool docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json
cargo nextest run -p nako-transcode hls audio --no-fail-fast
cargo nextest run -p nako-transcode remux --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run focused transcode planning gates before the server HLS gate. Broaden only
when the implementation touches shared playback runtime behavior.

## Evidence Ledger

### TIRP-010 - Scope and evidence freeze

Status: Done

Evidence:

- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/DESIGN.md`
- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/TODO.md`
- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/architecture/LANES.md`
- `python -m json.tool docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json`
  - 2026-05-31: Passed.
- `git diff --check -- docs/workstreams/transcode-interface-and-runtime-plan-deepening docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LANES.md docs/workstreams/README.md`
  - 2026-05-31: Passed with only Windows line-ending warnings.

Notes:

- The first executable task is HLS runtime plan Interface deepening.
- HDR tone mapping, broad hardware capability matrices, HLS lifecycle
  consolidation, and resource admission unification are outside this first
  workstream unless the planner splits a task explicitly.

### TIRP-020 - HLS runtime plan Interface

Status: Done

Interface changes:

- Added `HlsRuntimePlanRequest` and `HlsRuntimePlan` in `nako-transcode`.
- Added `TranscodePipelinePlanner::plan_hls_runtime` to assemble HLS execution
  policy, profile identity, request variant, and request identity behind the
  transcode Interface.
- Added `HlsMediaRenditionPlan::selected_from_probe` so audio sidecar and
  selected subtitle rendition identity assembly is transcode-owned.
- Added `HlsStagingPolicy::layout_for_runtime_plan` so server HLS staging
  consumes the runtime plan instead of reassembling HLS output/request variant
  details.

Evidence:

- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/artifact.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/staging_policy.rs`
- `cargo nextest run -p nako-transcode hls_runtime_plan --no-fail-fast`
  - 2026-05-31: Passed, 2 tests.
- `cargo nextest run -p nako-transcode hls audio --no-fail-fast`
  - 2026-05-31: Passed, 50 tests.
- `cargo nextest run -p nako-server hls --no-fail-fast`
  - 2026-05-31: Passed, 61 tests; one slow adaptive HLS regression.
- `cargo fmt --all -- --check`
  - 2026-05-31: Passed.
- `git diff --check`
  - 2026-05-31: Passed with only Windows line-ending warnings.

Planner fresh verification on 2026-05-31:

- `cargo nextest run -p nako-transcode remux --no-fail-fast` passed, 13
  tests.
- `cargo nextest run -p nako-transcode hls --no-fail-fast` passed, 50 tests.
- `cargo nextest run -p nako-server hls --no-fail-fast` passed, 61 tests, 6
  slow.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending warnings.

Notes:

- Runtime behavior is unchanged: existing server HLS regression coverage for
  selected audio, subtitle sidecars, multi-audio sidecars, adaptive ladder,
  seek generation, acceleration identity, fallback, and reuse passed.
- No HDR tone mapping, subtitle burn-in, hardware capability matrix expansion,
  HLS lifecycle consolidation, resource admission unification, or direct
  `nako-transcode` dependency on `nako-playback` was introduced.

### TIRP-030 - FFmpeg Adapter Interface ratchet

Status: Done

Interface changes:

- Added `FfmpegExecutionPlanner`, `HlsExecutionPlanRequest`, and
  `RemuxExecutionPlanRequest` as the public execution planning Interface for
  FFmpeg-backed HLS/remux execution requests.
- Kept raw `HlsRequest`, `RemuxRequest`, `FfmpegCommandBuilder`,
  `FfmpegCommandPlan`, `FfmpegArg`, and `FfmpegOverwritePolicy` crate-internal
  in `nako-transcode`; `lib.rs` now only re-exports `RemuxContainer` from the
  low-level FFmpeg module.
- Made `TranscodeExecutionRequest` an opaque engine-start package with
  crate-visible command details and public read-only accessors for session,
  source, kind, and output path.
- Updated server HLS/remux orchestration to hold `FfmpegExecutionPlanner` and
  submit high-level execution plan requests instead of constructing raw FFmpeg
  requests or builder state.

Evidence:

- `crates/nako-transcode/src/execution.rs`
- `crates/nako-transcode/src/ffmpeg.rs`
- `crates/nako-transcode/src/lib.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/playback/remux.rs`
- `cargo nextest run -p nako-transcode "ffmpeg_execution_planner_plans" --no-fail-fast`
  - 2026-05-31: Passed, 2 tests.
- `cargo nextest run -p nako-transcode hls --no-fail-fast`
  - 2026-05-31: Passed, 51 tests.
- `cargo nextest run -p nako-transcode remux --no-fail-fast`
  - 2026-05-31: Passed, 14 tests.
- `cargo nextest run -p nako-server hls --no-fail-fast`
  - 2026-05-31: Passed, 61 tests, 23 slow. First local attempt timed out
    before completion with a short harness timeout; rerun with the same command
    completed successfully.
- `rg -n "\b(FfmpegCommandBuilder|HlsRequest|RemuxRequest|FfmpegArg|FfmpegOverwritePolicy)\b" crates/nako-server/src/app/playback/hls.rs crates/nako-server/src/app/playback/remux.rs`
  - 2026-05-31: Passed with no matches.
- `cargo fmt --all -- --check`
  - 2026-05-31: Passed.
- `git diff --check`
  - 2026-05-31: Passed with only Windows line-ending warnings.

Planner fresh verification on 2026-05-31:

- `python -m json.tool docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json`
  passed.
- `cargo nextest run -p nako-transcode hls --no-fail-fast` passed with 51
  tests run.
- `cargo nextest run -p nako-transcode remux --no-fail-fast` passed with 14
  tests run.
- `cargo nextest run -p nako-server hls --no-fail-fast` passed with 61 tests
  run and 9 slow tests.
- `rg -n "\b(FfmpegCommandBuilder|HlsRequest|RemuxRequest|FfmpegArg|FfmpegOverwritePolicy)\b" crates/nako-server/src/app/playback/hls.rs crates/nako-server/src/app/playback/remux.rs`
  returned no matches.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending warnings.

Notes:

- Runtime behavior is unchanged: HLS planning still uses FFmpeg overwrite
  allow semantics, remux planning still uses overwrite-never semantics, and the
  existing server HLS runtime regressions passed.
- No HDR tone mapping, subtitle burn-in, hardware capability matrix expansion,
  HLS lifecycle consolidation, resource admission unification, public API DTO
  or generated contract changes, or direct `nako-transcode` dependency on
  `nako-playback` was introduced.

## Residual Risks

- HLS lifecycle and resource admission remain separate shallow areas; do not
  solve them opportunistically in this workstream closeout.
- Follow-on HDR/tone-map work should extend the transcode-owned planner and
  execution Interfaces instead of reintroducing server-owned FFmpeg request
  assembly.
