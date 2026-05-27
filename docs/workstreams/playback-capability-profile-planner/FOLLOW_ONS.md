# Playback Capability Profile Planner - Follow Ons

Status: Active
Last updated: 2026-05-27

## FFMPEG Hardware Pipeline Planner

Recommended next lane:
`docs/workstreams/ffmpeg-hardware-pipeline-planner/`

Scope:

- convert planner transcode requirements into decode/filter/encode pipeline
  requirements;
- model software decode and hardware decode separately;
- expand hardware capability reports from encoder presence to decoder,
  hwaccel, filter, bitstream-filter, device, and smoke-probe evidence;
- add platform adapters for NVENC/NVDEC, QSV, VAAPI, AMF, and VideoToolbox
  without exposing FFmpeg command strings to Public Client API.

## Subtitle Audio HDR Transcode Maturity

Recommended follow-on:
`docs/workstreams/subtitle-audio-hdr-transcode-maturity/`

Scope:

- subtitle delivery profile support: drop, sidecar, HLS subtitle, embed,
  burn-in;
- audio compatibility conditions: channels, sample-rate, bitrate, codec,
  secondary audio, external audio;
- HDR, color range, bit depth, and tone-map requirements;
- Admin diagnostics for unsupported subtitle/HDR paths.

## HLS Output Maturity

Recommended follow-on:
`docs/workstreams/hls-output-maturity/`

Scope:

- multi-codec or multi-variant HLS where product needs justify it;
- segment lifecycle, retention, cleanup, and throttling policy;
- codec string reporting and media playlist manifest correctness;
- future optimized-version and reusable artifact relationship.

## Media Technical Facts Breadth

Recommended follow-on:
`docs/workstreams/media-technical-facts-breadth/`

Scope:

- persist probe facts needed by richer compatibility conditions:
  profile, level, frame-rate, bit depth, interlace, rotation, HDR/range,
  color primaries, transfer, matrix, subtitle flags, and stream disposition;
- keep these facts distinct from **Canonical Metadata**.

