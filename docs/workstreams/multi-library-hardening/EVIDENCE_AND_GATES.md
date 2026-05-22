# Multi-Library Hardening Evidence And Gates

Status: Completed
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
cargo nextest run -p nako-server app_startup_reports_configured_library_reconciliation --no-fail-fast
```

This captures the explicit startup reconciliation report for configured
libraries.

## Gate Set

### Characterization Gate

```powershell
cargo check -p nako-server --tests
cargo nextest run -p nako-server startup --no-fail-fast
```

Proves startup/config behavior is visible before code movement.

### Reconciliation Gate

```powershell
cargo check -p nako-server --tests
cargo check -p nako-db --tests
cargo nextest run -p nako-server <reconciliation-filter> --no-fail-fast
```

Proves the server and SQLite adapter agree on Library desired-state
reconciliation.

### Closeout Gate

```powershell
cargo fmt --all -- --check
cargo nextest run -p nako-server --no-fail-fast
git diff --check
```

Broaden to workspace gates if the implementation changes shared repository
traits or public API shapes.

### Review Gate

Run `review-workstream` before accepting MLH-030 and before closeout. Record
blocking findings, missing gates, and residual risks here.

## Evidence Anchors

- `docs/workstreams/multi-library-hardening/PHASE8_0_CORRECTNESS_BASELINE.md`
- `crates/nako-server/src/config.rs`
- `crates/nako-server/src/app/startup.rs`
- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-server/src/app/library.rs`
- `crates/nako-server/src/app/metadata.rs`
- `crates/nako-db/src/library.rs`
- `crates/nako-core/src/media/library.rs`

## Fresh Evidence

2026-05-18, MLH-010:

- Historical M8 multi-library decisions promoted into standard workstream docs.
- First executable task set to startup/config behavior characterization.
- Public Client Source Locator redaction and Library Access are explicit
  non-goals for this lane.

2026-05-18, MLH-020:

- Added startup characterization for persisting multiple configured libraries
  with library-scoped roots and presets.
- Added startup characterization showing configured desired state currently
  overwrites an existing persisted Library row with the same ID.
- Added startup characterization showing persisted Library rows missing from
  configuration are currently retained.
- Added config characterization showing `default_library_from_config` returns
  the first configured library when several libraries are configured.
- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server startup --no-fail-fast` passed: 13 passed,
  103 skipped.
- `cargo nextest run -p nako-server default_library_from_multi_library_config_returns_first_configured_library --no-fail-fast`
  passed: 1 passed, 115 skipped.

Fresh verification is required before marking implementation tasks or the lane
complete.

2026-05-18, MLH-030:

- Added `ConfiguredLibraryReconciliationService` in
  `crates/nako-server/src/app/library_reconciliation.rs` as the named startup
  reconciliation boundary.
- Startup now records a reconciliation report with counts for configured,
  added, updated, unchanged, and retained unconfigured libraries.
- Added characterization coverage for the reconciliation report in
  `crates/nako-server/src/app/tests/startup.rs`.
- `cargo check -p nako-server --tests` passed.
- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-server startup --no-fail-fast` passed: 14 passed,
  103 skipped.
- `cargo nextest run -p nako-server app_startup_reports_configured_library_reconciliation --no-fail-fast`
  passed: 1 passed, 116 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

Fresh verification is required before closing the lane.

2026-05-18, MLH-040:

- Scan jobs now load the reconciled persisted Library row by ID before building
  scanner options, so scan depth and roots come from the database authority.
- Metadata maintenance and refresh now resolve Library profiles through
  `LibraryRepository` using the media source's Library ID.
- NFO import now resolves local metadata policy from the persisted Library
  instead of carrying the full server config into `NfoAppService`.
- Storage diagnostics now lists reconciled persisted libraries and reports
  retained-but-unconfigured libraries as unavailable without exposing local
  paths or backend secret details.
- Added regression tests:
  `scan_library_uses_reconciled_library_row_after_startup`,
  `metadata_refresh_uses_reconciled_library_profile`,
  `nfo_import_uses_reconciled_library_policy`, and
  `storage_diagnostics_lists_reconciled_libraries_missing_from_config`.
- `cargo nextest run -p nako-server storage --no-fail-fast` passed: 10 passed,
  111 skipped.
- `cargo check -p nako-server --tests` passed.
- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-server startup --no-fail-fast` passed: 15 passed,
  106 skipped.
- `cargo nextest run -p nako-server metadata --no-fail-fast` passed: 15 passed,
  106 skipped.
- `cargo nextest run -p nako-server nfo --no-fail-fast` passed: 6 passed,
  115 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.
- `cargo nextest run -p nako-server --no-fail-fast` passed: 121 passed, 0
  skipped.

Fresh review is required before closing the lane.

2026-05-18, MLH-050:

- Closeout review found no blocking code-quality issues.
- Added startup validation for duplicate configured local roots and unsupported
  WebDAV root schemes, while preserving distinct backend endpoint roots.
- Added startup coverage for rejected duplicate local roots, rejected invalid
  WebDAV root schemes, and allowed same WebDAV root on different endpoints.
- `cargo check -p nako-server --tests` passed.
- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-server startup --no-fail-fast` passed: 18 passed,
  106 skipped.
- `cargo nextest run -p nako-server --no-fail-fast` passed: 124 passed, 0
  skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.
