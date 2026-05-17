# Admin Playback Runtime Diagnostics TODO

Status: Completed
Last updated: 2026-05-18

## APRD.0 Planning Baseline

- [x] APRD-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-playback-runtime-diagnostics,docs/GOALS.md]
  Goal: Open M56 workstream and record the Admin API, DTO, and redaction
  boundaries.
  Validation: workstream docs exist and reference ADR 0027 direction.
  Evidence: this workstream.
  Handoff: Continue with APRD-020.

## APRD.1 Runtime Diagnostics Read Model

- [x] APRD-020 [owner=codex] [deps=APRD-010] [scope=crates/taru-api/src/admin.rs,crates/taru-server/src/app/playback]
  Goal: Add safe playback runtime diagnostics DTOs and app snapshot support.
  Validation: `cargo check -p taru-api --tests`, focused `taru-api` and
  `taru-server` app tests.
  Evidence: `AdminPlaybackRuntimeDiagnosticsResponse`,
  `PlaybackRuntimeDiagnostics`, and
  `admin_playback_runtime_diagnostics_serializes_safe_summary_fields`.
  Handoff: Keep DTOs admin-owned; do not touch `taru-client-protocol`.

- [x] APRD-030 [owner=codex] [deps=APRD-020] [scope=crates/taru-server/src/http/admin.rs,crates/taru-server/src/http/tests/system.rs,crates/taru-api/src/openapi.rs]
  Goal: Add `GET /admin/v1/playback/runtime` with route-level redaction, auth,
  and public OpenAPI/SDK leakage protection.
  Validation: `cargo check -p taru-server --tests`, focused admin HTTP tests,
  public OpenAPI/SDK leakage checks.
  Evidence: `admin_v1_playback_runtime_reports_safe_diagnostics`, updated
  bearer-auth route protection, and public OpenAPI route exclusion.
  Handoff: Existing Public Client API playback routes remain compatible.

- [x] APRD-040 [owner=codex] [deps=APRD-030] [scope=docs/GOALS.md,docs/workstreams/admin-playback-runtime-diagnostics,docs/workstreams/admin-web-console]
  Goal: Close M56 with evidence and update admin-web-console data-source
  notes.
  Validation: closeout evidence maps every M56 requirement to tests and docs.
  Evidence: updated workstream evidence, GOALS entry, admin-web-console notes,
  and HTTP API docs.
  Handoff: Next likely follow-up is event outbox list/filter or storage
  staging/cache diagnostics.
