# Playback Media Maturity First Slices - Evidence And Gates

Status: Completed
Last updated: 2026-05-28

## Gate Set

### Focused Iteration Gates

```bash
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-client-protocol --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server renderer --no-fail-fast
```

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/playback-media-maturity-first-slices/WORKSTREAM.json
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-28 PMMFS-010: Opened the first-slice playback media maturity lane
  after `source-aware-transcode-runtime` and
  `playback-capability-profile-and-rendition-planning` closed. Validation is
  passed with `python3 -m json.tool
  docs/workstreams/playback-media-maturity-first-slices/WORKSTREAM.json` and
  `git diff --check -- docs/workstreams/playback-media-maturity-first-slices
  docs/workstreams/README.md`.
- 2026-05-28 PMMFS-020: Added richer Public Client capability input for
  bitrate, resolution, audio channels, HDR, selected subtitles, HLS variant
  policy, and HLS segment container across protocol DTOs, OpenAPI, generated
  TypeScript/Kotlin SDKs, Rust client query helpers, server query adapters,
  browser ticket validation, and renderer registration. Validation passed with
  `cargo nextest run -p nako-client-protocol --no-fail-fast`, `cargo nextest
  run -p nako-api --no-fail-fast`, `cargo nextest run -p nako-client
  --no-fail-fast`, `cargo check -p nako-client-cli`, and `npm run check
  --prefix sdk/typescript`.
- 2026-05-28 PMMFS-030: Added `HlsVariantPolicy`, `HlsSegmentContainer`, and
  `HlsOutputRequirement` planning vocabulary in `nako-transcode`, carried by
  playback `TranscodeRequirement`. Validation passed with `cargo nextest run -p
  nako-transcode --no-fail-fast` and `cargo nextest run -p nako-playback
  --no-fail-fast`.
- 2026-05-28 PMMFS-040: Direct-play evaluation now emits explicit reasons for
  bitrate, resolution, HDR, audio-channel, and selected-subtitle incompatibility
  from client capability limits. Validation passed with `cargo nextest run -p
  nako-playback --no-fail-fast`, `cargo nextest run -p nako-server playback
  --no-fail-fast`, and `cargo nextest run -p nako-server renderer
  --no-fail-fast`.
- 2026-05-28 PMMFS-050: Closeout checks passed with `cargo fmt --all --
  --check`, `git diff --check`, and refreshed generated SDK/Admin contract
  files.

## Notes

- Do not claim executable fMP4/CMAF or adaptive bitrate ladder runtime support
  before a runtime lane verifies FFmpeg output and serving behavior.
- Do not expose source locators, raw host paths, command lines, or internal
  transcode requirements on Public Client surfaces.
- Reference projects under `repo-ref/` are behavior references only; write
  original Nako records, tests, docs, and implementations.
