# Admin Playback Session Read Model Evidence And Gates

Status: Completed
Last updated: 2026-05-18

## Expected Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo nextest run -p taru-api --no-fail-fast
cargo check -p taru-db --tests
cargo nextest run -p taru-db transcode --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo nextest run -p taru-api public_openapi --no-fail-fast
cargo nextest run -p taru-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

## Evidence Anchors

- `crates/taru-core/src/repository/transcode.rs`
- `crates/taru-db/src/playback.rs`
- `crates/taru-db/src/tests.rs`
- `crates/taru-api/src/admin.rs`
- `crates/taru-server/src/app/playback/mod.rs`
- `crates/taru-server/src/http/admin.rs`
- `crates/taru-server/src/http/query.rs`
- `crates/taru-server/src/http/tests/system.rs`
- `docs/workstreams/admin-web-console/V0_CONTEXT.md`
- `docs/GOALS.md`

## Evidence Log

- 2026-05-18: APS-010 opened the workstream and recorded the Admin API,
  Public Client API, and redaction boundaries for M55.
- 2026-05-18: APS-020 added `TranscodeSessionListFilter` and SQLite
  list/filter support for source, kind, state, and pagination.
- 2026-05-18: APS-030 added redacted `AdminPlaybackSessionListItem`,
  `AdminPlaybackSessionListResponse`, and `GET
  /admin/v1/playback/sessions`. The list DTO omits `output_path` and raw
  failure messages.
- 2026-05-18: APS-040 updated M55 closeout docs and admin-web-console data
  source notes.

## Completed Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo nextest run -p taru-api --no-fail-fast
cargo check -p taru-db --tests
cargo nextest run -p taru-db transcode --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo nextest run -p taru-api public_openapi --no-fail-fast
cargo nextest run -p taru-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

Results:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p taru-api --tests`: passed.
- `cargo nextest run -p taru-api --no-fail-fast`: 16 tests passed.
- `cargo check -p taru-db --tests`: passed.
- `cargo nextest run -p taru-db transcode --no-fail-fast`: 3 tests passed.
- `cargo check -p taru-server --tests`: passed.
- `cargo nextest run -p taru-server http::tests::system --no-fail-fast`: 7
  tests passed.
- `cargo nextest run -p taru-server http::tests::playback --no-fail-fast`: 16
  tests passed.
- `cargo nextest run -p taru-api public_openapi --no-fail-fast`: 3 tests
  passed.
- `cargo nextest run -p taru-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast`:
  1 test passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.
- `git diff --name-only -- crates/taru-client-protocol`: no output.
