# Source fingerprint hash durable executor command first slice

## Goal

Add a narrow internal app-service command that executes one queued source
fingerprint hash durable job by id. The command should claim the job through
`DurableJobRuntime`, reuse the redaction-safe queued execution planner, execute
the hash through VFS, and persist a narrow `SourceFingerprintHashJobSummary` in
`summary_json`.

## What I Already Know

- `JobKind::SourceFingerprintHash`, redaction-safe job input, resource class
  mapping, internal enqueue, queued execution planning, and job summary contract
  already exist.
- `StorageBackendRegistry::backend_for_media_source` can resolve the configured
  `LibraryStorageBackend` for a Media Source.
- `nako-library::SourceFingerprintHashExecutor` can execute partial/full hash
  requests through a `StorageBackend`.
- This task must not start a scheduler loop or expose an API route.

## Requirements

- Extend `SourceFingerprintHashAppService` with an internal command method that
  accepts `JobId`.
- The method must claim and complete/fail/cancel the durable job through
  `DurableJobRuntime`.
- The operation must:
  - load the claimed/current job by id;
  - use `prepare_source_fingerprint_hash_execution` for job contract and
    redaction-safe request validation;
  - resolve the current Media Source backend through `StorageBackendRegistry`;
  - execute `SourceFingerprintHashExecutor`;
  - convert the report into `SourceFingerprintHashJobSummary`;
  - serialize that summary via `DurableJobRuntime::serialize_summary`.
- The summary JSON must not include source locator, `StorageUri`, local path,
  backend URL, etag, credential, raw digest, or fingerprint material.
- Add focused app tests proving:
  - full or partial command execution succeeds and persists summary JSON;
  - the job is no longer claimable after success;
  - summary JSON stays redaction-safe.

## Acceptance Criteria

- `cargo check -p nako-server --tests` passes.
- `cargo nextest run -p nako-server source_fingerprint_hash_execute --no-fail-fast`
  passes.
- Existing source hash enqueue/prepare tests remain green.
- Docs/specs mark the internal command as shipped while scheduler/API/evidence
  persistence remain follow-ons.

## Out Of Scope

- No automatic scheduler, background worker, runtime supervisor spawn, or
  startup hook.
- No Admin/Public API route or DTO.
- No schema migration.
- No evidence persistence outside durable job `summary_json`.
- No duplicate relationship mutation or automatic Media Source merge behavior.

## Technical Notes

- Main files likely touched:
  - `crates/nako-server/src/app/source_hash.rs`
  - `crates/nako-server/src/app/composition.rs`
  - `crates/nako-server/src/app/tests/source_hash.rs`
- Relevant docs/specs:
  - `.trellis/spec/nako-server/backend/*`
  - `.trellis/spec/nako-library/backend/quality-guidelines.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/LANES.md`
