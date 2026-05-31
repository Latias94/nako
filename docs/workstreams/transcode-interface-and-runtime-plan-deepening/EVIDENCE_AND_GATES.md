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

## Residual Risks

- The first implementation may reveal that a small server adapter still needs
  to translate playback-owned values into transcode-owned values. Keep that
  adapter thin; do not make `nako-transcode` depend on `nako-playback` without
  planner review.
- Tightening `pub use` can break tests or downstream internal callers. Ratchet
  exports only after the higher-level Interface exists.
- HLS lifecycle and resource admission remain separate shallow areas; do not
  solve them opportunistically in `TIRP-020`.
