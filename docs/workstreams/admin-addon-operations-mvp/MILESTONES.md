# Admin Addon Operations MVP — Milestones

Status: Completed
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

Status: Complete. AAO-040 shipped redaction-safe Addon Health Checks through
`POST /admin/v1/addons/{addon_id}/health-check`. AAO-050 shipped hosted
surface read models through `GET /admin/v1/addons/{addon_id}/surfaces`.
AAO-060 shipped bounded resource-call diagnostics through
`POST /admin/v1/addons/{addon_id}/diagnostics/resource-call`.

Exit criteria:

- Admin Addon Health Check exists and is redaction-safe. Complete.
- Admin Addon surface read models are shaped for UI use. Complete.
- Resource-call diagnostics are bounded and safe. Complete.
- Addon Sidecars never receive admin bearer tokens. Complete for health
  checks, hosted surfaces, and resource-call diagnostics.

## M3 — Closeout

Status: Complete. AAO-070 closed the lane after fresh focused Addon gates,
workspace check, workspace nextest, formatting, and diff evidence. PostgreSQL
opt-in contracts were skipped because `TARU_TEST_POSTGRES_URL` was not set.

Exit criteria:

- All AAO tasks are complete or split into named follow-ons. Complete.
- Evidence and gates are fresh. Complete.
- API docs and Addon Author Guide explain the shipped operator behavior.
  Complete.
