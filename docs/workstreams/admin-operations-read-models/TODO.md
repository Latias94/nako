# Admin Operations Read Models TODO

Status: Completed
Last updated: 2026-05-18

## AORM.0 Planning Baseline

- [x] AORM-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-operations-read-models]
  Goal: Open M57-M59 workstream and freeze route, redaction, and boundary
  rules.
  Validation: workstream docs exist and agree with ADR 0027.
  Evidence: `docs/workstreams/admin-operations-read-models/DESIGN.md`.
  Handoff: Continue with AORM-020.

## AORM.1 Event Outbox Read Model

- [x] AORM-020 [owner=codex] [deps=AORM-010] [scope=crates/nako-core/src/repository/jobs.rs,crates/nako-db/src/event_outbox.rs,crates/nako-db/src/tests.rs]
  Goal: Add event outbox list/filter repository support.
  Validation: `cargo check -p nako-db --tests`, focused `nako-db` event
  outbox tests.
  Evidence: `OutboxEventListFilter` and
  `sqlite_store_lists_outbox_events_with_filters_and_pagination`.
  Handoff: No schema migration should be needed.

- [x] AORM-030 [owner=codex] [deps=AORM-020] [scope=crates/nako-api/src/admin.rs,crates/nako-server/src/app/webhooks.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/query.rs,crates/nako-server/src/http/tests]
  Goal: Add `GET /admin/v1/events` with admin-owned redacted DTOs.
  Validation: `cargo check -p nako-api --tests`, `cargo check -p
  nako-server --tests`, focused `nako-server` admin HTTP tests, public
  OpenAPI/SDK leakage checks.
  Evidence: `AdminOutboxEventListResponse`,
  `admin_v1_events_lists_filters_and_redacts_payloads`, and admin auth tests.
  Handoff: Do not expose `payload_json`, `idempotency_key`, or raw
  `last_error`.

## AORM.2 Storage Staging Diagnostics

- [x] AORM-040 [owner=codex] [deps=AORM-030] [scope=crates/nako-api/src/admin.rs,crates/nako-server/src/app/storage.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/query.rs,crates/nako-server/src/http/tests]
  Goal: Add `GET /admin/v1/storage/staging` with redacted staging manifest
  diagnostics and storage cache/budget summary.
  Validation: focused API/server tests.
  Evidence: `AdminStorageStagingDiagnosticsResponse`,
  `VfsCacheSummary`, and
  `admin_v1_storage_staging_lists_filters_and_redacts_paths`.
  Handoff: Full VFS cache object/failure listing remains a follow-on; this
  slice intentionally exposes only safe cache counters.

## AORM.3 Sanitized Server Config Diagnostics

- [x] AORM-050 [owner=codex] [deps=AORM-040] [scope=crates/nako-api/src/admin.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/tests]
  Goal: Add `GET /admin/v1/system/config` with sanitized server configuration
  diagnostics.
  Validation: focused API/server tests.
  Evidence: `AdminServerConfigDiagnosticsResponse` and
  `admin_v1_system_config_reports_sanitized_configuration`.
  Handoff: No runtime config edit route in this slice.

## AORM.4 Closeout

- [x] AORM-060 [owner=codex] [deps=AORM-050] [scope=docs/GOALS.md,docs/api/HTTP_API.md,docs/workstreams/admin-web-console,docs/workstreams/admin-operations-read-models]
  Goal: Close M57-M59 with evidence and update admin-web-console data-source
  notes.
  Validation: closeout evidence maps every objective requirement to tests and
  docs.
  Evidence: updated workstream evidence, GOALS entry, HTTP API docs, and
  admin-web-console notes.
  Handoff: Next likely follow-up is catalog governance or deeper extension
  operations.
