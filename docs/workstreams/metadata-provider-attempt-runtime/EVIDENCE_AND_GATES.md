# Metadata Provider Attempt Runtime Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Baseline Evidence

- M40 introduced `MetadataRefreshPort` and `MetadataAttemptPort`.
- `crates/nako-metadata/src/strategy.rs` still owns provider attempt execution,
  provider search/fetch, attempt classification, refresh commit, and catalog
  hydration orchestration in one Module.
- Current M44 scope intentionally avoids provider breadth, public API changes,
  repository trait churn, NFO, playback, and database schema changes.

## Focused Gates

```powershell
cargo fmt --all -- --check
cargo check -p nako-metadata --tests
cargo nextest run -p nako-metadata --no-fail-fast
cargo nextest run -p nako-metadata strategy::port_tests::refresh_service_uses_refresh_and_hydration_ports_without_sqlite --no-fail-fast
```

## Closeout Gates

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Log

- 2026-05-17: Workstream opened for M44.
- 2026-05-17: Added internal `provider_attempt` runtime Module in
  `nako-metadata`.
- 2026-05-17: `MetadataStrategyExecutor::refresh_item` now delegates available,
  disabled, unavailable, and unregistered provider attempt handling while
  keeping refresh commit and catalog hydration explicit.
- 2026-05-17: Focused validation passed:
  - `cargo check -p nako-metadata --tests`.
  - `cargo nextest run -p nako-metadata --no-fail-fast`: 27 tests passed.

## Closeout Evidence

- Provider lookup/fetch, raw response construction, attempt recording, skipped
  attempt construction, and provider error classification live in
  `crates/nako-metadata/src/provider_attempt.rs`.
- `MetadataStrategyExecutor::refresh_item` preserves its public workflow shape
  and still commits refresh results before catalog hydration.
- Public HTTP API, OpenAPI, SDK/protocol crates, repository traits, database
  schema, NFO, and playback behavior did not change.
- Validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -p nako-metadata --tests`: passed.
  - `cargo nextest run -p nako-metadata --no-fail-fast`: 27 tests passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: passed.
  - `git diff --check`: passed.
