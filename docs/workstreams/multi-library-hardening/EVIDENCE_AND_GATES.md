# Multi-Library Hardening Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
cargo nextest run -p taru-server app_startup_reports_configured_library_reconciliation --no-fail-fast
```

This captures the explicit startup reconciliation report for configured
libraries.

## Gate Set

### Characterization Gate

```powershell
cargo check -p taru-server --tests
cargo nextest run -p taru-server startup --no-fail-fast
```

Proves startup/config behavior is visible before code movement.

### Reconciliation Gate

```powershell
cargo check -p taru-server --tests
cargo check -p taru-db --tests
cargo nextest run -p taru-server <reconciliation-filter> --no-fail-fast
```

Proves the server and SQLite adapter agree on Library desired-state
reconciliation.

### Closeout Gate

```powershell
cargo fmt --all -- --check
cargo nextest run -p taru-server --no-fail-fast
git diff --check
```

Broaden to workspace gates if the implementation changes shared repository
traits or public API shapes.

### Review Gate

Run `review-workstream` before accepting MLH-030 and before closeout. Record
blocking findings, missing gates, and residual risks here.

## Evidence Anchors

- `docs/workstreams/multi-library-hardening/PHASE8_0_CORRECTNESS_BASELINE.md`
- `crates/taru-server/src/config.rs`
- `crates/taru-server/src/app/startup.rs`
- `crates/taru-server/src/app/jobs.rs`
- `crates/taru-server/src/app/library.rs`
- `crates/taru-server/src/app/metadata.rs`
- `crates/taru-db/src/library.rs`
- `crates/taru-core/src/media/library.rs`

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
- `cargo check -p taru-server --tests` passed.
- `cargo nextest run -p taru-server startup --no-fail-fast` passed: 13 passed,
  103 skipped.
- `cargo nextest run -p taru-server default_library_from_multi_library_config_returns_first_configured_library --no-fail-fast`
  passed: 1 passed, 115 skipped.

Fresh verification is required before marking implementation tasks or the lane
complete.

2026-05-18, MLH-030:

- Added `ConfiguredLibraryReconciliationService` in
  `crates/taru-server/src/app/library_reconciliation.rs` as the named startup
  reconciliation boundary.
- Startup now records a reconciliation report with counts for configured,
  added, updated, unchanged, and retained unconfigured libraries.
- Added characterization coverage for the reconciliation report in
  `crates/taru-server/src/app/tests/startup.rs`.
- `cargo check -p taru-server --tests` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo nextest run -p taru-server startup --no-fail-fast` passed: 14 passed,
  103 skipped.
- `cargo nextest run -p taru-server app_startup_reports_configured_library_reconciliation --no-fail-fast`
  passed: 1 passed, 116 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

Fresh verification is required before closing the lane.
