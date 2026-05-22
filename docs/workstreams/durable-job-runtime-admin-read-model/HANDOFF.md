# Durable Job Runtime And Admin Read Model Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M54 is complete. The workstream deepened durable job runtime behavior and added
an Admin API v1 Jobs/Tasks read model.

Completed implementation:

- `nako-server::app::job_runtime` owns common durable job lifecycle handling.
- Library scan, metadata refresh/maintenance, and NFO import/export now use
  that Module.
- `JobListFilter` and SQLite list/filter support back `GET /admin/v1/jobs`.
- `AdminJobListItem` is a redacted list DTO and does not expose raw job input,
  summary, or error payloads.
- Public OpenAPI, TypeScript SDK, and `nako-client-protocol` stayed clean.

## Next Recommended Task

Pick one follow-on Admin API read model:

- playback session list/filter for Playback & Transcode;
- event outbox list/filter for Automation/Webhooks;
- storage staging/cache diagnostics for Storage.

## Constraints

- Keep Public Client API and `nako-client-protocol` unchanged.
- Keep Admin API DTOs in `nako-api::admin`.
- Keep HTTP handlers thin; app/read-model Modules should compose behavior.
- Preserve startup recovery semantics from M41.
- Do not scaffold frontend UI.
- Do not add retry/resume/cancel semantics without a dedicated design slice.
