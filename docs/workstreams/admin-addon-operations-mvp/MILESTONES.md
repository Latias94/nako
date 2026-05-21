# Admin Addon Operations MVP — Milestones

Status: Active
Last updated: 2026-05-21

## M0 — Contract And Goal Baseline

Status: Complete. AAO-010 froze the MVP route contract and chose terminal
`unregistered` lifecycle state with token revocation, grant clearing, and audit
preservation instead of physical deletion.

Exit criteria:

- Workstream is indexed and set as the current project goal.
- MVP routes and non-goals are explicit.
- Unregister lifecycle policy is decided before implementation.

## M1 — Lifecycle Mutation

Status: Complete. AAO-020 shipped explicit enable/disable mutation through
`PATCH /admin/v1/addons/{addon_id}/status`. AAO-030 shipped terminal
unregister through `POST /admin/v1/addons/{addon_id}/unregister`, with
SQLite/PostgreSQL repository parity and runtime Addon Token access failing
while disabled or unregistered.

Exit criteria:

- Admin API has explicit enable/disable mutation. Complete.
- Admin API has unregister/delete behavior with documented semantics.
  Complete.
- Runtime Addon Token access fails for disabled or unregistered Addons.
  Complete.
- SQLite/PostgreSQL semantics stay aligned for lifecycle state changes.
  Complete.

## M2 — Health And Diagnostics

Status: In Progress. AAO-040 shipped redaction-safe Addon Health Checks
through `POST /admin/v1/addons/{addon_id}/health-check`. AAO-050 shipped
hosted surface read models through
`GET /admin/v1/addons/{addon_id}/surfaces`. Resource-call diagnostics remain.

Exit criteria:

- Admin Addon Health Check exists and is redaction-safe. Complete.
- Admin Addon surface read models are shaped for UI use. Complete.
- Resource-call diagnostics are bounded and safe.
- Addon Sidecars never receive admin bearer tokens. Complete for health
  checks; must remain true for diagnostics and hosted surfaces.

## M3 — Closeout

Exit criteria:

- All AAO tasks are complete or split into named follow-ons.
- Evidence and gates are fresh.
- API docs and Addon Author Guide explain the shipped operator behavior.
