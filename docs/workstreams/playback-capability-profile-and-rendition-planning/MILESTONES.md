# Playback Capability Profile And Rendition Planning Milestones

Status: Completed
Last updated: 2026-05-28

## M1 - Workstream Opened

Exit criteria:

- Durable docs exist.
- Refactor brief names deletion, boundary, testing, and risk plans.
- Workstream index links the lane.

Status: Complete.

## M2 - Rendition Boundary Owns Selected Output

Exit criteria:

- `PlaybackDecision` has one selected-output field:
  `PlaybackRenditionPlan`.
- Public DTO mapping preserves the existing safe `direct_play` and
  `transcode_plan` response shape.
- Planner tests assert rendition variants directly.

Status: Complete.

## M3 - Target Profile Owns Transcode Profile Identity

Exit criteria:

- `PlaybackProfile` is deleted.
- `PlaybackTargetProfile` builds remux and HLS transcode profiles.
- Server remux/HLS request identities use target-profile identity.

Status: Complete.

## M4 - Lane Verified And Closed

Exit criteria:

- Focused Rust gates pass.
- JSON/fmt/diff checks pass.
- Closeout docs record shipped changes, evidence, follow-ons, and residual
  risk.

Status: Complete.
