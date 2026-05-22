# Admin Web Addon Onboarding Milestones

Status: Completed
Last updated: 2026-05-22

## M1 Planning Baseline

Status: completed.

Exit criteria:

- Workstream scope, non-goals, and safety rules are documented.
- Top-level goal/roadmap trackers point to this active lane.

## M2 Manifest Registration Data Seam

Status: completed.

Exit criteria:

- Admin Web has a typed `registerAddon` client method.
- Data source exposes a manifest JSON onboarding action.
- Tests prove disabled-by-default registration request shape and safe error
  handling.

## M3 Onboarding UI

Status: completed.

Exit criteria:

- Admin Web renders a paste-and-preview manifest onboarding panel.
- Successful registration updates the Addon Operations model and guides the
  administrator to Install Guide / Health Check continuation.
- UI tests cover success and failure paths.

## M4 Closeout

Status: completed.

Exit criteria:

- Rust and Admin Web validation gates pass.
- Docs explain the difference between registration, install guide, health
  check, and lifecycle automation.
- Follow-ons are recorded without leaving this lane active.
