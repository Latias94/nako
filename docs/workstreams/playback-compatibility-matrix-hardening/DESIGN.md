# Playback Compatibility Matrix Hardening

Status: Active
Last updated: 2026-05-31

## Why This Lane Exists

`nako-playback` now owns playback decisions, output requirements, audio output
requirements, and color pipeline requirements. The decisions are covered by
focused tests, but the growing compatibility space is not yet represented as a
single matrix that proves interactions between container support, codec support,
HDR capability, audio channel limits, downmix intent, and normalization intent.

That makes future HDR, audio, and transcode work harder to review. A change can
look correct in one focused test while shifting another Direct Play / Remux /
Transcode decision.

## Target State

When this workstream closes:

- `nako-playback` has a table-driven compatibility matrix for representative
  Direct Play, Remux, and HLS Transcode cases;
- HDR tone-map-required sources are proven not to select Remux;
- audio channel/downmix/normalization requirements are proven to travel with
  HLS Transcode decisions;
- container/codec incompatibility reasons are covered without relying on server
  orchestration tests;
- no cross-crate Interface changes are required.

## In Scope

- `nako-playback` tests and small playback-only helpers when they reduce matrix
  duplication;
- playback decision reason coverage;
- playback-owned requirement propagation assertions.

## Out Of Scope

- `nako-transcode` policy, profile, pipeline, or FFmpeg command planning;
- `nako-server` HLS/remux orchestration;
- Public Client API DTO shape;
- persisted user preferences or device profile databases;
- web/mobile/native player behavior.

## Architecture Direction

Keep `nako-playback` as the pure decision planner. The matrix should make the
planner Interface more trustworthy without teaching playback about FFmpeg,
staging, HLS artifacts, or server runtime admission.

The deletion test is: if the matrix were deleted, future maintainers would have
to rediscover compatibility interactions through scattered focused tests and
server regressions. The matrix should concentrate those interactions in one
playback-only test surface.
