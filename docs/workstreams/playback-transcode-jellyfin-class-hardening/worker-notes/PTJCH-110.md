# PTJCH-110 Worker Note

Status: Merged
Merged commit: `0d3bd96f`
Last updated: 2026-05-31

## Summary

`PTJCH-110` deepened playback capability handling for remux fallback:

- `evaluate_remux` now applies playback output bitrate and resolution
  constraints instead of only checking remux container/codecs.
- Transcode requirement reasons now include non-compatible remux evaluation
  reasons as well as direct-play reasons.
- A table-driven playback test covers client bitrate cap, client resolution
  cap, and user bitrate preference cases where remux is blocked and HLS
  transcode becomes the selected mode.

## Validation

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31. `git diff --check` emitted LF/CRLF
working-copy warnings for touched Rust files and no whitespace errors.
