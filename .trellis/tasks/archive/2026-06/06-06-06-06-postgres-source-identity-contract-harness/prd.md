# PostgreSQL source identity contract harness suite

## Goal

Add a focused PostgreSQL contract harness suite for source identity persistence
risks so operators and release gates can run the relevant ignored `nako-db`
PostgreSQL contracts without falling back to the broad `all-contracts` suite.

## What I already know

* `scripts/postgres-contract-harness.ps1` and
  `scripts/postgres-contract-harness.sh` currently support
  `managed-artwork`, `storage-runtime`, and `all-contracts`.
* `crates/nako-db/src/contract_tests.rs` already contains PostgreSQL ignored
  contract names for library-scoped source identity, source duplicate
  relationship identity, source duplicate fingerprint matching, and VFS
  attribution/staging behavior.
* This task should not introduce schema, repository, API, or runtime behavior
  changes. It is a harness targeting improvement.

## Requirements

* Add a `source-identity` suite to both PowerShell and Bash PostgreSQL contract
  harnesses.
* The new suite must run the existing ignored PostgreSQL tests that cover:
  source identity preservation, source duplicate relationship persistence,
  source fingerprint match lookup, and VFS staging attribution/source-related
  persistence.
* Keep the suite filters explicit and maintainable; do not use the broad
  `postgres_` filter.
* Preserve existing harness behavior for caller-provided PostgreSQL URLs,
  local temporary PostgreSQL clusters, skip behavior, cleanup, and keep-data
  behavior.
* Update command help or durable docs when they enumerate supported harness
  suites.

## Acceptance Criteria

* [x] `scripts/postgres-contract-harness.ps1 -Suite source-identity` is an
  accepted suite value and maps to explicit `nako-db` nextest filters.
* [x] `scripts/postgres-contract-harness.sh --suite source-identity` is an
  accepted suite value and maps to the same logical filter family.
* [x] Existing suite names continue to work unchanged.
* [x] Focused tests or script validation prove the suite selection contract.
* [x] Formatting/check gates for the changed files pass.

## Definition of Done

* Tests added or updated where appropriate.
* Focused validation is run and recorded.
* No unrelated working tree changes are reverted or deleted.
* Trellis task context reflects the specs used for implementation and checking.

## Technical Approach

Extend the harness suite switch/case blocks with `source-identity` and use the
existing contract test naming convention as the source of truth. The suite is
an orchestration-only change, so implementation should avoid changing database
schema, repository traits, migrations, or contract test bodies unless a naming
drift is discovered during validation.

## Out of Scope

* Adding new source identity repository behavior.
* Adding or changing database migrations.
* Changing Admin/Public API contracts.
* Running the full PostgreSQL contract suite unless needed to diagnose a
  harness failure.

## Technical Notes

* Relevant spec files:
  `.trellis/spec/nako-db/backend/index.md`,
  `.trellis/spec/nako-db/backend/database-guidelines.md`,
  `.trellis/spec/nako-db/backend/quality-guidelines.md`, and
  `.trellis/spec/guides/code-reuse-thinking-guide.md`.
* Existing source identity filters found in
  `crates/nako-db/src/contract_tests.rs` include
  `postgres_library_media_contract_preserves_library_scoped_source_identity`,
  `postgres_source_duplicate_contract_upsert_is_idempotent_by_canonical_pair`,
  `postgres_source_duplicate_contract_lists_fingerprint_matches_and_pair_lookup`,
  `postgres_vfs_staging_contract_round_trips_attribution_variants`, and
  `postgres_vfs_staging_contract_preserves_reservation_budget_and_leases`.
