# Evidence

## 2026-06-05

- Task opened after source fingerprint hash scheduling diagnostics shipped and
  architecture maps were reconciled to point future work at queue/operator
  integration.
- Scope is contract-only: durable job kind, redaction-safe job input, and
  runtime budget mapping. No enqueue service, executor, API, DB schema, VFS
  read, or source reconciliation behavior is in scope.
- Implemented `JobKind::SourceFingerprintHash` with persisted
  `source_fingerprint_hash` round-trip coverage in `nako-core`.
- Added `nako-library::source_hash::SourceFingerprintHashJobInput` and
  `SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS`. The durable input contract
  carries only Media Library ID, Media Source ID, source scheme, and
  `SourceFingerprintHashMode`; it derives safely from an in-process
  `SourceFingerprintHashRequest` without retaining `StorageUri`.
- Mapped `disk.scan.source_fingerprint_hash` to the existing `disk.scan`
  runtime budget class in `nako-server`.
- Updated Trellis specs and architecture maps to distinguish this shipped
  contract from future enqueue, executor, API, evidence persistence, and source
  reconciliation follow-ons.

## Verification

- `cargo fmt --all`
- `cargo check -p nako-core -p nako-library -p nako-server --tests`
- `cargo nextest run -p nako-core source_fingerprint_hash_kind_round_trips --no-fail-fast` — 1 passed
- `cargo nextest run -p nako-library source_hash --no-fail-fast` — 13 passed
- `cargo nextest run -p nako-server runtime_job_resource_class_mapping --no-fail-fast` — 2 passed
- `cargo fmt --all -- --check`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-05-source-fingerprint-hash-durable-job-contract-first-slice`
