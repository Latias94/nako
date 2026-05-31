# PTJCH-130 Worker Note

Status: Merged
Merged commit: `bb3835e0`
Last updated: 2026-05-31

## Summary

`PTJCH-130` split the FFmpeg command adapter internals:

- `ffmpeg.rs` now exposes the builder facade and delegates to internal
  `common`, `remux`, and `hls` modules.
- HLS planning is split into input, filters, encoders, muxer, seek, and
  sidecar helpers.
- Remux validation and command planning are isolated in the remux adapter.
- A regression test verifies the primary HLS output is planned before sidecar
  audio/subtitle outputs.

## Validation

```text
cargo nextest run -p nako-transcode ffmpeg hls --no-fail-fast
cargo nextest run -p nako-transcode remux --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31 before and after rebasing onto the main branch
that already contained `PTJCH-120`.
