# Server Runtime Deepening Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Required Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-server --tests
cargo nextest run -p taru-server app::runtime --no-fail-fast
cargo nextest run -p taru-server app::tests::startup --no-fail-fast
cargo nextest run -p taru-server app::tests::metadata --no-fail-fast
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence To Record

- Startup workflow report fields and tests.
- Runtime supervisor durable job helper and diagnostics.
- Migrated library scan / metadata background job call sites.
- Confirmation that public API, SDK, CLI, database schema, playback decisions,
  and NFO round-trip behavior were not changed.

## Closeout Evidence

- `crates/taru-server/src/app/startup.rs` owns `ServerStartupWorkflow` and
  `ServerStartupReport`.
- `TaruApp::new_with_store` composes app services, then delegates startup side
  effects to `ServerStartupWorkflow`.
- `RuntimeSupervisor::spawn_job` records supervised job successes and failures.
- Library scan, metadata refresh, and metadata maintenance background jobs use
  `spawn_job`.
- Close-out validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -p taru-server --tests`: passed.
  - `cargo nextest run -p taru-server app::runtime --no-fail-fast`: 5 tests
    passed.
  - `cargo nextest run -p taru-server app::tests::startup --no-fail-fast`: 6
    tests passed.
  - `cargo nextest run -p taru-server app::tests::metadata --no-fail-fast`: 10
    tests passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: 284 tests passed.
  - `git diff --check`: passed.
- Public API, SDK, CLI, database schema, playback decisions, and NFO round-trip
  behavior were intentionally not changed.
