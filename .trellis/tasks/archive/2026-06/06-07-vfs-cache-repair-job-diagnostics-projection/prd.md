# VFS Cache Repair Job Diagnostics Projection

## Goal

Add a redaction-safe diagnostics projection for Admin Job responses when the job
kind is `VfsCacheRepair`, so operators can understand completed or failed VFS
cache repair jobs without exposing raw job input, raw summary JSON, storage
URIs, paths, backend URLs, credentials, etags, fingerprints, or raw backend
errors.

## What I Already Know

- `docs/architecture/STORAGE_VFS.md` and `docs/architecture/CONTROL_PLANE.md`
  list broader operator diagnostics for VFS cache repair jobs as a follow-on
  after the durable queue/executor/Admin route first slice.
- `AdminJobListItem` and `JobResponse` currently expose only boolean
  `has_input`, `has_summary`, and `has_error` flags for all job kinds.
- Existing tests intentionally prevent raw `input_json`, `summary_json`, and
  `error` from leaking through Admin Job DTOs.
- VFS cache repair job summary JSON is already designed as a redaction-safe
  typed summary, but the generic Admin Jobs surface does not project it.
- The right boundary is a typed, optional diagnostic field on the Admin job DTO,
  not exposing arbitrary raw job payloads or adding destructive repair policy.

## Requirements

- Add an optional Admin job diagnostics projection for `JobKind::VfsCacheRepair`.
- Reuse the existing `AdminVfsCacheRepairJobSummary` DTO shape for successful
  VFS cache repair summary projection when `summary_json` parses correctly.
- Include a redaction-safe failure diagnostic for failed VFS cache repair jobs
  that have `error`, without returning the raw `error` string.
- Keep non-VFS-cache-repair jobs unchanged except for the new optional field
  being `null`/absent by value.
- Keep raw `input_json`, raw `summary_json`, and raw `error` out of
  `AdminJobListItem`, `JobResponse`, and cancellation responses.
- Do not add routes, mutate job execution behavior, enqueue retries, purge
  cache, delete/invalidate cache entries, mutate backend configuration, or
  write library files.
- Update generated Admin contract output if API DTO shape changes.

## Acceptance Criteria

- [x] `AdminJobListItem` includes an optional safe diagnostics field for
      `VfsCacheRepair` jobs.
- [x] `JobResponse` includes the same safe diagnostics field for
      `GET /jobs/{job_id}`.
- [x] Completed VFS cache repair jobs with valid summary JSON expose the typed
      `AdminVfsCacheRepairJobSummary` projection.
- [x] Failed VFS cache repair jobs expose only safe failure facts such as
      `has_error` and a stable safe message/failure class, not the raw error.
- [x] Non-VFS jobs continue to redact raw payloads and have no VFS repair
      diagnostics payload.
- [x] Admin API contract generation/tests are updated.
- [x] Focused API/server tests pass.
- [x] `cargo fmt --all -- --check` and `git diff --check` pass.
- [x] Trellis task validation passes.

## Definition Of Done

- Code, tests, generated contract, docs/spec/task evidence, and context are
  committed together.
- Admin job diagnostics remain redaction-safe by construction and tests.
- No runtime, DB, scheduler, repair policy, or storage mutation behavior changes
  are included.

## Technical Approach

- Add a small DTO in `crates/nako-api/src/admin/operations.rs`, likely
  `AdminJobDiagnostics`, with a VFS cache repair variant/payload.
- Build it from `Job` only for `JobKind::VfsCacheRepair`.
- Parse safe summary JSON into the existing VFS cache repair summary DTO.
- For failed jobs, expose a stable safe diagnostic derived from the presence of
  an error and the VFS cache repair job kind, without copying raw error text.
- Reuse the same builder from `JobResponse::from_job` and
  `AdminJobListItem::from_job` so list/detail/cancel command responses stay
  consistent.
- Update generated Admin contract artifacts if the project generator requires
  it.

## Decision (ADR-lite)

**Context**: The durable VFS cache repair first slice intentionally kept raw job
input, summary, and error private. That preserved safety, but generic Admin Jobs
only show boolean flags, which is too weak for the documented follow-on of
broader operator diagnostics.

**Decision**: Add a typed optional safe diagnostics projection on existing Admin
Job DTOs for `VfsCacheRepair` only. Do not add a new job-detail route or expose
arbitrary job payloads.

**Consequences**: Operators get actionable VFS cache repair status from the
existing Admin Jobs surfaces. Future job kinds can add their own typed safe
projection, but the generic raw payloads remain private.

## Out Of Scope

- No raw `input_json`, raw `summary_json`, or raw `error` exposure.
- No new Admin route.
- No Admin Web UI changes unless generated contract compilation requires them.
- No retry/requeue policy change.
- No automated repair policy.
- No cache purge/delete/invalidation or backend configuration mutation.
- No DB schema or repository method change.

## Technical Notes

- Architecture sources:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
- Existing DTOs:
  - `crates/nako-api/src/admin/operations.rs`
  - `crates/nako-api/src/admin/storage.rs`
- Existing server mapping:
  - `crates/nako-server/src/http/jobs.rs`
  - `crates/nako-server/src/http/admin.rs`
- Existing VFS repair app code/tests:
  - `crates/nako-server/src/app/storage.rs`
  - `crates/nako-server/src/app/tests/storage.rs`
  - `crates/nako-server/src/http/tests/system.rs`
- Predecessor evidence:
  - `.trellis/tasks/archive/2026-06/06-07-vfs-cache-durable-repair-queue-first-slice/prd.md`
  - `.trellis/tasks/archive/2026-06/06-07-vfs-cache-repair-action-plan-policy-refactor/evidence.md`
