# Phase 20.0: Server Surface Decomposition

## Summary

M20 reduces the `taru-server` test surface from two oversized integration test
files into bounded-context modules. This phase intentionally avoids behavior
changes. The goal is to make future playback, metadata, storage, addon, and
automation work easier to review without forcing every change through one giant
test file.

## App Tests

`crates/taru-server/src/app/tests.rs` was replaced by:

- `app/tests/mod.rs`
- `app/tests/startup.rs`
- `app/tests/storage.rs`
- `app/tests/nfo.rs`
- `app/tests/metadata.rs`
- `app/tests/playback.rs`
- `app/tests/staging.rs`

Shared app test fixtures stay in `app/tests/mod.rs`.

## HTTP Tests

`crates/taru-server/src/http/tests.rs` was replaced by:

- `http/tests/mod.rs`
- `http/tests/system.rs`
- `http/tests/webhooks.rs`
- `http/tests/automation.rs`
- `http/tests/addons.rs`
- `http/tests/library.rs`
- `http/tests/metadata.rs`
- `http/tests/catalog.rs`
- `http/tests/playback.rs`

Shared HTTP route fixtures stay in `http/tests/mod.rs`.

## Boundary Notes

- No route behavior changed.
- No new user-facing API was added.
- HTTP tests remain focused on explicit `taru-api` request and response DTOs.
- App tests remain focused on application service behavior and repository
  effects rather than SQLite internals.
- The largest remaining server service files are now clearer follow-up targets:
  `app/playback.rs`, `app/metadata.rs`, and their shared fixtures.

## Validation

Close-out validation:

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace
git diff --check
```
