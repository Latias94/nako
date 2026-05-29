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
