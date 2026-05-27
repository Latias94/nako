# Playback Capability Profile Planner - Milestones

Status: Completed
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

Exit criteria:

- ADR 0044 exists and is referenced by this workstream.
- Scope is limited to playback capability planning and decision reports.
- FFmpeg hardware breadth, HDR execution, subtitle execution, and adaptive HLS
  are explicitly follow-ons.

## M1 - Characterization And Profile Model

Exit criteria:

- Existing direct/remux/HLS decisions are characterized before migration.
- `PlaybackTargetProfile` and `PlaybackDecisionReport` exist as pure
  `nako-playback` records.
- Profile identity covers capability and preference facts used for
  session/cache reuse.

## M2 - Planner Migration

Exit criteria:

- `PlaybackPlanner` evaluates profiles and typed compatibility conditions.
- Decisions include a report and do not require callers to infer reasons.
- Server playback orchestration constructs profiles and executes returned
  plans.

## M3 - Decode-Ready Follow-On Split

Exit criteria:

- Hardware decode pipeline, subtitle/HDR maturity, and HLS output maturity are
  split or recorded as explicit follow-ons.
- This lane does not silently expand into FFmpeg backend breadth.

## M4 - Verification And Closeout

Exit criteria:

- Focused playback and server playback gates pass.
- Formatting and diff checks pass.
- Docs match shipped behavior.
- Workstream status is complete or handoff-ready.
