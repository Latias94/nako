# Admin Playback Session Read Model Evidence And Gates

Status: Completed
Last updated: 2026-05-18

## Expected Gates

```bash
cargo fmt --all -- --check
cargo check -p nako-api --tests
cargo nextest run -p nako-api --no-fail-fast
cargo check -p nako-db --tests
cargo nextest run -p nako-db transcode --no-fail-fast
cargo check -p nako-server --tests
cargo nextest run -p nako-server http::tests::system --no-fail-fast
cargo nextest run -p nako-api public_openapi --no-fail-fast
cargo nextest run -p nako-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/nako-client-protocol
```

## Evidence Anchors

- `crates/nako-core/src/repository/transcode.rs`
- `crates/nako-db/src/playback.rs`
- `crates/nako-db/src/tests.rs`
- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/query.rs`
- `crates/nako-server/src/http/tests/system.rs`
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
cargo check -p nako-api --tests
cargo nextest run -p nako-api --no-fail-fast
cargo check -p nako-db --tests
cargo nextest run -p nako-db transcode --no-fail-fast
cargo check -p nako-server --tests
cargo nextest run -p nako-server http::tests::system --no-fail-fast
cargo nextest run -p nako-api public_openapi --no-fail-fast
cargo nextest run -p nako-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/nako-client-protocol
```

Results:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p nako-api --tests`: passed.
- `cargo nextest run -p nako-api --no-fail-fast`: 16 tests passed.
- `cargo check -p nako-db --tests`: passed.
- `cargo nextest run -p nako-db transcode --no-fail-fast`: 3 tests passed.
- `cargo check -p nako-server --tests`: passed.
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`: 7
  tests passed.
- `cargo nextest run -p nako-server http::tests::playback --no-fail-fast`: 16
  tests passed.
- `cargo nextest run -p nako-api public_openapi --no-fail-fast`: 3 tests
  passed.
- `cargo nextest run -p nako-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast`:
  1 test passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.
- `git diff --name-only -- crates/nako-client-protocol`: no output.
