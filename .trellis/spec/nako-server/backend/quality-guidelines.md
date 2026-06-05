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
- Internal source fingerprint hash queued execution planner changes must prove
  successful in-memory request recovery, wrong job kind/resource rejection,
  malformed or unsafe input rejection, binding mismatch rejection, locator
  scheme drift rejection, queued job non-mutation, and locator/path/input
  redaction with focused app tests.
- Internal source fingerprint hash durable executor command changes must prove
  a job is claimed through durable lease runtime, completed with a redaction-safe
  `SourceFingerprintHashJobSummary`, no longer claimable after success, and no
  summary JSON contains locator/path/fingerprint/hash material. Keep automatic
  scheduling and API routes out of command-only slices.
- Source fingerprint hash scheduler integration changes must prove
  scheduler-originated execution succeeds through a claimed-job helper,
  unrelated claimable jobs cannot hide disk-scan candidates, cross-kind
  starvation ordering is preserved, execution failures persist redaction-safe
  durable job errors, and existing background scan scheduler behavior remains
  green. Keep API routes, schema changes, evidence persistence, and duplicate
  reconciliation out of scheduler-only slices.

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

## Scenario: Remote Access Config Gate Fixtures

### 1. Scope / Trigger

- Trigger: changing remote access cookbook docs, `deploy/remote-access/*.toml`,
  `scripts/remote-access-config-gate.*`, or `config-check` network readiness
  output.
- Purpose: prove reverse-proxy and external tunnel-provider examples are
  accepted by `nako-server config-check --json --create-dirs` while raw network
  facts remain redacted.

### 2. Signatures

- PowerShell gate:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/remote-access-config-gate.ps1`.
- Bash gate:
  `bash scripts/remote-access-config-gate.sh`.
- Fixtures:
  `deploy/remote-access/reverse-proxy.nako.toml` and
  `deploy/remote-access/tunnel-provider.nako.toml`.
- Output reports:
  `target/release-gate/remote-access/<fixture>-config-check.json`.
- Expected network check IDs:
  `network.access`, `network.proxy`, `network.origins`, and
  `network.tunnel_providers`.

### 3. Contracts

- Fixtures must keep auth enabled and use loopback/private listener defaults.
- `reverse_proxy` fixtures must use HTTPS `external_base_url`, exact
  `allowed_origins`, and explicit reviewed `trusted_proxy_sources`.
- `tunnel_provider` fixtures must declare external provider metadata only; they
  must not start, supervise, or configure tunnel processes.
- Gate scripts must set fixture-only environment variables for auth/tunnel
  tokens and restore the caller environment afterward.
- JSON reports must not contain raw fixture URLs, tunnel token values, bearer
  token values, private origins, trusted proxy sources, forwarded header names,
  or local host details such as `127.0.0.1`.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Config-check exits non-zero | Gate fails and leaves no successful report for that fixture |
| Overall report status is not `pass` | Gate fails |
| Expected network check is missing or not `pass` | Gate fails |
| Report contains fixture URL, origin, proxy source, header name, token, or local host detail | Gate fails |
| Tunnel provider config implies process/runtime ownership | Reject the docs/fixture change or move it to a dedicated architecture task |
| Bash cannot run in the current environment | At least `bash -n` must pass and the reason actual execution could not run must be recorded |

### 5. Good/Base/Bad Cases

- Good: add a new Cloudflare Tunnel fixture that declares provider kind and
  `token_env`, then update both gate scripts with the expected checks and
  redaction assertions.
- Base: cookbook docs add provider guidance without changing fixtures; run
  `git diff --check` and the PowerShell gate.
- Bad: adding a tunnel supervisor, endpoint discovery route, wildcard CORS
  origin, or raw `public_url` echo in config-check output.

### 6. Tests Required

- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/remote-access-config-gate.ps1`.
- `bash -n scripts/remote-access-config-gate.sh`.
- Actual Bash gate when the shell environment has working Cargo/Rust.
- `python .trellis/scripts/task.py validate <remote-access-task-dir>`.
- `git diff --check`.
- `cargo fmt --all -- --check` only when Rust code changes.

### 7. Wrong vs Correct

#### Wrong

```toml
[network]
exposure_mode = "tunnel_provider"
external_base_url = "https://nako.example.com?token=secret"
```

Embedding provider or bearer secrets in URLs makes config-check, logs, and
support bundles harder to redact.

#### Correct

```toml
[[network.tunnel_providers]]
id = "cloudflared"
kind = "cloudflare_tunnel"
public_url = "https://nako.example.com"
token_env = "NAKO_TUNNEL_TOKEN"
```

Tunnel credentials stay in environment-backed operator secrets; Nako records
only readiness declarations and redacted diagnostics.

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
