# Admin Playback Session Read Model TODO

Status: Completed
Last updated: 2026-05-18

## APS.0 Planning Baseline

- [x] APS-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-playback-session-read-model]
  Goal: Open M55 workstream and record API, DTO, and redaction boundaries.
  Validation: workstream docs exist and reference ADR 0027 direction.
  Evidence: this workstream.
  Handoff: Continue with APS-020.

## APS.1 Session List Read Model

- [x] APS-020 [owner=codex] [deps=APS-010] [scope=crates/nako-core/src/repository/transcode.rs,crates/nako-db/src/playback.rs,crates/nako-db/src/tests.rs]
  Goal: Add transcode session list/filter repository support.
  Validation: `cargo check -p nako-db --tests`, focused `nako-db` playback
  tests.
  Evidence: `TranscodeSessionListFilter`, SQLite list/filter implementation,
  and `sqlite_store_lists_transcode_sessions_with_filters_and_pagination`.
  Handoff: No schema migration was needed.

- [x] APS-030 [owner=codex] [deps=APS-020] [scope=crates/nako-api/src/admin.rs,crates/nako-server/src/app/playback/mod.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/query.rs,crates/nako-server/src/http/tests]
  Goal: Add `GET /admin/v1/playback/sessions` with admin-owned redacted DTOs.
  Validation: `cargo check -p nako-api --tests`, `cargo nextest run -p
  nako-api --no-fail-fast`, `cargo check -p nako-server --tests`, focused
  `nako-server` admin HTTP tests, public OpenAPI/SDK leakage checks.
  Evidence: `AdminPlaybackSessionListItem`, `GET
  /admin/v1/playback/sessions`, route tests covering filtering, redaction, and
  auth protection.
  Handoff: Existing Public Client API playback session routes remain
  compatible.

- [x] APS-040 [owner=codex] [deps=APS-030] [scope=docs/GOALS.md,docs/workstreams/admin-playback-session-read-model,docs/workstreams/admin-web-console]
  Goal: Close M55 with evidence and update admin-web-console data-source notes.
  Validation: closeout evidence maps every M55 requirement to tests and docs.
  Evidence: updated workstream evidence, GOALS entry, and admin-web-console
  notes.
  Handoff: Next likely follow-up is playback runtime diagnostics or event
  outbox list/filter.
