# Source fingerprint hash scheduler integration first slice

## Goal

Wire queued source fingerprint hash durable jobs into the existing server
scheduler path so one runnable `JobKind::SourceFingerprintHash` job can be
claimed and executed by the already shipped app-service executor command. This
turns the queued contract into scheduler-originated execution without adding
operator/API surfaces or evidence persistence.

## What I Already Know

- `JobKind::SourceFingerprintHash`, redaction-safe durable input, `disk.scan`
  resource-class mapping, internal enqueue, queued execution planning,
  redaction-safe summary JSON, and an internal single-job executor command are
  already shipped.
- `SourceFingerprintHashAppService::execute_source_fingerprint_hash_job` can
  claim one explicit job id through `DurableJobRuntime`, execute VFS-backed
  hashing, and persist `SourceFingerprintHashJobSummary`.
- `docs/architecture/CONTROL_PLANE.md`, `docs/architecture/STORAGE_VFS.md`, and
  `docs/architecture/LANES.md` all name automatic scheduler/operator integration
  as the next follow-on.
- Server specs require durable orchestration to live in app runtime/scheduler
  boundaries and prohibit hidden raw `tokio::spawn` loops.

## Requirements

- Reuse the existing durable job scheduler/runtime boundary instead of creating
  a source-hash-specific background loop.
- Add source fingerprint hash support to the scheduler dispatch path for one
  claimed runnable job at a time.
- Dispatch must call the existing `SourceFingerprintHashAppService`
  executor command, not duplicate VFS hashing or summary serialization logic.
- Jobs must continue to use the existing `disk.scan.source_fingerprint_hash`
  resource class and mapped `disk.scan` budget behavior.
- Persisted job input, summaries, errors, diagnostics, and logs must not include
  raw `StorageUri`, Source Locator, local path, backend URL, etag, credential,
  raw digest, fingerprint, or hash material.
- Add focused tests proving a queued source fingerprint hash job can be executed
  through the scheduler path and remains redaction-safe.

## Acceptance Criteria

- [x] Scheduler-originated source fingerprint hash execution marks the job
  succeeded and persists redaction-safe `summary_json`.
- [x] A succeeded source fingerprint hash job is no longer claimable.
- [x] The scheduler does not start a new source-hash-specific runtime loop,
  API route, schema migration, evidence writer, duplicate reconciliation, or
  automatic Media Source merge behavior.
- [x] Focused `nako-server` scheduler/source-hash tests pass under nextest.
- [x] `cargo check -p nako-server --tests` passes.
- [x] Docs/specs are updated if the scheduler contract changes.

## Out Of Scope

- No Admin/Public API route or DTO.
- No operator UI or manual trigger endpoint.
- No schema migration.
- No evidence persistence outside durable job `summary_json`.
- No duplicate relationship mutation or automatic Media Source merge behavior.
- No broad job-kind scheduler rewrite beyond the minimum needed source hash
  dispatch hook.

## Technical Notes

- Likely files:
  - `crates/nako-server/src/app/runtime.rs`
  - `crates/nako-server/src/app/job_runtime.rs`
  - `crates/nako-server/src/app/jobs.rs`
  - `crates/nako-server/src/app/source_hash.rs`
  - `crates/nako-server/src/app/tests/startup.rs`
  - `crates/nako-server/src/app/tests/source_hash.rs`
- Current code findings:
  - `runtime_budget_class_for_job_resource_class` already maps
    `JobKind::SourceFingerprintHash` plus
    `disk.scan.source_fingerprint_hash` onto the `disk.scan` runtime budget.
  - `JobAppService::schedule_queued_library_scans` is the nearest existing
    scheduler shape: acquire scan permit, list claimable jobs, skip storage
    admission-blocked candidates, claim lease by job id/kind/resource/library,
    then `RuntimeSupervisor::spawn_job`.
  - `finish_claimed_library_scan_job` schedules a follow-up tick after each
    claimed scan job; source hash scheduling should make an explicit decision
    whether to share or mirror that follow-up behavior.
- Specs/docs to keep in view:
  - `.trellis/spec/nako-server/backend/directory-structure.md`
  - `.trellis/spec/nako-server/backend/logging-guidelines.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
  - `.trellis/spec/nako-library/backend/quality-guidelines.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/LANES.md`

## Implementation Plan

1. Inspect the existing durable scheduler dispatch path and library scan job
   scheduler tests to identify the smallest hook for new job-kind dispatch.
2. Add a source fingerprint hash branch that delegates to
   `SourceFingerprintHashAppService::execute_source_fingerprint_hash_job`.
3. Add focused tests using local VFS test storage to prove scheduler-originated
   execution and summary redaction.
4. Run focused nextest and `cargo check -p nako-server --tests`; update docs or
   specs only if the shipped scheduler contract changes.
