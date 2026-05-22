# Network Access Boundary — Handoff

Status: Complete
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

NAB-040 is complete. It added Admin-only network readiness diagnostics to
`/admin/v1/system/config`, including exposure mode, readiness checks, external
endpoint scheme plus host fingerprint, trusted proxy source counts, browser
origin counts, tunnel-provider declaration state, and token presence booleans.
It refreshed the generated Admin Web contract and typed Admin Web data mapping
so the console can render network readiness without raw URLs, hostnames,
credential values, forwarded headers, local paths, or Public Client API churn.

NAB-050 is complete. Final closeout evidence is recorded, this workstream is
marked complete, and the remaining work is split into follow-ons rather than
hidden in the network lane.

## Closeout State

- Task ID: NAB-050
- Status: DONE
- Final scope:
  - `docs/workstreams/network-access-boundary`
  - `docs/workstreams/post-rpd-product-hardening`
  - `docs/workstreams/README.md`
- Review result: no blocking findings. The target state is met, and built-in
  NAT traversal runtime, client endpoint discovery, identity/RBAC, downloader
  protocols, AI-assisted library ops, Addon runtime/distribution, and library
  mutation remain split follow-ons.

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
- Admin readiness diagnostics belong to the Admin boundary. Public Client API
  and `taru-client-protocol` remain untouched until a dedicated remote-client
  endpoint discovery lane exists.
- Tunnel provider config/readiness is declarative only. Starting cloudflared,
  ngrok, Tailscale Funnel, relay services, or NAT traversal is a follow-on lane.

## Blockers

- None.

## Next Recommended Action

Return to `post-rpd-product-hardening`.

Recommended next mainline lane: `ai-assisted-library-ops`, scoped to Generated
Artifact proposal/readiness and explicit accept/reject planning before Addon
runtime/distribution consumes AI or side-effect proposal queues.
