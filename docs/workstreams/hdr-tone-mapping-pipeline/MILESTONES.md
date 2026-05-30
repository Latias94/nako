# HDR Tone Mapping Pipeline - Milestones

Status: Active
Last updated: 2026-05-30

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
- HDR implementation remains blocked while `ACDN-020` owns the same playback
  vocabulary files.

## M1 - Playback Color Requirement Vocabulary

Exit criteria:

- activation criteria are defined by `HTP-010`;
- playback-owned values can express direct-compatible HDR and tone-map-required
  cases.

Status: Ready after planner activation.

Activation criteria:

- accepted `ACDN-020` is merged into the HDR branch;
- the HDR worker rereads `nako-playback` after the audio requirement changes;
- no Public Client API, media probe schema, or transcode code change is needed
  for the playback vocabulary slice.

## M2 - Transcode Tone-Mapping Strategy

Exit criteria:

- activation criteria are defined by `HTP-010`;
- software and hardware FFmpeg strategies are deterministic and testable;
- CPU fallback behavior is explicit.

Status: Blocked pending `HTP-020`.

First media-output target:

- software-first HLS HDR-to-SDR tone mapping for HDR10/PQ or HLG sources;
- SDR-only client path represented by playback's color requirement;
- deterministic command-plan coverage before any hardware-specific strategy;
- hardware tone mapping and Dolby Vision/HDR10+ dynamic behavior deferred to
  follow-ons unless the planner splits them earlier.

## M3 - Verification And Closeout

Exit criteria:

- final gates pass with fresh evidence or the lane records explicit blockers;
- docs and `WORKSTREAM.json` reflect active/completed/deferred status;
- follow-ons are split or explicitly deferred.

Status: Blocked pending implementation plan.
