# Quality Guidelines

Use these gates for `crates/nako-server` feature work.

## Test Patterns

- Prefer `#[tokio::test]` for async app, route, storage, and runtime tests.
- Use `#[test]` for pure functions such as parser, playlist, selection, or
  constant-time comparison behavior.
- Route tests use Axum routers plus `tower::ServiceExt`, not a live network
  server, unless an external service fixture is required.
- Tests create isolated data with `tempfile::tempdir()` and in-memory database
  helpers when possible.
- Auth and access tests should assert both status code and public response body
  shape, including `WWW-Authenticate: Bearer` for `401`.
- ADR 0053 requires new list surfaces to stay bounded and paginated rather than
  returning unbounded JSON.
- When playback admission policy is changed, cover immediate rejection, typed
  bounded wait paths such as `HlsStart`/`HlsSupersede`, and the affected app
  flow. Keep wait constants and configured-capacity checks in the resource
  helper layer.
- HLS admission tests must prove ordinary startup rejects unconfigured capacity
  before FFmpeg input staging, waits only within the bounded policy when
  capacity is busy, preserves `HlsSupersede` for replacements, and releases
  acquired permits when staging or runner work fails.
- Durable job resource-class mapping changes must cover
  `runtime_budget_class_for_job_resource_class` with focused server tests,
  especially when a feature-specific persisted resource class maps onto an
  existing runtime budget such as `disk.scan`.
- Internal source fingerprint hash enqueue changes must prove safe job input
  serialization, missing-source rejection, cross-library rejection, and
  locator/path/error-message redaction with focused app tests.

## Gate Selection

- Narrow server change:
  `cargo check -p nako-server --tests`
- Focused server behavior:
  `cargo nextest run -p nako-server <filter> --no-fail-fast`
- Cross-crate API/server change:
  `cargo check -p nako-api -p nako-server --tests`
  and focused `nako-api` + `nako-server` nextest filters.
- Full Rust closeout:
  `cargo fmt --all -- --check`, `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast`.

## Forbidden Patterns

- Do not add unauthenticated sensitive routes outside the explicit public route
  groups in `http.rs`.
- Do not return raw domain/database records from HTTP handlers.
- Do not log secrets, raw tokens, playback tickets, or local filesystem paths.
- Do not rely on `cargo test` as the default Rust gate when `cargo nextest` is
  available; this repo has `.config/nextest.toml` and CI installs nextest.

## Evidence

- `.config/nextest.toml`
- `.github/workflows/release-gate.yml`
- `crates/nako-server/src/http/tests/mod.rs`
- `crates/nako-server/src/app/tests/*.rs`
