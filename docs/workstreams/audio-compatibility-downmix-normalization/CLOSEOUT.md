# Audio Compatibility Downmix Normalization - Closeout

Status: Closed
Date: 2026-05-31

## Result

`ACDN-010` through `ACDN-050` are complete. Nako now models playback-owned
audio output requirements, propagates them through transcode policy/profile and
server HLS adaptation, and emits deterministic FFmpeg HLS downmix and
normalization filters when requested.

## Final Gates

```text
cargo nextest run -p nako-playback audio --no-fail-fast
cargo nextest run -p nako-transcode hls audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/audio-compatibility-downmix-normalization/WORKSTREAM.json
git diff --check
```

All final gates passed on 2026-05-31.

## Follow-ons

- persisted audio preferences and night-mode/product controls;
- client UI controls for downmix and normalization intent;
- broad device profile databases and calibration evidence;
- dialogue clarity or audio enhancement;
- subtitle burn-in;
- HDR tone mapping.
