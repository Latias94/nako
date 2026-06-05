# Remote access cookbook and config gate fixtures

## Goal

Give self-hosted operators concrete remote access guidance and add config-check
fixture/release-gate coverage before any endpoint discovery or built-in tunnel
runtime work begins.

## Requirements

- Add or deepen cookbook guidance for Caddy, Nginx, DDNS, Tailscale Funnel,
  Cloudflare Tunnel, ngrok, and generic external tunnel providers.
- Include playback ticket, CORS/origin, HTTPS, trusted proxy, and tunnel token
  caveats.
- Add config-check fixtures or release-gate validation for representative
  reverse-proxy and tunnel-provider configurations.
- Keep tunnel providers external; do not add process supervision or tunnel
  runtime to `nako-server`.
- Preserve redaction of URLs, tokens, headers, private origins, and host
  details in diagnostics and test fixtures.

## Acceptance Criteria

- [x] Cookbook docs explain supported remote access shapes and non-goals.
- [x] At least one reverse-proxy fixture and one tunnel-provider fixture are
      validated by config-check or release-gate tooling.
- [x] Fixture output assertions prove sensitive network fields stay redacted.
- [x] Existing local-only/private-network examples remain conservative.
- [x] No Public Client endpoint discovery route is added.
- [x] No built-in tunnel provider runtime is added.

## Implementation Evidence

- Cookbook:
  `docs/deployment/REMOTE_ACCESS.md`.
- Self-hosted and release links:
  `docs/deployment/SELF_HOSTED.md` and
  `docs/deployment/RELEASE_CHECKLIST.md`.
- Reverse-proxy fixture:
  `deploy/remote-access/reverse-proxy.nako.toml`.
- Tunnel-provider fixture:
  `deploy/remote-access/tunnel-provider.nako.toml`.
- Config gates:
  `scripts/remote-access-config-gate.ps1` and
  `scripts/remote-access-config-gate.sh`.
- Spec update:
  `.trellis/spec/nako-server/backend/quality-guidelines.md`.
- PowerShell gate passed and verified `network.access`, `network.proxy`,
  `network.origins`, and `network.tunnel_providers` report `pass` while raw
  URLs, token values, proxy sources, forwarded header names, and `127.0.0.1`
  are absent from reports.
- Bash syntax gate passed. Actual Bash gate did not complete in the available
  WSL environment because non-login Bash lacked `cargo`; login Bash found
  Cargo but WSL `rustc 1.95.0` ICE'd while compiling `nako-server`, before the
  fixture assertions ran. Windows/PowerShell Cargo execution passed.

## Definition of Done

- Relevant docs and fixture checks pass.
- `cargo fmt --all -- --check` is run only if Rust code changes.
- `git diff --check` passes.

## Out of Scope

- No endpoint discovery API.
- No LAN versus remote client selection model.
- No process lifecycle for Cloudflare/Tailscale/ngrok.
- No Addon Manager implementation.

## Technical Notes

- Parent audit: `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/`
- Key research:
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/remote-access-network-operations.md`
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/synthesis.md`
- Likely files:
  - `docs/deployment/`
  - `deploy/`
  - `scripts/release-gate.*`
  - `crates/nako-server/src/config/preflight.rs` only if fixture support needs
    a small parser/test hook
