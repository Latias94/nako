# Durable Job Runtime And Admin Read Model Milestones

Status: Completed
Last updated: 2026-05-17

## M54: Durable Job Runtime And Admin Job List Read Model

Objective:

- Deepen server durable job lifecycle handling.
- Reduce duplicated start/succeed/fail logic in scan, metadata, and NFO
  workflows.
- Add the first Admin API v1 Jobs/Tasks read model.

Deliverables:

- Server-side durable job lifecycle Module.
- Migrated scan, metadata, and NFO workflow job execution paths.
- `JobListFilter` or equivalent repository read model.
- `GET /admin/v1/jobs`.
- Admin-owned job list DTOs in `nako-api::admin`.
- Focused tests and closeout docs.

Exit criteria:

- Existing scan, metadata, and NFO job behavior is preserved.
- Common job lifecycle behavior has one authoritative implementation in
  `nako-server::app::job_runtime`.
- Admin Console can list/filter jobs through `/admin/v1/jobs`.
- Public Client API, public OpenAPI, public SDKs, and
  `nako-client-protocol` remain unchanged.
- Validation gates passed on 2026-05-17.

## Follow-Ons

- Playback session list/filter for Admin API v1.
- Storage runtime diagnostics read model.
- Admin API contract generation, separate from Public Client OpenAPI/SDK.
- Durable retry/resume policy, only after concrete workflows need it.
