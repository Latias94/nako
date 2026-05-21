# Managed Import Staging — TODO

Status: Active
Last updated: 2026-05-21

Task IDs use the `MIS` prefix.

## M0 — Lane Open

- [x] MIS-010 [owner=planner] [deps=post-rpd PRPH-050,nfo-link-authority closeout] [scope=docs/workstreams/managed-import-staging]
  Goal: Open the Managed Import Staging lane with boundaries, non-goals,
  first executable slice, gates, and parent routing.
  Validation: workstream docs agree and `WORKSTREAM.json` is valid JSON.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: Execute MIS-020.

## M1 — Durable Import Artifact Domain

- [ ] MIS-020 [owner=codex] [deps=MIS-010] [scope=crates/taru-core,crates/taru-db]
  Goal: Add Managed Import artifact IDs, states, source kinds, repository
  traits, SQLite/PostgreSQL migrations, and backend-neutral contract tests.
  Validation: `cargo nextest run -p taru-db managed_import --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Evidence: core domain records, repository trait, migrations, DB contract
  tests.
  Handoff: Wire app/service diagnostics in MIS-030.

## M2 — App Service Diagnostics

- [ ] MIS-030 [owner=codex] [deps=MIS-020] [scope=crates/taru-server]
  Goal: Add server app service methods to create/list redacted Managed Import
  artifacts without fetching external bytes or writing library files.
  Validation: focused server app tests prove redaction, library scoping, and no
  library mutation.
  Evidence: `taru-server` app tests and service boundary.
  Handoff: Add non-mutating promotion plan preview in MIS-040.

## M3 — Promotion Plan Preview

- [ ] MIS-040 [owner=codex] [deps=MIS-020,MIS-030,nfo-link-authority] [scope=taru-core,taru-server]
  Goal: Produce a non-mutating promotion plan that explains target library,
  destination locator, duplicate/link hints, NFO authority preview, provider
  identity hints, and blocked reasons.
  Validation: tests prove planning does not create/copy/link/delete library
  files and records explicit blocked reasons.
  Evidence: promotion plan model and app tests.
  Handoff: Decide whether apply belongs here or a split follow-on.

## M4 — Apply Split Decision

- [ ] MIS-050 [owner=planner] [deps=MIS-040] [scope=docs/workstreams/managed-import-staging]
  Goal: Decide whether first promotion apply can be safely implemented in this
  lane or must split to `link-apply-and-import-promotion`.
  Validation: DESIGN/HANDOFF document rollback, cleanup, audit, and operator
  confirmation requirements.
  Evidence: updated split decision.
  Handoff: Implement apply or open follow-on.

## M5 — Closeout

- [ ] MIS-060 [owner=planner] [deps=MIS-050] [scope=docs/workstreams/managed-import-staging]
  Goal: Close or split Managed Import Staging.
  Validation: evidence gates are fresh; post-RPD umbrella points to next lane.
  Evidence: closeout journal and parent handoff.
  Handoff: Return to `post-rpd-product-hardening` for next lane scoring.