# Access Boundary And Token Authentication Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Starting Repro

- Server HTTP routes are currently reachable without inbound client auth.
- Addon auth, webhook signing, provider secrets, and WebDAV credentials exist,
  but they are outbound integration secrets.
- `taru-client-protocol` has stable public error codes, but not auth-specific
  codes yet.

## Gate Set

### Targeted Iteration Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
```

### Protocol Direction Gate

```bash
cargo tree -p taru-client-protocol
```

This must not show dependencies on `taru-core`, `taru-streaming`,
`taru-transcode`, or `taru-server`.

### Auth Focus Gate

```bash
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo nextest run -p taru-server config --no-fail-fast
cargo nextest run -p taru-server http::tests::system --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo nextest run -p taru-server http::tests --no-fail-fast
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Anchors

- `docs/adr/0024-inbound-token-authentication-boundary.md`
- `docs/api/HTTP_API.md`
- `docs/development/LOCAL_SETUP.md`
- `crates/taru-client-protocol/src/lib.rs`
- `crates/taru-api/src/lib.rs`
- `crates/taru-server/src/config.rs`
- `crates/taru-server/src/http.rs`
- `crates/taru-server/src/http/tests/system.rs`

## Prompt-To-Artifact Checklist

- Establish HTTP access boundary:
  ADR 0024, auth middleware, and route tests.
- Define Public Client API, Server Admin/Internal API, and outbound integration
  auth boundaries:
  DESIGN.md, ADR 0024, HTTP API docs, and local setup docs.
- Implement token authentication foundation:
  config, middleware, tests.
- Preserve M30 error envelope:
  `ClientErrorCode`, `ErrorResponse`, and auth failure tests.
- Cover route-level auth behavior:
  missing token, wrong token, correct token, health bypass, no token leakage.
- Validate:
  final gate output recorded before closeout.

## Recorded Evidence

### ABA-010 Scope And Boundary Baseline

- Workstream docs define the M31 inbound auth problem, target state, non-goals,
  task ledger, gate set, and prompt-to-artifact checklist.
- ADR 0024 records the inbound token authentication boundary decision.

### ABA-020 Protocol And Config Slice

- `taru-client-protocol` owns stable `unauthorized` and `forbidden` error
  codes.
- `TaruServerConfig` owns `[auth]` with auth enabled by default and
  `TARU_ADMIN_TOKEN` as the default token environment reference.
- Test-only app/router constructors explicitly use `AuthConfig::disabled()` so
  existing route tests opt out of auth instead of depending on ambient
  environment.

### ABA-030 HTTP Middleware Slice

- `crates/taru-server/src/http/auth.rs` validates `Authorization: Bearer
  <token>` before non-health route handlers run.
- `GET /health` bypasses auth for readiness/preflight.
- Missing or invalid bearer tokens return `401` with `WWW-Authenticate:
  Bearer` and an `unauthorized` public error envelope.
- Auth failures do not include the expected token or attempted token in the
  response body.

### ABA-040 Docs And Route Evidence Slice

- `docs/api/HTTP_API.md` documents bearer-token use, public health, and auth
  error behavior.
- `docs/development/LOCAL_SETUP.md` documents local `TARU_ADMIN_TOKEN` setup
  and test auth opt-out.
- Route-level HTTP tests prove protected and public behavior.

### ABA-050 Closeout

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run -p taru-client-protocol --no-fail-fast`: passed, 4 tests.
- `cargo nextest run -p taru-server config --no-fail-fast`: passed, 11 tests.
- `cargo nextest run -p taru-server http::tests::system --no-fail-fast`:
  passed, 4 tests.
- `cargo nextest run -p taru-server http::tests --no-fail-fast`: passed, 35
  tests.
- `cargo nextest run --workspace --no-fail-fast`: passed, 256 tests.
- `cargo tree -p taru-client-protocol`: passed; only `serde` is a normal
  dependency and `serde_json` is dev-only.
- `git diff --check`: passed with Git CRLF normalization warnings only.
