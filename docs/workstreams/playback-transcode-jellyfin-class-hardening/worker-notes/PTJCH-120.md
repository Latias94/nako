# PTJCH-120 Worker Note

Status: Merged
Merged commit: `9f841951`
Last updated: 2026-05-31

## Summary

`PTJCH-120` deepened stage-aware Transcode Pipeline Capability matching:

- `HardwareAccelerationCapability` can now query available stage features.
- Pipeline source compatibility now checks requested decode-stage support for
  HEVC/AV1 instead of treating H.264 as the only supported hardware source
  input.
- QuickSync HEVC decode is selected when the decode stage is present and falls
  back when the stage is missing.
- Unavailable requested pipeline reasons now preserve fail-policy vs CPU
  fallback semantics.

## Validation

```text
cargo nextest run -p nako-transcode pipeline hardware probe --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31 before and after rebasing onto main.
