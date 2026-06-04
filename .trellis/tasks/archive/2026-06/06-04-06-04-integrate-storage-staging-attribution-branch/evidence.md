# Evidence

## Integration Decision

The stale `task/06-03-06b-storage-staging-attribution-persistence` branch should
not be merged directly. Its feature commit, `6da64fdc feat(storage): persist
staging attribution`, has already been absorbed on `main` by:

- `8d9daa18 feat(storage): persist staging attribution`
- `644ecc52 chore(trellis): archive 06b staging attribution task`

`git cherry -v main task/06-03-06b-storage-staging-attribution-persistence`
still reports `6da64fdc` as unique because the patch shape differs, but the
current `main` implementation already carries the core/API/DB/server/Admin
contract behavior. The current implementation is stricter than the stale patch
in places, including the enum-shaped `StagingAttribution` contract.

## Additional Coverage Added

Added `staging_manifest_contract_round_trips_attribution_variants` in
`crates/nako-db/src/contract_tests.rs`.

The contract now explicitly proves:

- `attributed(library_id)` round-trips through the staging manifest repository.
- `ambiguous()` round-trips without a library id.
- `unknown()` round-trips without a library id.
- Updating a record from `attributed` to `ambiguous` clears the library id.
- The same contract is registered for SQLite and PostgreSQL adapters.

PostgreSQL execution remains optional and depends on `NAKO_TEST_POSTGRES_URL`.

## Verification

Passed:

- `cargo check -p nako-db -p nako-server -p nako-api --tests`
- `cargo nextest run -p nako-core staging_attribution_rejects_invalid_persisted_combinations --no-fail-fast`
- `cargo nextest run -p nako-db round_trips_attribution_variants --no-fail-fast`
- `cargo nextest run -p nako-db vfs_staging --no-fail-fast`
- `cargo nextest run -p nako-db sqlite_store_rejects_invalid_staging_attribution_shape --no-fail-fast`
- `cargo nextest run -p nako-server webdav_scan_admission ambiguous_same_root --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_storage_staging_lists_filters_and_redacts_paths admin_v1_storage_staging_attributes_policy_slices_without_raw_backend_data --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

Notes:

- A previous exploratory filter, `cargo nextest run -p nako-server admin_storage_staging --no-fail-fast`, matched no tests and returned `no tests to run`; the exact server HTTP test names above were used instead.
- `git diff --check` emitted only Git's line-ending warning for the touched Rust file, not a whitespace error.
