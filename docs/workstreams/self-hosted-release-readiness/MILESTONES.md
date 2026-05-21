# Self-Hosted Release Readiness — Milestones

Status: Completed
Last updated: 2026-05-21

## M0 — Baseline And Scope

Exit criteria:

- [x] Workstream is opened with durable docs.
- [x] Current repo release gate baseline is recorded.
- [x] Task ledger preserves the full user objective.

## M1 — Local Release Gate

Exit criteria:

- [x] A first-party local release gate entrypoint exists.
- [x] Fast/focused and broader release modes are documented.
- [x] Formatting and diff checks are included.

## M2 — PostgreSQL Contract Harness

Exit criteria:

- [x] PostgreSQL contract tests can be run through a repo-owned local harness.
- [x] Harness safely creates and cleans temporary PostgreSQL state under
  `target/`.
- [x] CI-ready PostgreSQL contract job shape is documented or implemented.

## M3 — API/SDK/Redaction Gate

Exit criteria:

- [x] Admin/Public API contract checks are part of release readiness.
- [x] SDK/OpenAPI generation or synchronization checks are documented and
  runnable.
- [x] Redaction inventory and tests cover public/admin self-host diagnostics.

## M4 — Deployment Examples

Exit criteria:

- [x] SQLite self-host example exists.
- [x] PostgreSQL self-host example exists.
- [x] Operator configuration docs cover auth, DB, artifact roots, staging
  roots, Addons, Webhooks, Playback Runtime, and diagnostics.

## M5 — Backup/Restore/Upgrade

Exit criteria:

- [x] SQLite backup/restore runbook exists.
- [x] PostgreSQL backup/restore runbook exists.
- [x] Artifact root, NFO sidecar, cache, staging, and secret boundaries are
  documented.
- [x] Migration and upgrade expectations are explicit.

## M6 — End-To-End Self-Host Smoke

Exit criteria:

- [x] A self-host smoke path exercises library, metadata/NFO, Addon artwork,
  Managed Artwork, public image serving, playback, and diagnostics.
- [x] SQLite path is runnable locally.
- [x] PostgreSQL path or equivalent contract gate is runnable or explicitly
  documented.

## M7 — Closeout

Exit criteria:

- [x] Workstream TODO is complete or residual work is split into follow-ons.
- [x] Evidence and gates prove every explicit requirement from the goal.
- [x] `WORKSTREAM.json` status is completed.
