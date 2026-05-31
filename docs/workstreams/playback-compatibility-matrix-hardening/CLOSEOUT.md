# Playback Compatibility Matrix Hardening - Closeout

Date: 2026-05-31
Status: Closed

## Result

`PCMH-010` through `PCMH-030` are complete. `nako-playback` now has a
table-driven compatibility matrix for representative Direct Play, Remux, and
HLS Transcode decisions.

The matrix covers compatible MP4/H.264/AAC Direct Play, compatible-stream MKV
Remux, unsupported video/audio codec HLS Transcode, HDR-to-SDR tone-map-required
Remux denial, audio downmix-required Remux denial, requested HLS output shape,
and audio output requirement combinations for passthrough, downmix,
normalization, and combined downmix plus normalization.

## Final Gates

```text
cargo nextest run -p nako-playback compatibility --no-fail-fast
cargo nextest run -p nako-playback hdr audio --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/playback-compatibility-matrix-hardening/WORKSTREAM.json
git diff --check
```

All final gates passed on 2026-05-31. `git diff --check` reported only Windows
line-ending normalization warnings.

## Follow-ons

- full device profile compatibility matrices;
- persisted playback preferences or player controls;
- Public Client API compatibility reporting;
- transcode execution policy and FFmpeg command-plan matrices;
- server HLS composition cases that cannot be proven in `nako-playback`.

## Residual Risks

This lane proves representative planner behavior, not an exhaustive device
database. It does not validate executable FFmpeg filters, runtime admission, or
player-specific behavior.
