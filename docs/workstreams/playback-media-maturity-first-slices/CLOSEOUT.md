# Playback Media Maturity First Slices - Closeout

Status: Completed
Last updated: 2026-05-28

## Summary

This lane completed the first post-source-aware media maturity refactor slice.
Nako now accepts richer client capability profiles, carries HLS adaptive/fMP4
planning intent in transcode requirements, and explains more direct-play
incompatibilities before falling back to transcode.

## Delivered

- Added Public Client capability fields for video bitrate, resolution, audio
  channels, HDR support, selected-subtitle delivery, HLS variant policy, and
  HLS segment container.
- Mapped those fields through browser ticket validation, playback query params,
  renderer registration, Public Client DTOs, OpenAPI, generated TypeScript and
  Kotlin SDKs, Rust client query helpers, and server adapters.
- Added `HlsVariantPolicy`, `HlsSegmentContainer`, and `HlsOutputRequirement`
  planner vocabulary in `nako-transcode`.
- Added `TranscodeRequirement.hls_output` and profile identity coverage in
  `nako-playback`.
- Added `video_hdr_unsupported` and explicit direct-play reasons for bitrate,
  resolution, HDR, audio-channel, and selected-subtitle limits.

## Verification

Passed:

```bash
python3 -m json.tool docs/workstreams/playback-media-maturity-first-slices/WORKSTREAM.json
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-client-protocol --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server renderer --no-fail-fast
cargo nextest run -p nako-client --no-fail-fast
cargo check -p nako-client-cli
cargo fmt --all -- --check
git diff --check
npm run check --prefix sdk/typescript
```

Notes:

- `npm run check --prefix sdk/typescript` initially failed because `tsc` was not
  installed locally. `npm ci --prefix sdk/typescript` installed the locked
  dependency, and the check then passed.
- `cargo nextest run -p nako-api --no-fail-fast` initially detected stale
  generated TypeScript contract output. The Admin contract plus Public
  TypeScript/Kotlin SDKs were regenerated, then the gate passed.

## Follow-Ons

- Executable adaptive HLS ladder planning and FFmpeg command output.
- Executable fMP4/CMAF segment output and serving semantics.
- Subtitle burn-in/sidecar execution paths.
- DLNA-style device profile import/adaptation if external renderer lanes need
  it.
- rsmpeg execution adapter feasibility as a separate runtime adapter lane.
