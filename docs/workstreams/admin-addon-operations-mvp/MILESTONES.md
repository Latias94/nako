# Admin Addon Operations MVP — Milestones

Status: Active
Last updated: 2026-05-21

## M0 — Contract And Goal Baseline

Exit criteria:

- Workstream is indexed and set as the current project goal.
- MVP routes and non-goals are explicit.
- Unregister lifecycle policy is decided before implementation.

## M1 — Lifecycle Mutation

Exit criteria:

- Admin API has explicit enable/disable mutation.
- Admin API has unregister/delete behavior with documented semantics.
- Runtime Addon Token access fails for disabled or unregistered Addons.
- SQLite/PostgreSQL semantics stay aligned for lifecycle state changes.

## M2 — Health And Diagnostics

Exit criteria:

- Admin Addon Health Check exists and is redaction-safe.
- Admin Addon surface read models are shaped for UI use.
- Resource-call diagnostics are bounded and safe.
- Addon Sidecars never receive admin bearer tokens.

## M3 — Closeout

Exit criteria:

- All AAO tasks are complete or split into named follow-ons.
- Evidence and gates are fresh.
- API docs and Addon Author Guide explain the shipped operator behavior.
