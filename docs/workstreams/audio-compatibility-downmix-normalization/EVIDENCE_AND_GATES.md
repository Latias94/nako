# Audio Compatibility Downmix Normalization - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Required Gates

```text
python -m json.tool docs/workstreams/audio-compatibility-downmix-normalization/WORKSTREAM.json
cargo nextest run -p nako-playback audio --no-fail-fast
cargo nextest run -p nako-transcode hls audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run the playback package gate before transcode propagation. Broaden to server
HLS only after runtime adaptation changes.

## Evidence Ledger

### ACDN-010 - Scope and evidence freeze

Status: Done

Evidence:

- `docs/workstreams/audio-compatibility-downmix-normalization/DESIGN.md`
- `docs/workstreams/audio-compatibility-downmix-normalization/TODO.md`
- `docs/workstreams/audio-compatibility-downmix-normalization/WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Notes:

- The first executable task is playback requirement vocabulary only.
- HDR tone mapping, subtitle burn-in, persisted preferences, and web controls
  are outside this lane.

## Residual Risks

- Real device audio capability databases may require a later profile import or
  calibration lane.
- Normalization defaults can become product-sensitive; keep initial behavior
  deterministic and explainable.
- Downmix and HDR implementation both touch playback/transcode seams. Do not
  implement HDR code concurrently with this lane.
