# Admin Web Addon Credential and Grant Onboarding Milestones

Status: Completed
Last updated: 2026-05-22

## M1 Planning Baseline

Status: completed.

Exit criteria:

- Workstream opened with explicit raw-token and grant safety rules.
- Top-level trackers point to this active lane.

## M2 Contract and Data Seam

Status: completed.

Exit criteria:

- Generated Admin API contract includes explicit one-time token issue/rotation
  response types and grant replacement request type.
- Admin Web client/data-source exposes narrow action methods.
- Tests prove raw tokens are not part of load data.

## M3 Operator UI

Status: completed.

Exit criteria:

- Admin Web can issue, rotate, and revoke Addon Tokens.
- Admin Web can replace accepted Addon Grants.
- Enable readiness checklist is rendered for the selected Addon.

## M4 Closeout

Status: completed.

Exit criteria:

- Rust and Admin Web validation gates pass.
- Docs explain the credential/grant flow and one-time raw token handling.
- Recommended follow-on is recorded.
