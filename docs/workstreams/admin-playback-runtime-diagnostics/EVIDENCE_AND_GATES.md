# Admin Playback Runtime Diagnostics Evidence And Gates

Status: Completed
Last updated: 2026-05-18

## Expected Gates

```bash
cargo fmt --all -- --check
cargo check -p nako-api --tests
cargo nextest run -p nako-api --no-fail-fast
cargo check -p nako-server --tests
cargo nextest run -p nako-server http::tests::system --no-fail-fast
cargo nextest run -p nako-api public_openapi --no-fail-fast
cargo nextest run -p nako-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/nako-client-protocol
```

## Evidence Anchors

- `crates/nako-api/src/admin.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/tests/system.rs`
- `docs/workstreams/admin-web-console/V0_CONTEXT.md`
- `docs/GOALS.md`

## Evidence Log

- 2026-05-18: APRD-010 opened the workstream and recorded the Admin API,
  Public Client API, and redaction boundaries for M56.
- 2026-05-18: APRD-020 added admin-owned playback runtime diagnostics DTOs and
  a playback app diagnostics snapshot covering hardware policy, selected
  acceleration, capability evidence, transcode budgets, remux runtime limits,
  remote playback budgets, and staging cleanup configuration.
- 2026-05-18: APRD-030 added `GET /admin/v1/playback/runtime` and route tests
  covering safe diagnostics, local-path redaction, auth protection, and public
  OpenAPI/SDK boundary preservation.
- 2026-05-18: APRD-040 updated closeout docs, HTTP API docs, and
  admin-web-console data-source notes.

## Completed Gates

Results:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p nako-api --tests`: passed.
- `cargo nextest run -p nako-api --no-fail-fast`: 17 tests passed.
- `cargo check -p nako-server --tests`: passed.
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`: 8
  tests passed.
- `cargo nextest run -p nako-api public_openapi --no-fail-fast`: 3 tests
  passed.
- `cargo nextest run -p nako-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast`:
  1 test passed.
