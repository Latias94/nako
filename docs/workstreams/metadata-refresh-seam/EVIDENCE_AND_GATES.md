# Metadata Refresh Seam Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Required Gates

```bash
cargo fmt --all -- --check
cargo check -p nako-metadata --tests
cargo nextest run -p nako-metadata --no-fail-fast
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence To Record

- Workstream docs and top-level roadmap updates.
- Current refresh/confirmation dependency audit.
- Chosen workflow-port shape and why it is not a mechanical trait split.
- Fake-port test coverage for the new seam.
- Existing SQLite-backed metadata tests still passing.
- Confirmation that public API, SDK, CLI, playback, NFO Round Trip, provider
  breadth, and DB schema were not changed.

## Closeout Evidence

- `crates/nako-metadata/src/strategy.rs` defines `MetadataRefreshPort`,
  `MetadataAttemptPort`, `MetadataRefreshSnapshot`, and
  `MetadataRefreshCommit`.
- `MetadataRefreshService` and `MetadataStrategyExecutor` now require
  `CatalogHydrationPort + MetadataRefreshPort + MetadataAttemptPort` instead
  of directly naming the wider repository trait combination.
- Refresh calculation uses a loaded snapshot, while refresh persistence,
  provider subject/mapping writes, and library-item confirmation sit behind
  `commit_refresh`.
- Catalog hydration remains behind M39's `CatalogHydrationPort`.
- `strategy::port_tests::refresh_service_uses_refresh_and_hydration_ports_without_sqlite`
  proves refresh behavior through fake workflow ports.
- Focused validation so far:
  - `cargo check -p nako-metadata --tests`: passed.
  - `cargo nextest run -p nako-metadata --no-fail-fast`: 27 tests passed.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: 286 tests passed.
  - `git diff --check`: passed.
