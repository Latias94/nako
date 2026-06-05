# Admin Source Fingerprint Hash Trigger Evidence

## Implementation Summary

- Added `POST /admin/v1/source-fingerprint-hashes`.
- Added Admin request DTOs for `library_id`, `source_id`, `mode`,
  `partial_prefix_bytes`, and optional priority.
- The HTTP handler validates full/partial mode shape, delegates to
  `SourceFingerprintHashAppService::enqueue_source_fingerprint_hash`, and
  returns redacted `AdminJobListItem` with `202 Accepted`.
- Regenerated Admin TypeScript contracts for `apps/admin-web` and `web`.
- Updated `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
  to record the implemented Admin trigger contract.

## Validation

- `cargo check -p nako-api -p nako-server --tests` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed.
- `cargo nextest run -p nako-api admin_source_fingerprint_hash --no-fail-fast`
  passed.
- `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`
  passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Git line-ending warnings.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-admin-source-fingerprint-hash-trigger-first-slice`
  passed.

## Redaction And Boundary Checks

- Admin response tests assert no Source Locator, local path fragment, source
  scheme field, durable input JSON, summary JSON, raw token, raw hash, or raw
  fingerprint is returned.
- Persisted source hash job input tests assert only the safe durable input
  fields are stored.
- Failure route tests cover missing source, cross-library source, invalid
  locator, invalid partial prefix, and non-admin rejection without enqueueing
  jobs.
- No duplicate relationship mutation, retry/requeue command, automatic scan
  scheduling, schema migration, or source-hash-specific runtime loop was added.
