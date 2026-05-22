# Network Access Boundary Design

Status: Complete
Last updated: 2026-05-22

## Why This Lane Exists

Nako's server is now packageable, authenticated by default, diagnosable for
playback, and able to intake acquisition candidates without direct library
writes. A self-hosted media server still needs safe remote access: operators
will place Nako behind reverse proxies, tunnels, VPNs, LAN DNS names, or future
Nako-managed tunnel providers.

Without an explicit network access boundary, remote access work will likely
sprawl across config, HTTP handlers, auth, CORS, docs, Admin diagnostics, and
future tunnel runtimes. The first-principles risk is not how to punch through a
NAT; it is whether Nako can tell which external endpoints are trusted, which
headers are authoritative, which origins are allowed, and whether a deployment
is safe enough for first-party clients.

## Relevant Authority

- ADRs:
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
- Existing docs:
  - `docs/deployment/SELF_HOSTED.md`
  - `docs/workstreams/access-boundary-auth`
  - `docs/workstreams/release-packaging-and-distribution`
  - `docs/workstreams/post-rpd-product-hardening`
- Related code:
  - `crates/nako-server/src/config.rs`
  - `crates/nako-server/src/http`
  - `crates/nako-api/src/admin.rs`
  - `crates/nako-api/src/admin_contract.rs`
  - `apps/admin-web/src/adminApi`

## Problem

Operators need to know whether Nako is safe to expose beyond loopback:

- which external base URL clients should use;
- whether auth is enabled and health remains the only public route;
- whether trusted proxy headers are accepted only from trusted sources;
- whether TLS/proto/host rewriting is configured safely;
- whether browser origins are allowed deliberately rather than by wildcard;
- whether a tunnel provider is registered, healthy, and scoped;
- whether diagnostics can explain misconfiguration without exposing tokens,
  local paths, tunnel credentials, internal IPs beyond safe categories, or raw
  request headers.

## Target State

When this lane closes:

- Nako has an explicit Network Access Policy vocabulary and config shape.
- Config validation distinguishes local-only, reverse-proxy, private-network,
  and tunnel-provider exposure modes.
- Trusted proxy/header handling is explicit and default-deny.
- CORS/origin policy is explicit and redacted in diagnostics.
- Admin-only diagnostics report network readiness and blockers without leaking
  bearer tokens, tunnel credentials, internal URLs with secrets, raw headers,
  or local filesystem paths.
- Public Client API and `nako-client-protocol` remain unchanged unless a
  dedicated client-contract follow-on is opened.
- Built-in NAT traversal runtime and concrete relay/tunnel implementations are
  split follow-ons.

## In Scope

- Network access workstream docs and task ledger.
- Network exposure config/domain records for external URL, mode, trusted proxy,
  forwarded-header, origin, and tunnel-provider readiness policy.
- Config-check validation for unsafe exposure combinations.
- HTTP boundary enforcement for trusted forwarded headers and CORS/origin rules
  if existing code lacks a safe seam.
- Admin-only read model for network readiness diagnostics.
- Deployment docs for reverse proxy and tunnel-provider policy.
- Tests proving auth stays enabled by default and remote diagnostics are
  redacted.

## Out Of Scope

- Built-in NAT traversal, relay servers, TURN/STUN, hole punching, or persistent
  tunnel dialing runtime.
- User accounts, OAuth/OIDC, LDAP, passkeys, sharing links, or RBAC.
- Protocol downloader adapters or external download-client secrets.
- AI-generated artifacts or autonomous writes.
- Addon runtime/distribution.
- Public Client API or SDK changes unless split.
- Library file writes, promotion apply, NFO sidecar mutation, or media catalog
  mutations.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Inbound bearer auth is the correct prerequisite for remote access. | High | ADR 0024 and `access-boundary-auth` closeout | If user-account auth becomes urgent, split a dedicated identity/RBAC lane rather than expanding this one. |
| First safe slice should be policy/readiness, not a tunnel runtime. | High | Self-hosted docs already recommend reverse proxy/VPN/tunnel boundaries | If a concrete tunnel provider is required immediately, define it behind the provider readiness interface and keep credentials redacted. |
| Admin diagnostics are the right first operator surface. | High | ADR 0027 and prior Admin read-model lanes | If full UI is needed, split Admin UI polish after the read model is stable. |
| Public Client API should remain stable for this lane. | Medium | Remote access mostly changes deployment/server behavior | If clients need an endpoint-discovery route, open a client-contract task with protocol gate coverage. |

## Architecture Direction

Keep network access as a server/runtime boundary, not as a business-domain
mutation path:

```text
nako-server::config
  Owns operator-provided network exposure policy, external base URLs, trusted
  proxy CIDRs/sources, forwarded header mode, CORS/origin allow-list, and tunnel
  provider declarations.

nako-server::http
  Owns request-time enforcement: auth remains before protected routes,
  forwarded headers are trusted only under policy, CORS/origin behavior follows
  config, and diagnostics never echo raw sensitive headers.

nako-api::admin / nako-server::http::admin
  Own Admin-only network readiness DTOs/routes. These surfaces expose safe
  status categories and redacted endpoint references, not tokens, tunnel
  credentials, full raw headers, or local network secrets.

docs/deployment
  Own operator guidance for reverse proxies, TLS termination, private networks,
  and tunnel-provider integration.
```

## Closeout Condition

This lane is closed. It met the closeout condition when:

- network access policy/config has a stable vocabulary and validation tests;
- HTTP trusted-proxy/header and origin behavior is explicit or explicitly
  deferred with evidence;
- Admin diagnostics report network readiness safely;
- self-hosted docs explain reverse proxy/tunnel deployment boundaries;
- tests prove no token/header/credential/path leakage;
- Public Client API and `nako-client-protocol` remain unchanged or are covered
  by a dedicated client-contract decision;
- NAT traversal runtime, relay services, identity/RBAC, downloader protocols,
  AI, and Addon runtime are split or deferred.
