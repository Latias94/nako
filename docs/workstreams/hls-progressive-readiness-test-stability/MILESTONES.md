# HLS Progressive Readiness Test Stability - Milestones

Status: Closed
Last updated: 2026-05-31

## M0 - Scope And Repro Freeze

Exit criteria:

- HRLB-040 gate failure is recorded with exact failing tests.
- Non-goals are explicit.
- The follow-on is linked from playback architecture and workstream indexes.

Status: Completed by `HPRTS-010`.

## M1 - Stabilize Progressive Readiness Gate

Exit criteria:

- The root cause is classified as test harness timing, fixture contention, or
  real runtime behavior.
- The fix is behavior-preserving, or planner approval exists for any runtime
  behavior change.
- Both progressive readiness tests pass individually.
- The default full HLS gate passes.

Status: Completed by `HPRTS-020`.

## M2 - Closeout And HRLB Retry

Exit criteria:

- Fresh HLS, formatting, and diff gates pass.
- This workstream records closeout evidence.
- HRLB-040 is rerun and HRLB is closed only if its required gates pass.

Status: Completed by `HPRTS-030`.
