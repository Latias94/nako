# HDR Tone Mapping Pipeline - Milestones

Status: Active
Last updated: 2026-05-31

## M0 - Research And Scope Freeze

Exit criteria:

- current HDR/color probe facts are identified;
- playback/client capability input gaps are documented;
- first implementation task has explicit owned scope and validation;
- shared-scope conflict with audio compatibility is resolved by sequencing.

Status: Done.

HTP-010 result:

- existing probe facts are sufficient for the first playback planning slice;
- existing client facts are sufficient for the first slice through
  `supports_hdr=false`, with richer display/profile data deferred;
- the first implementation task is playback-only `HTP-020`;
- the shared playback vocabulary conflict was resolved for `HTP-020` by
  merging accepted `ACDN-020` into this HDR branch before implementation.

## M1 - Playback Color Requirement Vocabulary

Exit criteria:

- activation criteria are defined by `HTP-010`;
- playback-owned values can express direct-compatible HDR and tone-map-required
  cases.

Status: Done and accepted.

Evidence:

- `HTP-020` stayed inside `nako-playback`;
- `TranscodeRequirement` now carries a playback-owned color pipeline
  requirement;
- tests cover HDR passthrough, HDR-to-SDR tone mapping intent, and deferred
  unsupported dynamic HDR paths;
- no Public Client API, media probe schema, transcode, server HLS, or web code
  changed.

## M2 - Transcode Tone-Mapping Strategy

Exit criteria:

- activation criteria are defined by `HTP-010`;
- software and hardware FFmpeg strategies are deterministic and testable;
- CPU fallback behavior is explicit.

Status: Done, pending planner review.

First media-output target:

- software-first HLS HDR-to-SDR tone mapping for HDR10/PQ or HLG sources;
- SDR-only client path represented by playback's color requirement;
- deterministic command-plan coverage before any hardware-specific strategy;
- hardware tone mapping and Dolby Vision/HDR10+ dynamic behavior deferred to
  follow-ons unless the planner splits them earlier.

Evidence:

- `nako-transcode` owns color pipeline execution policy values without adding a
  dependency on `nako-playback`;
- HLS runtime/profile identity carries the color pipeline requirement so
  SDR-mapped and non-tone-mapped outputs do not reuse the same request identity;
- FFmpeg command planning emits a deterministic software HDR-to-SDR video
  filter before HLS H.264 encoding;
- HLS server code only maps playback requirements into transcode-owned request
  values and starts the existing runtime/execution planner path.

## M3 - Verification And Closeout

Exit criteria:

- final gates pass with fresh evidence or the lane records explicit blockers;
- docs and `WORKSTREAM.json` reflect active/completed/deferred status;
- follow-ons are split or explicitly deferred.

Status: Ready for planner review and closeout.
