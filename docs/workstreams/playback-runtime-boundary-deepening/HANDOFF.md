# Playback Runtime Boundary Deepening - Handoff

Status: Completed
Last updated: 2026-05-28

## Current State

This lane is closed after `source-aware-transcode-runtime` closed. PRBD-020
extracted HLS artifact serving into `hls_artifact.rs`, PRBD-030 extracted
server-side playback support evidence context plus runtime diagnostics
collection into `support.rs`, and PRBD-040 recorded an explicit no-split
decision for store traits in this lane.

## Active Task

- None. PRBD-010 through PRBD-050 are complete.

## Decisions Since Last Update

- Keep `PlaybackAppService` as the composition entry point for now.
- Extract cohesive behavior before introducing any new trait.
- Do not add adaptive/fMP4/rsmpeg behavior in this lane.
- Public/Admin API shapes should remain stable unless an intentional redaction
  contract update is explicitly scoped.
- PRBD-030 did not change Admin DTOs or generated contracts; it only moved
  server-side collection.
- Store trait narrowing is not performed in this lane. HLS artifact serving no
  longer needs a store, support lookup is private and too small for a new trait,
  and HLS/remux execution store narrowing should be a future runtime-store lane
  only if execution orchestration grows further.

## Blockers

- None.

## Next Recommended Action

- Open a new lane for adaptive HLS/fMP4, subtitle/audio/HDR maturity, rsmpeg
  adapter feasibility, or HLS/remux execution store-port narrowing when product
  pressure justifies it.
