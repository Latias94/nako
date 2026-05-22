# Typed Storage Errors Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Baseline Evidence

- `crates/nako-server/src/http/error.rs` classified storage errors by parsing
  message strings for timeout, auth, rate limit, staging budget, and staging
  validation cases.
- `crates/nako-vfs/src/webdav.rs` already centralizes WebDAV retry/status
  behavior, making it the first useful source-classification target.

## Focused Gates

```powershell
cargo check -p nako-core --tests
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs --no-fail-fast
cargo check -p nako-server --tests
cargo nextest run -p nako-server http::tests::system::api_errors_map_playback_storage_categories --no-fail-fast
```

## Closeout Gates

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Log

- 2026-05-17: Workstream opened for M45.
- 2026-05-17: Added `StorageErrorKind` and storage helper constructors in
  `nako-core`.
- 2026-05-17: Replaced HTTP storage message parsing with typed
  `StorageErrorKind` matching while preserving public error codes.
- 2026-05-17: Classified WebDAV, local VFS, staging, playback file IO,
  transcode output IO, and test storage fakes at storage error construction
  sites.
- 2026-05-17: Focused validation passed:
  - `cargo check -p nako-core --tests`.
  - `cargo check -p nako-vfs --tests`.
  - `cargo nextest run -p nako-vfs --no-fail-fast`: 17 tests passed before the
    added focused WebDAV status test; focused status test passed separately.
  - `cargo check -p nako-server --tests`.
  - `cargo nextest run -p nako-server http::tests::system::api_errors_map_playback_storage_categories --no-fail-fast`: 1 test passed.

## Closeout Evidence

- `NakoError::Storage` now includes `StorageErrorKind`.
- `crates/nako-server/src/http/error.rs` no longer parses storage error
  messages to classify storage failures.
- `crates/nako-vfs/src/webdav.rs` maps WebDAV request/status failures to typed
  timeout, unauthorized, rate-limited, network, and HTTP-status categories.
- Staging budget and staging validation failures use explicit storage kinds.
- Public storage error DTO behavior stayed stable.
- Validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -p nako-core --tests`: passed.
  - `cargo check -p nako-vfs --tests`: passed.
  - `cargo nextest run -p nako-vfs --no-fail-fast`: passed.
  - `cargo check -p nako-server --tests`: passed.
  - `cargo nextest run -p nako-server http::tests::system::api_errors_map_playback_storage_categories --no-fail-fast`: passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: 293 tests passed.
  - `git diff --check`: passed.
