# Adaptive HLS Source-Aware Ladder Runtime Milestones

Status: Active
Last updated: 2026-05-28

## Milestone 1 - Workstream Opened

Status: Done

- Created durable docs for the source-aware adaptive HLS runtime lane.
- Split adaptive breadth into this lane's immediate targets and later
  follow-ons.

## Milestone 2 - Source-Aware Ladder Contract

Status: Pending

- Adaptive fMP4 runtime chooses renditions from typed source/client facts.
- Ladder identity is stable enough for session reuse and artifact
  reconstruction.

## Milestone 3 - Audio-Presence-Aware FFmpeg Planning

Status: Pending

- Audio-bearing adaptive fMP4 plans remain valid.
- Video-only adaptive fMP4 plans do not reference non-existent audio streams.

## Milestone 4 - Server Runtime Integrated And Closed

Status: Pending

- Server staging, artifact serving, playlist rewrite, cleanup, and redaction
  tests consume the same ladder contract.
- Focused gates pass and the workstream is closed.
