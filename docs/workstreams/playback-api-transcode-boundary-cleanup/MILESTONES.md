# Milestones

## M1 - API Crate Dependency Removed

Status: Done

Exit criteria:

- `nako-api` has no direct `nako-transcode` dependency.
- Public Client playback decision JSON is unchanged for transcode decisions.
- Admin playback/config JSON keeps previous hardware and readiness values.
- Server-side mapping owns runtime-to-DTO conversions.

## M2 - Follow-On Boundary Decision Recorded

Status: Done

Exit criteria:

- Remaining playback planner dependency on transcode types is inventoried.
- Follow-on scope is either rejected with rationale or split into a new bounded
  task/lane.

## M3 - Lane Verified

Status: Done

Exit criteria:

- Focused `nako-api` and `nako-server` gates pass.
- Workstream evidence includes commands and results.
- Closeout records whether API DTO ownership is now stable enough for future
  HDR, audio, and hardware scheduler work.
