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

Status: In progress. AAO-020 shipped explicit enable/disable mutation through
`PATCH /admin/v1/addons/{addon_id}/status`, with SQLite/PostgreSQL repository
parity and runtime Addon Token access failing while disabled.

Exit criteria:

- Admin API has explicit enable/disable mutation. Complete.
- Admin API has unregister/delete behavior with documented semantics.
- Runtime Addon Token access fails for disabled Addons. Unregistered behavior
  remains AAO-030.
- SQLite/PostgreSQL semantics stay aligned for enable/disable status mutation.

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
