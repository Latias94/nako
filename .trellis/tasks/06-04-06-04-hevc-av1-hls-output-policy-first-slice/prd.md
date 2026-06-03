# HEVC AV1 HLS output policy first slice

## Goal

Add a bounded typed HLS video-output codec policy seam so Nako can distinguish
the shipped H264/AAC execution baseline from recognized future HEVC and AV1 HLS
output policy, without enabling HEVC/AV1 FFmpeg command execution or changing
public playback behavior in this slice.

## What I Already Know

* `docs/architecture/PLAYBACK.md` keeps HEVC/AV1 output policy as a playback /
  transcode follow-on after the broader hardware inventory work.
* `crates/nako-transcode/src/hardware.rs` already records optional HEVC/AV1
  encoder capability facts for VAAPI, NVENC, QSV, AMF, and VideoToolbox.
* `crates/nako-transcode/src/profile.rs` currently validates HLS output codecs
  as raw strings and allows only H264 video plus AAC audio.
* FFmpeg HLS encoder planning currently maps selected hardware acceleration to
  H264 encoders only. Changing that execution path is out of scope for this
  first slice.

## Assumptions

* H264/AAC remains the default and only executable HLS output profile.
* HEVC and AV1 should become recognized policy vocabulary before any FFmpeg
  encoder execution change lands.
* The useful first slice is a typed validation / diagnostics seam in
  `nako-transcode`, not a playback planner or API contract change.

## Requirements

* Add typed HLS video-output codec policy vocabulary in `nako-transcode`.
* Recognize at least H264, HEVC/H265, and AV1 as distinct policy values.
* Preserve existing default behavior: omitted HLS video codec and explicit H264
  remain valid.
* Reject HEVC and AV1 HLS output profiles as recognized but deferred /
  unsupported, with a distinct validation reason from unknown codecs.
* Reject unknown HLS output codecs as unsupported.
* Keep AAC as the only allowed HLS audio output codec.
* Do not change playback decisions, public API DTOs, server HLS runtime routes,
  FFmpeg command builders, artifact manifests, schema, or Admin/Web surfaces.
* Add focused tests for policy classification and HLS profile validation.

## Acceptance Criteria

* [x] HLS output codec policy recognizes H264, HEVC/H265, and AV1.
* [x] HLS profile validation accepts omitted video codec and explicit H264.
* [x] HLS profile validation rejects HEVC and AV1 with a deferred unsupported
  reason.
* [x] HLS profile validation rejects unknown codecs with the existing unsupported
  class or a clearly separate unsupported class.
* [x] Existing HLS command planning remains H264-only.
* [x] `cargo fmt --all -- --check`, `cargo check -p nako-transcode --tests`,
  focused `cargo nextest run -p nako-transcode <filter> --no-fail-fast`,
  `git diff --check`, and Trellis validate pass.

## Definition Of Done

* Code and tests are committed with a Conventional Commit message.
* Verification evidence is persisted in this task directory.
* Reusable policy conventions are written back to relevant specs or
  architecture docs.
* Task is archived and the developer journal is recorded.

## Out Of Scope

* No HEVC/AV1 FFmpeg encoder argv changes.
* No HLS playlist, segment, codec-string, or container execution changes.
* No default playback profile change away from H264/AAC.
* No public API, generated SDK, Admin/Web, database schema, or server route
  change.
* No hardware availability based auto-selection.
* No copying from reference repositories.

## Technical Approach

* Add a small typed classifier in `crates/nako-transcode/src/profile.rs`, likely
  `HlsVideoOutputCodec`, `HlsVideoOutputPolicyStatus`, and an output policy
  decision helper.
* Update HLS profile validation to use the classifier:
  * omitted codec -> H264 baseline;
  * H264 -> executable;
  * HEVC/H265 and AV1 -> deferred unsupported;
  * anything else -> unsupported.
* Keep the FFmpeg HLS encoder builder unchanged, so executable command planning
  remains H264-only.
* Add unit tests next to existing transcode profile tests and run focused
  profile/HLS gates.

## Research References

* [`research/current-hls-output-policy.md`](research/current-hls-output-policy.md)
  - current code shape, policy gap, and bounded first-slice recommendation.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-transcode/backend/index.md`
  * `.trellis/spec/nako-transcode/backend/quality-guidelines.md`
  * `.trellis/spec/nako-playback/backend/index.md`
  * `.trellis/spec/nako-playback/backend/quality-guidelines.md`
* Relevant architecture docs:
  * `docs/architecture/PLAYBACK.md`
  * `docs/architecture/OPERATIONS_RELEASE.md`
