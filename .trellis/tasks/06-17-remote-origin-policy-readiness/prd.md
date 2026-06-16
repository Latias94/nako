# Remote Origin Policy Readiness

## Goal

Keep runtime Admin network readiness aligned with `config-check` for browser
origin policy. A remote self-hosted Nako instance must not show network
readiness as ready when the configured browser origins would be rejected by
preflight.

## What I Already Know

- `config-check` already rejects wildcard origins and invalid HTTP(S) origins
  in `crates/nako-server/src/config/preflight.rs`.
- Admin network diagnostics are built in
  `crates/nako-server/src/http/admin.rs` by `network_readiness_diagnostics`.
- Current `origin_policy_readiness_check` treats any non-empty
  `allowed_origins` as ready for remote exposure modes.
- Admin overview maps `AdminNetworkReadinessDiagnostics` into operator
  readiness, so a wrong network readiness result creates a wrong product-level
  operator signal.
- Network diagnostics must stay redacted: no raw endpoint, origin, proxy source,
  token, local host, path, or query values.

## Requirements

- For `reverse_proxy`, `tunnel_provider`, and `private_network`, Admin network
  readiness must mark invalid browser origins unavailable instead of ready.
- Invalid browser origins include wildcard `"*"`, blank strings, non-HTTP(S)
  origins, path-bearing origins, query-bearing origins, credential-bearing
  origins, and malformed values, matching the `config-check` origin policy.
- Missing browser origins for remote exposure modes remain degraded, not
  unavailable, preserving the current "CORS default-deny" operator signal.
- `local_only` remains ready for origin policy because remote browser access is
  disabled.
- Responses and tests must not expose raw origin values.
- Keep scope inside existing Admin network diagnostics and operator overview
  mapping.
- `config-example` and `config-check` must remain runnable without starting the
  server runtime, because the remote access fixture gate depends on
  `config-check --json --create-dirs`.

## Acceptance Criteria

- [ ] `GET /admin/v1/network/access` reports an unavailable origin-policy check
      when a remote exposure mode configures wildcard or otherwise invalid
      origins.
- [ ] The top-level Admin network readiness reason reflects the invalid origin
      when no higher-priority unavailable check exists.
- [ ] Admin overview's network operator check is unavailable with a
      redaction-safe source reason for the same configuration.
- [ ] Existing redaction tests continue to prove raw remote endpoint/origin
      material is absent.
- [ ] `config-example` and the remote access config gate run without the
      Windows debug stack overflow previously seen when config-only commands
      entered the async server command future.
- [ ] Focused server tests and formatting/check gates pass.

## Technical Approach

- Add a new `AdminNetworkReadinessReason` for invalid browser origin policy.
- Extract the existing preflight HTTP(S)-origin validation into a shared config
  helper so runtime Admin readiness and `config-check` cannot drift.
- Update `origin_policy_readiness_check` to use the helper before the
  configured/missing-origin branch.
- Keep `config-example` and `config-check` synchronous in `main.rs`; only
  commands that need `NakoApp` startup should create a Tokio runtime.
- Add focused route tests in `crates/nako-server/src/http/tests/system.rs` for
  network diagnostics and overview operator readiness.
- Regenerate Admin TypeScript contracts only if the generated enum output drifts.

## Out of Scope

- No new remote access runtime, tunnel supervisor, endpoint probing, or relay
  integration.
- No new Admin route.
- No change to `config-check` fixture semantics unless tests reveal an actual
  mismatch.
- No exposure of raw origin values in DTOs, logs, or test assertions.

## Technical Notes

- Relevant specs:
  - `.trellis/spec/nako-server/backend/http-api-patterns.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
  - `.trellis/spec/nako-server/backend/directory-structure.md`
  - `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  - `.trellis/spec/nako-api/backend/quality-guidelines.md`
- Related architecture:
  - `docs/architecture/CONTROL_PLANE.md`
  - ADR 0053 control-plane diagnostics and redaction baseline
- Network readiness priority means tests need auth enabled; otherwise
  `auth_disabled` correctly wins over origin policy. A configured-auth test
  helper keeps the invalid-origin assertion focused on the origin-policy check.
- Running the remote access PowerShell gate exposed an unrelated but blocking
  CLI reliability issue: Windows debug builds could stack overflow when
  config-only commands entered the full async command future. The fix is in
  scope because remote access readiness depends on `config-check` as an
  operator-facing preflight command.
