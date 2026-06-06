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
- Process-backed HLS and remux tests must use bounded polling helpers for
  transcode state and artifact readiness instead of immediate assertions. Full
  workspace nextest on Windows can start many fake FFmpeg processes
  concurrently; readiness budgets should cover that startup tail while still
  bounding hangs.
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

## Scenario: M1 Operator Journey Smoke Gate

### 1. Scope / Trigger

- Trigger: changing `scripts/m1-operator-journey-smoke.ps1`, changing the
  meaning of Product-Operator M1 smoke coverage, or moving the focused server,
  Admin Web, or docs-safe gates that script composes.
- Purpose: provide one repeatable M1 smoke entry point that maps the operator
  journey to existing deterministic gates without publishing release artifacts
  or adding runtime behavior.

### 2. Signatures

- PowerShell gate:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-operator-journey-smoke.ps1 [-Mode docs|server|admin-web|fast] [-SkipDocsGate]`.
- Default mode: `fast`.
- `docs` mode runs only the docs-safe release gate.
- `server` mode runs docs-safe release gate plus `scripts/self-host-smoke.ps1`.
- `admin-web` mode runs docs-safe release gate plus the focused Admin Web
  route/media smoke tests.
- `fast` mode runs docs-safe release gate, server self-host smoke, and focused
  Admin Web route/media smoke tests.

### 3. Contracts

- The script composes existing gates; it must not duplicate server smoke,
  release-gate, or Admin Web route/media test logic inline.
- The smoke maps Product-Operator M1 to concrete checks for configured Media
  Library visibility, scan/index visibility, browse/source inventory, playback
  readiness, Admin diagnostics/repair, and redaction.
- The script must not introduce schema changes, generated contract changes,
  API route shape changes, hidden runtime behavior, release artifact
  publication, automatic source hash scheduling, or automatic duplicate merge
  behavior.
- `-SkipDocsGate` is for focused local iteration only. Task evidence must state
  when it was used, and closeout should run a mode that includes the docs gate.
- Script output may print safe command names and coverage categories, but must
  not print bearer tokens, playback tickets, local paths, source locators,
  playback output paths, database URLs, source fingerprints, content hashes, or
  secret environment values.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Invalid `-Mode` value | PowerShell parameter validation rejects it before any gate runs |
| Docs-safe release gate fails | Script exits non-zero and does not run later gates in that mode |
| Server self-host smoke fails | Script exits non-zero and reports the failed step name |
| Admin Web route/media smoke fails | Script exits non-zero and reports the failed step name |
| A future change needs API/schema/generated-contract/runtime behavior | Stop and open a focused Trellis task instead of expanding the smoke script |
| A future gate needs live browser/manual evidence | Document it as an additional mode or follow-on; do not hide it under `fast` without explicit evidence |
| Output includes unsafe token/path/locator/secret material | Treat as a redaction failure and fix before closeout |

### 5. Good/Base/Bad Cases

- Good: add a focused `playback` mode that invokes an existing playback gate
  script and updates the coverage map, while keeping `fast` deterministic.
- Base: update the script to point at a renamed Admin Web focused test and run
  `-Mode fast` plus task validation.
- Bad: copy `self_host_smoke` setup into the script, call a live server with an
  inline token, publish package/container artifacts, or add route/schema
  behavior to make the smoke pass.

### 6. Tests Required

- `python ./.trellis/scripts/task.py validate <m1-smoke-task-dir>`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-operator-journey-smoke.ps1 -Mode docs`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-operator-journey-smoke.ps1 -Mode fast` for closeout when local Rust, Node, and Admin Web tooling are available.
- PowerShell parser check if a focused environment cannot run the full script.
- `git diff --check` after script, docs, task, or spec changes.

### 7. Wrong vs Correct

#### Wrong

```powershell
Write-Host "Token: $env:NAKO_ADMIN_TOKEN"
cargo run -p nako-server -- serve --publish-artifacts
```

The M1 smoke must not echo credentials or turn a validation gate into a release
publication command.

#### Correct

```powershell
Invoke-Step 'scripts/self-host-smoke.ps1' {
    pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1
}
```

Delegate to an existing focused gate and let that gate own its detailed setup,
assertions, and redaction behavior.

## Scenario: M1 Release Ladder Runner

### 1. Scope / Trigger

- Trigger: changing `scripts/m1-release-ladder.ps1`, changing the meaning of
  Product-Operator M1 release confidence, or changing how M1 composes
  release-gate and operator-smoke scripts.
- Purpose: provide one product-level validation entry point while preserving
  `scripts/release-gate.ps1` and `scripts/m1-operator-journey-smoke.ps1` as
  the owners of detailed gate logic.

### 2. Signatures

- PowerShell runner:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 [-Mode docs|smoke|fast|release-fast|playback|container|postgres|workspace|all] [-PostgresUrl <url>] [-SkipRedactionInventory]`.
- Default mode: `fast`.
- `fast` runs `release-gate.ps1 -Mode docs` once, then runs
  `m1-operator-journey-smoke.ps1 -Mode fast -SkipDocsGate`.
- `release-fast`, `playback`, `container`, `postgres`, and `workspace`
  delegate to the matching `release-gate.ps1` mode.
- `all` sequences fast, release-fast, playback, container, postgres, and
  workspace; after the first docs gate it skips repeated redaction inventory
  scans.
- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md` records the release-facing
  evidence meaning, tooling requirements, skipped-gate rules, and follow-up
  routing for every runner mode.

### 3. Contracts

- The runner is an orchestrator only. It must not copy the command list from
  `release-gate.ps1`, `self-host-smoke.ps1`, Admin Web tests, or playback
  release gates.
- Default `fast` mode must remain suitable for local M1 confidence. Expensive
  modes must stay explicit.
- The runner may print mode names, safe command names, and coverage categories.
  It must not print PostgreSQL URLs, bearer tokens, playback tickets, local
  media paths, Source Locators, playback output paths, raw fingerprints,
  content hashes, or secret environment values.
- Adding live-browser, package-publication, or release-artifact validation must
  be explicit new scope. Do not hide those steps under `fast`.
- Any runner mode addition, removal, rename, or semantic change must update
  `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`,
  `docs/deployment/RELEASE_CHECKLIST.md`, and
  `docs/architecture/OPERATIONS_RELEASE.md` in the same task.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Invalid `-Mode` value | PowerShell parameter validation rejects it before any gate runs |
| Docs release gate fails | Runner exits non-zero before M1 smoke in `fast` mode |
| M1 operator smoke fails | Runner exits non-zero and reports the failed step name |
| Expensive release-gate mode fails | Runner exits non-zero and reports the delegated mode |
| PostgreSQL URL is provided | Runner passes it to `release-gate.ps1` but prints only `<provided>` |
| Output includes unsafe URL/token/path/locator/secret material | Treat as a redaction failure and fix before closeout |

### 5. Good/Base/Bad Cases

- Good: add a future `live-browser` mode that invokes a dedicated browser smoke
  script and keeps default `fast` unchanged.
- Base: update the runner to call a renamed M1 smoke script and validate
  `-Mode docs` plus parser checks.
- Bad: paste the `release-gate.ps1` command list into the ladder, echo
  `$PostgresUrl`, publish artifacts, or add live browser work to default
  `fast`.

### 6. Tests Required

- PowerShell parser validation for `scripts/m1-release-ladder.ps1`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode docs -SkipRedactionInventory`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode fast -SkipRedactionInventory` for closeout when local Rust, Node, and Admin Web tooling are available.
- `rg -n "docs|smoke|fast|release-fast|playback|container|postgres|workspace|all" scripts/m1-release-ladder.ps1 docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
  or an equivalent review proving the matrix covers every runner mode.
- `python ./.trellis/scripts/task.py validate <m1-release-ladder-task-dir>`.
- `git diff --check`.

### 7. Wrong vs Correct

#### Wrong

```powershell
Write-Host "Postgres: $PostgresUrl"
cargo nextest run -p nako-server self_host_smoke --no-fail-fast
```

The runner must not print sensitive URLs or copy a detailed gate that already
has an owning script.

#### Correct

```powershell
Invoke-Step 'scripts/release-gate.ps1 -Mode playback' {
    pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode playback
}
```

Delegate to the existing release gate and keep M1 ladder logic at the product
orchestration layer.

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
