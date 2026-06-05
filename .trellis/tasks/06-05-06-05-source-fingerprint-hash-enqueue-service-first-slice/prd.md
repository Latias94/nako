# Source Fingerprint Hash Enqueue Service First Slice

## Goal

Add the first internal server enqueue seam for future source fingerprint hash
queue/operator integration without executing VFS reads, exposing API routes,
changing schema, or mutating source reconciliation behavior.

## Requirements

- Add a server app service entry point that can enqueue
  `JobKind::SourceFingerprintHash` for a known Media Source.
- The request carries:
  - `library_id`;
  - `source_id`;
  - `SourceFingerprintHashMode`;
  - optional `JobPriority`.
- The service must load the current `MediaSource` by `source_id`, verify it
  belongs to `library_id`, derive only the locator scheme from the current
  source locator, and build `SourceFingerprintHashJobInput`.
- The persisted job must use:
  - kind `source_fingerprint_hash`;
  - resource class `disk.scan.source_fingerprint_hash`;
  - `library_id`;
  - `source_id`;
  - safe `input_json`.
- Input JSON must not include raw `StorageUri`, Source Locator, local path,
  backend URL, credential, etag, fingerprint, or raw hash material.
- Keep this slice enqueue-only:
  - no scheduler or executor;
  - no VFS read;
  - no Admin/Public API;
  - no DB migration;
  - no hash evidence persistence;
  - no duplicate relationship mutation;
  - no automatic Media Source merge.

## Acceptance Criteria

- [ ] Enqueueing a valid source persists a queued `SourceFingerprintHash` job
      with safe input JSON and the source-fingerprint hash resource class.
- [ ] A source from another library is rejected before enqueuing.
- [ ] Missing source is rejected with `NotFound`.
- [ ] Locator/path/hash-sensitive values do not appear in persisted input JSON
      or error messages.
- [ ] Focused server tests and checks pass.
- [ ] Specs/architecture docs distinguish enqueue support from actual
      scheduling/execution/API/evidence persistence/reconciliation.

## Technical Approach

- Put the app service under `crates/nako-server/src/app/source_hash.rs`.
- Add the service to `NakoAppServices` and expose it through `NakoApp` for app
  tests.
- Reuse `MediaRepository::get_media_source`, `JobRepository::enqueue_job`, and
  `nako_library::SourceFingerprintHashJobInput`.
- Parse the persisted `MediaSource.locator` with `StorageUri::parse` only to
  derive `source_scheme`; do not persist the parsed URI.

## Out Of Scope

- Durable lease claiming or background execution.
- VFS hash execution.
- Admin/Public API routes or generated DTOs.
- Database schema or repository method changes.
- Operator diagnostics beyond persisted job state.
- Automatic duplicate merge or source identity upgrade.

## Technical Notes

- Previous contract slice:
  `.trellis/tasks/archive/2026-06/06-05-source-fingerprint-hash-durable-job-contract-first-slice/`.
- Relevant files:
  - `crates/nako-server/src/app/source_hash.rs`
  - `crates/nako-server/src/app.rs`
  - `crates/nako-server/src/app/composition.rs`
  - `crates/nako-server/src/app/tests/startup.rs`
  - `crates/nako-library/src/source_hash.rs`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
