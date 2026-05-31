# Transcode Interface And Runtime Plan Deepening - Milestones

Status: Active
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

Exit criteria:

- workstream files exist and agree on target state;
- architecture maps link the workstream;
- HDR implementation is explicitly serialized behind the Interface deepening.

Status: Done.

## M1 - HLS Runtime Plan Interface

Exit criteria:

- `nako-transcode` owns a higher-level HLS runtime plan Interface;
- server HLS source context no longer manually orders profile identity,
  request variant, execution policy, and artifact planning;
- pure transcode tests prove the plan shape.

Status: Done.

## M2 - FFmpeg Adapter Interface Ratchet

Exit criteria:

- low-level FFmpeg request and builder details are not part of the broad public
  crate Interface used by server;
- server HLS/remux paths enter through planned execution adapters;
- transcode command planning remains directly testable inside the crate.

Status: Done.

## M3 - Closeout And HDR Unblock

Exit criteria:

- fresh transcode/server gates pass;
- docs and `WORKSTREAM.json` reflect the final Interface shape;
- HDR `HTP-030` is either explicitly unblocked or replanned around a narrower
  follow-on.

Status: Pending `TIRP-040`.
