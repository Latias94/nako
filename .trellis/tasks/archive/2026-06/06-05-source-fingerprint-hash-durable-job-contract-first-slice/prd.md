# Source Fingerprint Hash Durable Job Contract First Slice

## Goal

Prepare the durable-job contract for future source fingerprint hash queue and
operator integration without executing VFS reads, adding API routes, changing
schema, or mutating source reconciliation behavior.

## Requirements

- Add a persisted `JobKind` for source fingerprint hash work in `nako-core`,
  with `as_str` / `parse` round-trip coverage.
- Add a redaction-safe durable job input contract for source fingerprint hash
  work that carries:
  - `library_id`;
  - `source_id`;
  - source scheme only;
  - `SourceFingerprintHashMode`.
- Job input must not include raw `StorageUri`, Source Locator, local path,
  backend URL, credential, etag, fingerprint, or raw hash material.
- Add a runtime resource-class mapping for future source fingerprint hash jobs
  so they share the disk scan budget rather than inventing a hidden runtime.
- Keep this slice contract-only:
  - no enqueue service;
  - no job scheduler/executor;
  - no Admin/Public API;
  - no DB migration;
  - no source lookup or VFS hash execution;
  - no duplicate relationship mutation or automatic Media Source merge.
- Update relevant specs and architecture maps so the next follow-on can build
  enqueue/execution behavior against this contract.

## Acceptance Criteria

- [ ] `JobKind::SourceFingerprintHash` round-trips through `as_str` and
      `parse`.
- [ ] Source fingerprint hash job input serializes without locator/path/hash
      leakage and can carry partial/full modes.
- [ ] `nako-server` runtime budget mapping accepts the new job kind/resource
      class and maps it to the existing disk scan budget class.
- [ ] Focused `nako-core`, `nako-library`, and `nako-server` checks/tests pass.
- [ ] Specs/architecture docs distinguish this contract from actual
      enqueueing, execution, API, persistence schema, and source reconciliation.

## Definition Of Done

- Rust code and tests are committed.
- Task evidence records verification commands.
- Task is archived.

## Technical Approach

- Put the persisted job kind in `crates/nako-core/src/job.rs` because job kind
  parsing is a cross-crate durable job contract.
- Put the hash job input beside the existing hash planner/executor in
  `crates/nako-library/src/source_hash.rs`, since it reuses
  `SourceFingerprintHashMode` and remains library/source-fingerprint scoped.
- Put runtime budget mapping in `crates/nako-server/src/app/runtime.rs`, mapping
  a source-fingerprint hash resource class to `disk.scan`.

## Out Of Scope

- Durable enqueue route/service.
- Scheduler loop or executor.
- VFS reads and hash evidence persistence.
- Admin/Public API or generated contracts.
- Database schema or repository methods.
- Automatic duplicate merge or source identity upgrade.

## Technical Notes

- Previous shipped planner:
  `.trellis/tasks/archive/2026-06/06-05-source-fingerprint-hash-scheduling-diagnostics-first-slice/`.
- Relevant files:
  - `crates/nako-core/src/job.rs`
  - `crates/nako-library/src/source_hash.rs`
  - `crates/nako-server/src/app/runtime.rs`
  - `.trellis/spec/nako-library/backend/quality-guidelines.md`
  - `.trellis/spec/nako-core/backend/quality-guidelines.md`
  - `.trellis/spec/nako-server/backend/directory-structure.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
