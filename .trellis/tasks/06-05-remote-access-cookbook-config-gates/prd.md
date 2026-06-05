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

- [ ] Cookbook docs explain supported remote access shapes and non-goals.
- [ ] At least one reverse-proxy fixture and one tunnel-provider fixture are
      validated by config-check or release-gate tooling.
- [ ] Fixture output assertions prove sensitive network fields stay redacted.
- [ ] Existing local-only/private-network examples remain conservative.
- [ ] No Public Client endpoint discovery route is added.
- [ ] No built-in tunnel provider runtime is added.

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
