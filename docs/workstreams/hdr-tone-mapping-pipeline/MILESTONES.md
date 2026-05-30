# HDR Tone Mapping Pipeline - Milestones

Status: Draft
Last updated: 2026-05-30

## M0 - Research And Scope Freeze

Exit criteria:

- current HDR/color probe facts are identified;
- playback/client capability input gaps are documented;
- first implementation task has explicit owned scope and validation;
- shared-scope conflict with audio compatibility is resolved by sequencing.

Status: Ready.

## M1 - Playback Color Requirement Vocabulary

Exit criteria:

- activation criteria are defined by `HTP-010`;
- playback-owned values can express direct-compatible HDR and tone-map-required
  cases.

Status: Blocked pending `HTP-010`.

## M2 - Transcode Tone-Mapping Strategy

Exit criteria:

- activation criteria are defined by `HTP-010`;
- software and hardware FFmpeg strategies are deterministic and testable;
- CPU fallback behavior is explicit.

Status: Blocked pending `HTP-010`.

## M3 - Verification And Closeout

Exit criteria:

- final gates pass with fresh evidence or the lane records explicit blockers;
- docs and `WORKSTREAM.json` reflect active/completed/deferred status;
- follow-ons are split or explicitly deferred.

Status: Blocked pending implementation plan.
