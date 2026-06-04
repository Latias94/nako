# PostgreSQL VFS cache failure authority decode

## Problem

Docker-backed PostgreSQL storage-runtime contract verification exposed a row
mapping bug in `postgres_vfs_staging_contract_round_trips_listing_failures_and_summary`.
The PostgreSQL adapter selects `vfs_cache_failures.library_id` as a UUID column
while the mapper decodes it through the existing optional string ID parser.

## Goal

Make PostgreSQL VFS cache failure authority round-trip with the same repository
contract semantics as SQLite.

## Scope

- Fix PostgreSQL VFS cache failure SELECT row shape or mapper so optional
  authority fields decode safely.
- Preserve the existing core repository trait and public/admin DTO surface.
- Re-run the Docker-backed PostgreSQL storage-runtime contract gate.

## Non-Goals

- No schema migration changes.
- No API or Admin-Web changes.
- No cache repair behavior expansion beyond the decode fix.

## Acceptance

- `postgres_vfs_staging_contract_round_trips_listing_failures_and_summary`
  passes against a real PostgreSQL database.
- The full storage-runtime PostgreSQL harness passes.
- Focused SQLite VFS cache/staging behavior remains green.

## Verification

- Failed first against Docker-backed PostgreSQL:
  `pwsh -File scripts/postgres-contract-harness.ps1 -Suite storage-runtime -DatabaseUrl postgres://...`
  exposed `library_id` UUID decoding through an optional string mapper.
- Passed after the fix:
  `cargo nextest run -p nako-db postgres_vfs_staging_contract_round_trips_listing_failures_and_summary --run-ignored ignored-only --no-fail-fast`.
- Passed after the fix:
  `pwsh -File scripts/postgres-contract-harness.ps1 -Suite storage-runtime -DatabaseUrl postgres://...`.
- Passed after the fix: `cargo check -p nako-db --tests`.
- Passed after the fix: `cargo nextest run -p nako-db vfs_cache --no-fail-fast`.
- Passed after the fix: `cargo fmt --all -- --check`.
- Passed after the fix: `git diff --check`.
