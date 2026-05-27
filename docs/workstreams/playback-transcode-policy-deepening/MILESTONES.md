# Playback Transcode Policy Deepening - Milestones

Status: Active
Last updated: 2026-05-27

## M0 - Workstream Open

Exit criteria:

- Scope, Jellyfin-class feature pressure, non-goals, and risks are documented.
- ADR 0038 records the planner/policy/runtime/engine decision.
- `WORKSTREAM.json` is valid.

## M1 - Characterization

Exit criteria:

- Existing direct/remux/HLS/Playback Session behavior is protected by tests.
- Current gaps against Jellyfin-class feature pressure are documented.
- Redaction and compatibility expectations are explicit before refactoring.

## M2 - Playback Planner

Exit criteria:

- Direct/remux/HLS selection is centralized behind a planner Interface.
- HTTP routes adapt planner output rather than own compatibility logic.
- Planner returns typed reasons suitable for safe client/admin presentation.

## M3 - Transcode Policy

Exit criteria:

- Hardware acceleration is modeled by decode/filter/encode stage selection and
  fallback policy, not a boolean.
- Subtitle strategy, bitrate, output format, and selected streams are policy
  outputs.
- FFmpeg-specific strings remain below the policy seam.

## M4 - Runtime Inventory And Engine Adapter

Exit criteria:

- FFmpeg/runtime/hardware capabilities are represented as redaction-safe
  snapshots.
- FFmpeg CLI is an Adapter behind typed start/cancel/progress semantics.
- Future remote worker or optimized-version adapters can satisfy the same
  engine Interface.

## M5 - Admin Settings And Artifact Lifecycle

Exit criteria:

- Admin settings and diagnostics use the same runtime/policy facts as playback.
- Segment deletion, throttling, cleanup, and transcode artifact lifecycle are
  explicit and testable.
- Public Client contracts remain safe and separate from Admin diagnostics.

Status: Complete via PTP-070. Playback runtime settings are persisted and
applied on startup; Admin diagnostics expose artifact lifecycle/throttle
evidence; startup artifact cleanup is rooted and covered by tests.

## M6 - Closeout

Exit criteria:

- Route compatibility and browser playback tickets are preserved.
- Evidence is fresh and recorded.
- Remaining breadth is split into named follow-on lanes.
