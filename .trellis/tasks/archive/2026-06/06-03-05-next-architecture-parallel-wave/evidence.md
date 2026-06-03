# Next Architecture Parallel Wave Evidence

Date: 2026-06-03

## Integration Summary

- Integrated child lanes `05a`, `05b`, `05c`, and `05d` into `main`.
- Resolved the server staging-pressure spec conflict by preserving scoped
  library/backend staging admission from `05a` and queued candidate preview /
  exact-claim fairness from `05b`.
- Preserved `05c` PostgreSQL storage-runtime parity harness changes that were
  stacked on the `05b` branch.
- Preserved `05d` library intake stable-candidate evidence module.
- Removed merge-leftover unused global staging-pressure helpers after queued
  scheduling moved to per-candidate library admission.
- Updated the queued staging-pressure test fixture to create pressure on the
  matching WebDAV slice instead of an unrelated local staging slice.

## Fresh Verification On Main

- `cargo check -p nako-server -p nako-db -p nako-library -p nako-api --tests`
  passed without warnings after the integration cleanup.
- `cargo nextest run -p nako-server scan_library_rejects_open_storage_circuit_before_pipeline scan_library_rejects_critical_staging_pressure_before_pipeline scan_library_allows_local_library_during_remote_staging_pressure webdav_scan_admission_ignores_local_staging_pressure webdav_scan_admission_blocks_matching_staging_pressure_without_raw_details admin_v1_storage_staging_attributes_policy_slices_without_raw_backend_data background_scan_scheduler_skips_blocked_library_and_schedules_runnable_scan job_scheduler_keeps_remote_scan_jobs_queued_while_staging_pressure_is_critical scan_library_persists_job_success --no-fail-fast`
  passed: 9 tests.
- `cargo nextest run -p nako-api admin_storage_staging_policy_slice_redacts_source_identity admin_contract --no-fail-fast`
  passed: 7 tests.
- `cargo nextest run -p nako-db storage_backend_health_contract vfs_staging_contract job_lease_contract sqlite_job_retry_contract_priority_policy_orders_fairly_and_recovers --no-fail-fast`
  passed: 8 tests.
- `cargo nextest run -p nako-library intake --no-fail-fast` passed: 3 tests.
- `pwsh -File scripts/postgres-contract-harness.ps1 -Suite storage-runtime -RequireTooling`
  passed: 3 PostgreSQL ignored contract tests.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with LF/CRLF normalization warnings only.
- `bash -n scripts/postgres-contract-harness.sh` passed.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .trellis docs crates scripts apps web`
  found no merge-conflict markers.

## Deferred Follow-Ons

- Broader PostgreSQL runtime suites beyond the selected `storage-runtime`
  harness remain future parity work.
- Real filesystem watcher runtime integration remains separate from the `05d`
  intake evidence MVP.
- Persisted attribution for ambiguous same-root multi-endpoint library staging
  records remains a future storage policy authority improvement.
