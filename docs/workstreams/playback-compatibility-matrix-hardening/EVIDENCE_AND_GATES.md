# Playback Compatibility Matrix Hardening - Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Required Gates

```text
python -m json.tool docs/workstreams/playback-compatibility-matrix-hardening/WORKSTREAM.json
cargo nextest run -p nako-playback compatibility --no-fail-fast
cargo nextest run -p nako-playback hdr audio --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run focused playback tests first. Broaden only if the implementation changes
shared playback planner behavior.

## Evidence Ledger

### PCMH-010 - Scope and evidence freeze

Status: Done

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/README.md`

Notes:

- This lane is safe to run beside HDR `HTP-030` because it is scoped to
  `nako-playback`.
- Any need to edit `nako-transcode`, `nako-server`, API DTOs, or web/player
  behavior must return to planner coordination.

### PCMH-020 - Playback decision matrix

Status: Done

Evidence:

- `crates/nako-playback/src/lib.rs`
- `playback_compatibility_matrix_covers_direct_play_remux_hls_transcode_hdr_and_audio`
- `playback_compatibility_matrix_audio_output_requirements_cover_downmix_and_normalization`

Coverage:

- Direct Play for an MP4/H.264/AAC compatible source.
- Remux for an MKV source with compatible video and audio streams.
- HLS Transcode for unsupported video codec.
- HLS Transcode for unsupported audio codec.
- HLS Transcode instead of Remux when an SDR-only client receives an HDR MKV
  and tone mapping is required.
- HLS Transcode instead of Remux when audio channel limits require downmix.
- Requested HLS Transcode carrying selected HLS output shape.
- Audio output requirement cases for stereo passthrough, surround downmix,
  normalization request, and combined downmix plus normalization.

Verification on 2026-05-31:

- `cargo nextest run -p nako-playback compatibility --no-fail-fast` passed
  with 2 tests run.
- `cargo nextest run -p nako-playback hdr audio --no-fail-fast` passed with
  12 tests run.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending normalization
  warnings.

Scope check:

- The task stayed inside `crates/nako-playback/src/lib.rs`.
- No transcode, server, Public Client API DTO, persisted preference, device
  profile, media probe schema, or web/player behavior changed.

## Residual Risks

- The matrix will prove representative compatibility behavior, not every
  future device profile database row.
- The lane intentionally does not add executable FFmpeg tone mapping or audio
  filter behavior.
