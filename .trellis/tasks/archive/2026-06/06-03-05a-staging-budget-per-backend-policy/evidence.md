# Staging Budget Per-Backend Policy Evidence

Date: 2026-06-03
Selected slice: per-backend and uniquely attributable per-library staging
budget policy for synchronous scan admission and Admin diagnostics.

## Selection

Chose the smallest durable MVP that moves staging pressure beyond one global
manifest total without forcing a schema migration:

- staging manifest records already persist `source_uri`, `source_scheme`,
  purpose, size, and redaction-safe record metadata;
- storage backend diagnostics already use typed backend keys
  (`library:{library_id}:{scheme}`);
- synchronous scan admission already has a typed entry seam through
  `StorageBackendRegistry::library_scan_admission_error`.

The shipped slice derives typed staging policy slices from existing manifest
records. It attributes pressure to:

- a backend aggregate slice for every configured source scheme; and
- a library slice when the configured library root can be uniquely matched from
  existing manifest source URIs.

This preserves current schema and current global diagnostics while removing the
worst false-positive behavior where local staging pressure could block remote
scan admission.

## Shipped Behavior

- Added typed staging budget policy slices in `nako-server` app storage
  diagnostics and mapped them to Admin DTOs.
- Synchronous remote scan admission now checks the matching library/backend
  policy slice instead of the global manifest total.
- If manifest records cannot be safely attributed to one configured library
  scope, admission falls back to the backend-level slice instead of inventing a
  false per-library attribution.
- Global staging pressure summary remains unchanged for current Admin
  compatibility and for queued scheduler protection.
- Admin staging diagnostics now include `policy_slices` while keeping existing
  summary fields and record redaction behavior intact.
- Refreshed generated Admin TypeScript contracts for both Admin Web surfaces and
  updated mock storage diagnostics data.

## Boundaries Preserved

- No schema migration or repository contract expansion.
- No PostgreSQL-specific staging policy query path; attribution is derived in
  the server app layer from paged manifest reads.
- No scan scheduler fairness changes. Global queued scheduler pressure behavior
  stays unchanged and remains lane `05b`.
- No watcher/debounce, Public Client API, or raw operator-secret expansion.
- No raw Source Locator, local path, fingerprint, credential, backend URL, or
  raw backend error is exposed by the new diagnostics or admission path.

## Verification

- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server webdav_scan_admission_ignores_local_staging_pressure webdav_scan_admission_blocks_matching_staging_pressure_without_raw_details scan_library_rejects_critical_staging_pressure_before_pipeline admin_v1_storage_staging_attributes_policy_slices_without_raw_backend_data --no-fail-fast`
  passed: 4 tests.
- `cargo nextest run -p nako-api admin_storage_staging_policy_slice_redacts_source_identity --no-fail-fast`
  passed: 1 test.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  passed: 6 tests.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with LF/CRLF normalization warnings only.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-03-05a-staging-budget-per-backend-policy`
  passed.

## Deferred Follow-Ons

- True per-library staging budgets for overlapping or same-root multi-endpoint
  libraries still need persisted attribution data or a schema-backed authority.
- PostgreSQL/runtime parity evidence for server-derived staging attribution
  remains a separate lane.
- Queued scan scheduler fairness that can let safe local work proceed under
  remote pressure remains `05b`.
- Any future operator actions that clean or repair staging pressure sources
  should build on these typed slices instead of adding raw manifest leak paths.

## Spec Update Judgment

Updated `.trellis/spec/nako-server/backend/directory-structure.md` because this
task changed the durable server-side library scan staging-pressure admission and
Admin diagnostics contract.
