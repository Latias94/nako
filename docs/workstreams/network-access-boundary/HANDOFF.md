# Network Access Boundary — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

This lane is open as the next mainline child of `post-rpd-product-hardening`
after Downloads / Watch-Folder Intake closed.

Prerequisites are complete:

- `access-boundary-auth` added inbound bearer auth enabled by default.
- `release-packaging-and-distribution` made self-hosted deployment and
  config-check workflows explicit.
- `playback-transcode-ops-hardening` added Admin supportability evidence.
- `downloads-watch-folder-intake` proved acquisition intake without direct
  library writes or Public Client API churn.

NAB-010 is complete. The lane is scoped to network access policy/readiness, not
built-in NAT traversal runtime.

NAB-020 is complete. It added `NetworkAccessConfig`, explicit exposure modes,
tunnel-provider declarations, config-check validation for reverse-proxy,
private-network, and tunnel-provider modes, trusted proxy source requirements,
browser origin validation, tunnel token environment checks, deployment docs,
and example config updates. It did not start a tunnel runtime or change Public
Client API / `taru-client-protocol`.

NAB-030 is complete. It added a server HTTP network boundary that preserves
bearer-auth precedence on protected routes, rejects authenticated disallowed
browser origins without echoing the origin or token, keeps `GET /health`
public, handles allowed CORS preflight requests, annotates allowed origins,
and trusts `X-Forwarded-Host` / `X-Forwarded-Proto` only when forwarded headers
are enabled and the request remote address matches configured trusted proxy
sources by exact IP or CIDR. It also wires real `ConnectInfo<SocketAddr>` into
the served router. It did not add a built-in NAT traversal runtime or change
Public Client API / `taru-client-protocol`.

## Active Task

- Task ID: NAB-040
- Owner: unassigned
- Files:
  - `crates/taru-api/src/admin.rs`
  - `crates/taru-api/src/admin_contract.rs`
  - `crates/taru-server/src/http/admin.rs`
  - `apps/admin-web/src/adminApi`
- Validation:
  - `cargo nextest run -p taru-api admin_contract --no-fail-fast`
  - `cargo nextest run -p taru-server http::tests::system --no-fail-fast`
  - `npm run check` from `apps/admin-web`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- Status: READY
- Review: expose Admin-only network readiness diagnostics and typed Admin web
  contract/client support without adding Public Client API, tunnel runtime,
  identity/RBAC, downloader protocols, AI writes, Addon runtime, or library
  mutation.

## Decisions Since Opening

- Start with policy/readiness, not NAT traversal runtime.
- Treat reverse proxy, private-network, and tunnel-provider exposure as config
  modes with explicit safety checks.
- Keep inbound Admin bearer auth separate from Addon/Webhook/provider/storage
  outbound secrets.
- Admin diagnostics are allowed; Public Client API and `taru-client-protocol`
  changes are not part of the first slice.
- Network exposure config stores policy and readiness declarations only. It
  does not start cloudflared/ngrok/Tailscale, open relay sockets, or own NAT
  traversal.
- HTTP request-time enforcement is server-owned and split from future Admin
  readiness diagnostics.
- Trusted forwarded headers are default-deny, require enabled policy and a
  trusted remote source, and must not echo untrusted raw host/proto values.
- Origin enforcement must preserve auth order so missing/invalid bearer tokens
  remain `401` before origin rejection on protected routes.

## Blockers

- None for NAB-040.

## Next Recommended Action

Execute NAB-040: expose Admin-only network readiness diagnostics that summarize
network access mode, external endpoint readiness, trusted proxy policy, origin
policy, and tunnel-provider declarations with strict redaction and no Public
Client protocol changes.
