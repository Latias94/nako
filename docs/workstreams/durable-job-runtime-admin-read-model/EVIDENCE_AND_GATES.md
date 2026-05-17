# Durable Job Runtime And Admin Read Model Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Expected Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo nextest run -p taru-api --no-fail-fast
cargo check -p taru-db --tests
cargo nextest run -p taru-db jobs --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server app::tests::runtime --no-fail-fast
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo nextest run -p taru-api public_openapi --no-fail-fast
cargo nextest run -p taru-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

Broaden to workspace gates if repository traits, route behavior, or shared
runtime semantics change more widely than expected.

## Evidence Anchors

- `crates/taru-server/src/app/runtime.rs`
- `crates/taru-server/src/app/jobs.rs`
- `crates/taru-server/src/app/metadata.rs`
- `crates/taru-server/src/app/nfo.rs`
- `crates/taru-server/src/http/admin.rs`
- `crates/taru-api/src/admin.rs`
- `crates/taru-core/src/repository/jobs.rs`
- `crates/taru-db/src/jobs.rs`
- `docs/workstreams/admin-web-console/V0_CONTEXT.md`
- `docs/GOALS.md`

## Evidence Log

- 2026-05-17: JRM-010 opened the workstream and recorded server-side
  architecture findings for M54.
- 2026-05-17: JRM-020 introduced `taru-server::app::job_runtime` and migrated
  library scan, metadata refresh/maintenance, and NFO import/export execution
  paths onto it. A discovered lifecycle gap was fixed: summary serialization
  failures now persist the durable job as failed.
- 2026-05-17: JRM-030 added `JobListFilter`, SQLite job list/filter support,
  redacted `AdminJobListItem` DTOs, and `GET /admin/v1/jobs`. The list DTO
  intentionally omits raw input, summary, and error payloads.
- 2026-05-17: JRM-040 updated M54 closeout docs and admin-web-console data
  source notes.

## Completed Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo nextest run -p taru-api --no-fail-fast
cargo check -p taru-db --tests
cargo nextest run -p taru-db jobs --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server app::job_runtime --no-fail-fast
cargo nextest run -p taru-server app::tests::nfo --no-fail-fast
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo nextest run -p taru-api public_openapi --no-fail-fast
cargo nextest run -p taru-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

Results:

- `taru-api`: 15 tests passed.
- `taru-db jobs`: 2 tests passed.
- `taru-server app::job_runtime`: 3 tests passed.
- `taru-server app::tests::nfo`: 3 tests passed.
- `taru-server http::tests::system`: 6 tests passed.
- Public OpenAPI leakage: 3 tests passed.
- TypeScript SDK leakage: 1 test passed.
- `crates/taru-client-protocol`: no diff.
