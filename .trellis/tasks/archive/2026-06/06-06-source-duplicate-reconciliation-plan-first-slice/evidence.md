# Source Duplicate Reconciliation Plan Evidence

## Implementation Summary

- Added core redacted Source Fingerprint parsing helpers and read-only
  reconciliation plan domain records.
- Added repository reads for bounded same-library fingerprint matches and
  canonical Source Duplicate Relationship pair lookup.
- Implemented SQLite and PostgreSQL adapter parity for the new reads.
- Added an internal server app service that returns read-only redacted
  reconciliation plans.
- Updated Trellis code-specs for the DB and server duplicate reconciliation
  contracts.

## Validation

- `cargo check -p nako-core -p nako-db -p nako-server --tests` passed.
- `cargo nextest run -p nako-core source_fingerprint --no-fail-fast` passed:
  9 tests passed.
- `cargo nextest run -p nako-db source_duplicate --no-fail-fast` passed:
  5 tests passed.
- `cargo nextest run -p nako-server source_duplicate --no-fail-fast` passed:
  3 tests passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Git line-ending warnings.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-source-duplicate-reconciliation-plan-first-slice`
  passed.

## Redaction And Boundary Checks

- The plan returns source ids, evidence kind, confidence, stale state, existing
  relationship status/id, and recommended action only.
- The plan does not expose Source Locators, local paths, etags, backend URLs,
  credentials, raw hashes, raw fingerprints, evidence values, or durable job
  input JSON.
- The app-service tests assert the plan is read-only by comparing relationships
  before and after planning.
- No Admin/Public route, apply endpoint, source merge, item merge, scan
  scheduling, or source hash completion writer was added.

## Workflow Notes

- Trellis sub-agent spawning was not used in this environment because the
  available sub-agent tool requires explicit user authorization for delegation.
  Equivalent Trellis context loading, spec review, implementation review, and
  focused verification were performed in the main session.
