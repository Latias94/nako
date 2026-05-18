# Admin Operations Read Models Evidence And Gates

Status: Completed
Last updated: 2026-05-18

## Expected Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo nextest run -p taru-api --no-fail-fast
cargo check -p taru-db --tests
cargo nextest run -p taru-db outbox --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo nextest run -p taru-api public_openapi --no-fail-fast
cargo nextest run -p taru-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

## Evidence Anchors

- `crates/taru-core/src/repository/jobs.rs`
- `crates/taru-db/src/event_outbox.rs`
- `crates/taru-api/src/admin.rs`
- `crates/taru-api/src/openapi.rs`
- `crates/taru-server/src/app/webhooks.rs`
- `crates/taru-server/src/app/storage.rs`
- `crates/taru-server/src/http/admin.rs`
- `crates/taru-server/src/http/query.rs`
- `crates/taru-server/src/http/tests/system.rs`
- `docs/api/HTTP_API.md`
- `docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md`
- `docs/workstreams/admin-web-console/V0_CONTEXT.md`
- `docs/GOALS.md`

## Evidence Log

- 2026-05-18: AORM-010 opened the workstream and recorded route, redaction,
  and Public Client API boundary rules.
- 2026-05-18: AORM-020 added `OutboxEventListFilter`, SQLite event outbox
  filtering, and DB tests for kind/status/library/source filters and
  pagination.
- 2026-05-18: AORM-030 added `GET /admin/v1/events` with admin-owned redacted
  event outbox list DTOs and route tests for filtering, pagination, redaction,
  auth protection, and public OpenAPI exclusion.
- 2026-05-18: AORM-040 added `GET /admin/v1/storage/staging` with redacted
  staging manifest rows, staging budget/startup cleanup summary, and VFS cache
  summary counters. Route tests cover path, URI, etag/fingerprint, validation
  error, and cache error redaction.
- 2026-05-18: AORM-050 added `GET /admin/v1/system/config` with sanitized
  auth, library, runtime, metadata, transcode, staging, and playback config
  diagnostics. Route tests cover database URL, local path, WebDAV credential,
  metadata proxy, provider URL, and literal header secret redaction.
- 2026-05-18: AORM-060 updated HTTP API, goal map, admin-web-console, and
  workstream docs.

## Completed Gates

- `cargo check -p taru-db --tests`: passed.
- `cargo nextest run -p taru-db outbox --no-fail-fast`: 2 tests passed.
- `cargo check -p taru-api --tests`: passed.
- `cargo nextest run -p taru-api --no-fail-fast`: 19 tests passed.
- `cargo check -p taru-server --tests`: passed.
- `cargo check -p taru-vfs --tests`: passed.
- `cargo nextest run -p taru-server http::tests::system --no-fail-fast`: 11
  tests passed.
- `cargo nextest run -p taru-api public_openapi --no-fail-fast`: 3 tests
  passed.
- `cargo nextest run -p taru-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast`:
  1 test passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF normalization warnings only.
- `git diff --name-only -- crates/taru-client-protocol`: no output.

## Notes

- Public OpenAPI/SDK gates are boundary tests. They are not enough by
  themselves; route-level tests must prove each admin read model's redaction
  behavior.
