# Current HLS output policy research

- Query: choose the smallest useful HEVC/AV1 HLS output policy slice.
- Scope: `nako-transcode` profile validation and policy vocabulary.
- Date: 2026-06-04.

## Findings

### Current code shape

- `crates/nako-transcode/src/profile.rs` stores HLS output video/audio codecs as
  optional strings on `HlsTranscodeProfile` and `TranscodeProfile`.
- `TranscodeProfile::validate_hls` currently accepts only `h264` video output
  and `aac` audio output.
- Omitted HLS output codecs are valid and are used by default HLS profile
  paths.
- `crates/nako-transcode/src/ffmpeg/hls/encoders.rs` maps hardware acceleration
  to H264 encoder names such as `h264_vaapi`, `h264_nvenc`, `h264_qsv`,
  `h264_amf`, and `h264_videotoolbox`.
- `crates/nako-transcode/src/hardware.rs` already tracks optional HEVC and AV1
  encoder facts by hardware vendor, but those facts are not an HLS output
  execution policy yet.
- `docs/architecture/PLAYBACK.md` keeps HEVC/AV1 output policy split from HLS
  admission, subtitle burn-in, and seek/restart work.

### Bounded first slice

The next safe slice should make output codec policy typed before enabling any
new FFmpeg execution path:

- Recognize H264, HEVC/H265, and AV1 in the HLS profile policy layer.
- Keep H264 as the only executable HLS video output.
- Mark HEVC and AV1 as recognized but deferred unsupported.
- Keep unknown codec rejection explicit.
- Leave playback planner, server routes, FFmpeg argv, artifacts, schema, and API
  untouched.

## Risks

- Enabling HEVC/AV1 FFmpeg encoder args in the same slice would require HLS
  compatibility, hardware availability, codec-string, segment/container, and
  client support decisions.
- Changing playback defaults away from H264/AAC would affect public behavior and
  client compatibility.
- Reusing raw codec strings throughout the profile validator would make later
  HEVC/AV1 execution policy harder to audit.

## Verification candidates

- Unit tests for HLS output codec classification.
- Profile validation tests for H264 accepted, HEVC/AV1 deferred, and unknown
  codec unsupported.
- Existing HLS command tests should still pass without HEVC/AV1 encoder argv
  changes.
