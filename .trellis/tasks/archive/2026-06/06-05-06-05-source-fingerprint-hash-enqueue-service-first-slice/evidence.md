# Evidence

## 2026-06-05

- Task opened after the source fingerprint hash durable job contract shipped.
- Scope is enqueue-only: server app service persists safe
  `SourceFingerprintHash` jobs for known sources. Scheduler, executor, VFS
  reads, API routes, schema changes, evidence persistence, and reconciliation
  remain follow-ons.
- Implemented `nako-server::app::source_hash::SourceFingerprintHashAppService`
  with an internal enqueue method for existing Media Sources.
- The service verifies Media Source library ownership, derives only source
  scheme from the current locator, and persists `SourceFingerprintHashJobInput`
  as safe input JSON with `disk.scan.source_fingerprint_hash`.
- Added focused app tests for successful enqueue, missing source,
  cross-library rejection, invalid locator rejection, and redaction of
  path/locator/fingerprint-sensitive values.
- Updated server specs and storage/library/control-plane architecture maps to
  distinguish internal enqueue support from scheduler/executor/API/evidence
  persistence/reconciliation follow-ons.

## Verification

- `cargo fmt --all`
- `cargo nextest run -p nako-server source_fingerprint_hash_enqueue --no-fail-fast` — 4 passed
- `cargo check -p nako-server --tests`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-05-06-05-source-fingerprint-hash-enqueue-service-first-slice`
