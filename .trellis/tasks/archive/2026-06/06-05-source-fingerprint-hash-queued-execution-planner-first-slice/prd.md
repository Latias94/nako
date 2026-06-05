# Source fingerprint hash queued execution planner first slice

## Goal

Add the internal app-service seam that prepares a queued source fingerprint hash
job for future execution. The seam must turn the persisted, redaction-safe job
input back into an in-memory `SourceFingerprintHashRequest` by resolving the
current Media Source locator, while still avoiding VFS reads, job lease
execution, API exposure, schema changes, or evidence persistence.

## What I Already Know

- Source fingerprint hash work already has `JobKind::SourceFingerprintHash`,
  `SourceFingerprintHashJobInput`, and the
  `disk.scan.source_fingerprint_hash` resource class.
- The enqueue service persists only `library_id`, `source_id`, `source_scheme`,
  and mode. It intentionally does not persist raw `StorageUri` or source
  locator text.
- Future execution must resolve the current source locator by source id before
  reading bytes through VFS.
- Marking queued hash jobs succeeded before a real executor exists would be
  misleading, so this slice should prepare but not run or claim durable jobs.

## Requirements

- Add a `nako-server` app-service method that accepts a persisted source
  fingerprint hash `Job` and prepares an in-memory execution request.
- Validate the job kind is `SourceFingerprintHash` and the resource class is
  `disk.scan.source_fingerprint_hash`.
- Deserialize `SourceFingerprintHashJobInput` from `job.input_json`.
- Validate job-level `library_id` and `source_id`, when present, match the
  deserialized input.
- Load the current `MediaSource` by the input source id and verify it still
  belongs to the input library id.
- Parse the current source locator into `StorageUri` only inside the in-memory
  planner output.
- Verify the current locator scheme still matches the persisted
  `source_scheme`.
- Return a prepared record containing job id, library id, source id, source
  scheme, mode, and the in-memory `SourceFingerprintHashRequest`.
- Keep all failure messages redaction-safe: no raw locator, local path,
  credential, query string, fingerprint, or raw JSON payload echo.

## Acceptance Criteria

- App tests cover successful preparation and exact request mode/URI recovery.
- App tests cover wrong job kind/resource class rejection.
- App tests cover malformed or missing input without leaking raw input content.
- App tests cover source/library mismatch and changed locator scheme without
  leaking locator/path content.
- Existing enqueue tests remain green.
- Focused server check and nextest filter pass.

## Out Of Scope

- No durable job claim loop, scheduler spawn, or runtime worker.
- No VFS reads and no `SourceFingerprintHashExecutor` invocation.
- No hash evidence persistence, duplicate relationship mutation, or source
  merge behavior.
- No Admin/Public API route or DTO.
- No schema migration.

## Technical Notes

- Main files likely touched: `crates/nako-server/src/app/source_hash.rs` and
  `crates/nako-server/src/app/tests/source_hash.rs`.
- Relevant specs:
  - `.trellis/spec/nako-server/backend/directory-structure.md`
  - `.trellis/spec/nako-server/backend/database-guidelines.md`
  - `.trellis/spec/nako-server/backend/error-handling.md`
  - `.trellis/spec/nako-server/backend/logging-guidelines.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
  - `.trellis/spec/nako-library/backend/quality-guidelines.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
