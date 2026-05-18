# Durable Job Runtime And Admin Read Model TODO

Status: Completed
Last updated: 2026-05-17

## JRM.0 Planning Baseline

- [x] JRM-010 [owner=planner] [deps=none] [scope=docs/workstreams/durable-job-runtime-admin-read-model]
  Goal: Open the M54 workstream and record the server-side architecture
  findings.
  Validation: workstream docs exist and identify scope, non-goals, and first
  executable slice.
  Evidence: this workstream.
  Handoff: Continue with JRM-020.

## JRM.1 Durable Job Runtime

- [x] JRM-020 [owner=codex] [deps=JRM-010] [scope=crates/taru-server/src/app/runtime.rs,crates/taru-server/src/app/jobs.rs,crates/taru-server/src/app/metadata.rs,crates/taru-server/src/app/nfo.rs]
  Goal: Introduce a server-side durable job lifecycle Module that centralizes
  start/succeed/fail handling, typed summary serialization, and supervised job
  diagnostics for scan, metadata, and NFO workflows.
  Validation: `cargo fmt --all -- --check`, `cargo check -p taru-server
  --tests`, focused `taru-server` app runtime/jobs/metadata/NFO tests, and
  `git diff --check`.
  Evidence: `crates/taru-server/src/app/job_runtime.rs` plus focused
  `app::job_runtime` and NFO tests. Summary serialization failure now marks the
  durable job failed instead of leaving it running.
  Handoff: Retry/resume/cancel policy remains a follow-on.

## JRM.2 Admin Job Read Model

- [x] JRM-030 [owner=codex] [deps=JRM-020] [scope=crates/taru-core,crates/taru-db,crates/taru-api,crates/taru-server/src/http/admin.rs]
  Goal: Add `GET /admin/v1/jobs` with a safe admin-owned DTO and repository
  list/filter support.
  Validation: `cargo fmt --all -- --check`, `cargo check -p taru-api --tests`,
  `cargo nextest run -p taru-api --no-fail-fast`, `cargo check -p taru-db
  --tests`, focused `taru-db` job repository tests, `cargo check -p
  taru-server --tests`, focused `taru-server` admin HTTP tests, public
  OpenAPI/SDK leakage checks, `git diff --check`, and no
  `crates/taru-client-protocol` diff.
  Evidence: `JobListFilter`, SQLite list/filter tests, redacted
  `AdminJobListItem`, `GET /admin/v1/jobs`, admin HTTP tests, and public
  OpenAPI/SDK leakage checks.
  Handoff: Existing root-level `GET /jobs/{job_id}` remains compatible.

- [x] JRM-040 [owner=codex] [deps=JRM-030] [scope=docs/GOALS.md,docs/workstreams/durable-job-runtime-admin-read-model,docs/workstreams/admin-web-console]
  Goal: Close M54 with evidence and update admin-web-console docs to mark
  Jobs/Tasks as backed by Admin API read data.
  Validation: close-out evidence maps every M54 requirement to tests and docs.
  Evidence: updated workstream evidence, GOALS entry, and admin-web-console
  data-source notes.
  Handoff: Next likely follow-up is playback session list/filter or event
  outbox list/filter.
