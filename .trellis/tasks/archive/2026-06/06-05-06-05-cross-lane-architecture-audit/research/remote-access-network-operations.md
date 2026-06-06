# Remote Access, Network Tunnel, And Operations-Release Audit

## Scope

Reviewed Nako's remote access, network tunnel, trusted proxy/header,
endpoint discovery, operator diagnostics, and operations-release surfaces.

Primary references:

- `CONTEXT.md`
- `docs/adr/0053-application-control-plane-boundary.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/network-access-boundary/`
- `docs/deployment/SELF_HOSTED.md`
- `deploy/**`
- `scripts/release-gate.*`
- `crates/nako-server/src/config.rs`
- `crates/nako-server/src/config/preflight.rs`
- `crates/nako-server/src/http.rs`
- `crates/nako-server/src/http/network.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/tests/system.rs`
- `crates/nako-api/src/admin/network.rs`
- `apps/admin-web/src/features/settings/SettingsPage.tsx`
- `crates/nako-client/src/lib.rs`
- `crates/nako-client-core/src/**`
- `docs/adr/0024-inbound-token-authentication-boundary.md`
- `docs/adr/0036-short-lived-browser-playback-tickets.md`
- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- `docs/adr/0040-casting-as-renderer-session-adapter.md`
- `docs/adr/0041-renderer-cast-safe-transport-tickets.md`

## Current State

Remote access is not starting from zero. The completed
`network-access-boundary` workstream already shipped the policy/readiness
foundation:

- `NetworkAccessConfig` owns `exposure_mode`, `external_base_url`,
  `trusted_proxy_headers`, `trusted_proxy_sources`, `allowed_origins`, and
  declarative `tunnel_providers`.
- `config-check` validates local-only, private-network, reverse-proxy, and
  tunnel-provider modes without leaking URL, token, origin, forwarded-header,
  database, provider, or path secrets.
- HTTP request-time enforcement is centralized in `http/network.rs`.
  Origin checks are default-deny when configured, CORS preflight is explicit,
  and forwarded host/proto are trusted only when proxy headers are enabled and
  the remote source matches exact-IP or CIDR policy.
- Root router layering keeps `/health` public, returns `x-request-id` through
  the trace middleware, and preserves auth behavior on protected routes.
- `/admin/v1/system/config` already exposes redacted network readiness:
  exposure mode, readiness checks, external endpoint scheme and host
  fingerprint, trusted proxy source count, origin count, tunnel provider
  declaration state, and token presence.
- Admin Web settings renders the safe network readiness summary through the
  generated Admin contract.
- Deployment examples stay conservative: source/host examples default to
  loopback and `local_only`; container examples bind inside the container and
  declare `private_network`; compose publishes host port on `127.0.0.1`.

What is still missing:

- no Public Client endpoint-discovery route or contract;
- no LAN versus remote endpoint selection model for clients;
- no built-in `cloudflared`, Tailscale, ngrok, STUN/TURN, relay, or hole
  punching runtime;
- no remote access cookbook beyond short reverse-proxy/tunnel config snippets;
- no release-gate mode that proves remote access examples or cookbook configs;
- no operator drill-down that verifies actual reverse proxy/tunnel reachability;
- no single pure classifier shared by config preflight and Admin readiness.

## Findings By Requested Surface

### Remote Access Endpoint

**Status**: policy foundation shipped; endpoint discovery not started.

Current interface:

- operator config: `external_base_url`;
- config validation: HTTPS is required for reverse-proxy and tunnel-provider
  exposure modes;
- Admin diagnostic: `external_endpoint.configured`, `scheme`, and
  `host_fingerprint`;
- HTTP runtime annotation: `x-nako-external-origin` can be returned from trusted
  forwarded host/proto headers.

Important distinction:

`x-nako-external-origin` is request-time annotation. It is not a client
endpoint inventory, not a public discovery contract, and not enough for LAN
versus remote endpoint selection.

Candidate tasks:

1. **self-hosted-remote-access-cookbook**
   - Type: ready bounded implementation.
   - Scope: docs and example configs for Caddy, Nginx, DDNS, Tailscale, and
     Cloudflare Tunnel; include playback ticket caveats and exact-origin CORS
     examples.
   - Files likely touched: `docs/deployment/SELF_HOSTED.md`,
     `docs/deployment/RELEASE_CHECKLIST.md`, maybe `deploy/**`.
   - Parallel safety: high, as long as it does not change config structs,
     Admin DTOs, or Public Client contracts.

2. **remote-access-config-fixture-release-gate**
   - Type: ready bounded implementation.
   - Scope: add docs/script fixtures that run `config-check --json` against
     reverse-proxy and tunnel-provider sample configs and assert redaction
     invariants.
   - Files likely touched: `scripts/release-gate.*`, `docs/deployment/**`,
     `deploy/**`.
   - Parallel safety: medium-high; serialize with release packaging or broad
     operations-release work.

3. **public-client-endpoint-discovery-contract**
   - Type: architecture audit before implementation.
   - Scope: decide whether authenticated clients receive configured remote/LAN
     endpoint candidates after connecting to one known endpoint, or whether
     local discovery is a separate client/platform concern.
   - Files likely touched after design: `nako-client-protocol`, `nako-api`,
     `nako-server`, `nako-client-core`, generated SDKs, docs.
   - Parallel safety: low; this crosses Public Client contracts, SDK behavior,
     auth, playback/cast transport, and client surfaces.

### Network Tunnel Provider

**Status**: declarative readiness shipped; concrete runtime deferred.

Current interface:

- config records provider `id`, `kind`, `public_url`, and `token_env`;
- supported provider kinds are `external`, `cloudflare_tunnel`,
  `tailscale_funnel`, and `ngrok`;
- config-check requires HTTPS `public_url` and non-empty token environment;
- Admin diagnostics expose provider ID, kind, endpoint configured/scheme/host
  fingerprint, token env name, and token presence, but not raw URL or token.

Architecture constraint:

ADR 0053 and `CONTEXT.md` both say Nako should not become a first-party relay
or built-in tunnel provider in the first phase. ADR 0034 also notes that tunnel
integration may need a separate Addon Sidecar or Addon Package when it has host
networking, privileged network config, relay credentials, or stricter
isolation needs.

Candidate tasks:

1. **tunnel-provider-cookbook-and-readiness-matrix**
   - Type: ready bounded implementation.
   - Scope: cookbook table for Cloudflare Tunnel, Tailscale Funnel, ngrok, and
     generic reverse proxy; include config snippets, expected readiness
     diagnostics, and "Nako does not start this process" wording.
   - Parallel safety: high if docs-only.

2. **network-tunnel-provider-runtime-decision**
   - Type: architecture audit / product decision.
   - Scope: decide whether concrete tunnel runtime belongs in an Addon Sidecar,
     future Addon Manager install guide, external operator docs, or a separate
     operations runtime. Do not implement a process supervisor during this task.
   - Parallel safety: medium-low; conflicts with Addon Manager, official addon
     catalog, operations-release, and security work.

3. **tunnel-provider-health-check-diagnostics**
   - Type: ready-ish implementation after the runtime decision is bounded.
   - Scope: read-only Admin diagnostics that can report declared provider
     reachability/status without echoing raw network errors, private relay URLs,
     tokens, headers, or provider payloads.
   - Parallel safety: medium; serializes with Admin DTO/generated contract and
     any Addon runtime-readiness work.

### Trusted Proxy And Header Policy

**Status**: request-time enforcement shipped.

Current interface:

- trusted forwarded headers are default-deny;
- `x-forwarded-host` and `x-forwarded-proto` are accepted only from trusted
  proxy source IPs/CIDRs and only when `trusted_proxy_headers = true`;
- forwarded host/proto values are sanitized: malformed multi-hop host values,
  path/query/credential-bearing values, and untrusted sources are ignored;
- allowed origins are exact and case-insensitive;
- CORS preflight allows authorization, content-type, range, and `x-request-id`;
- protected route auth precedence is preserved: missing/invalid bearer auth
  remains a `401` before origin rejection.

Candidate tasks:

1. **network-policy-classifier-deepening**
   - Type: fearless refactor candidate, but only worth doing before adding new
     readiness states.
   - Problem: `config/preflight.rs` and `http/admin.rs` independently classify
     similar network access facts into different status/reason models. This is
     manageable today, but adding endpoint discovery, tunnel health, or more
     proxy states will create drift risk.
   - Solution: extract a pure internal network policy classifier in
     `nako-server::config` or a focused submodule. Config preflight maps it to
     `ConfigPreflightCheck`; Admin diagnostics maps it to Admin DTOs.
   - Benefits: higher locality for network policy changes and one test surface
     for safety/redaction invariants.
   - Deletion test: deleting one of the duplicated classifiers would currently
     force the same auth/external endpoint/proxy/origin/tunnel checks to
     reappear in the other. That is real, but not urgent until new states are
     introduced.

2. **trusted-proxy-contract-refinement**
   - Type: architecture audit before implementation.
   - Scope: decide whether Nako should support RFC `Forwarded`, forwarded port,
     multiple proxy hops, or explicit proxy chain depth. The current narrow
     `X-Forwarded-Host`/`X-Forwarded-Proto` contract is safer and enough for
     the cookbook first slice.
   - Parallel safety: low with auth/CORS/trace-context changes.

### Endpoint Discovery

**Status**: not started.

Current client behavior:

- `NakoClient::new` requires a caller-supplied absolute `base_url`;
- `nako-client-core` request builders take `base_url` directly;
- there is no Public Client route returning configured endpoint candidates;
- there is no LAN discovery protocol, mobile/desktop auto-discovery surface,
  or remote endpoint failover model.

Why it needs architecture first:

- Clients need an endpoint before they can call a server. A server route can
  only offer alternates after a first endpoint already works.
- Returning raw configured endpoints is useful but sensitive; it may expose
  LAN hostnames, private domains, or tunnel hosts.
- Playback tickets, renderer cast-safe transport, and remote-network casts need
  endpoint choice to stay aligned with access policy and ticket validation.
- Public Client changes affect `nako-client-protocol`, generated SDKs,
  `nako-client-core`, web/native clients, and API docs.

Candidate task:

**client-endpoint-selection-architecture**

- Type: architecture audit.
- Output: define endpoint candidate kinds, auth/public visibility, LAN versus
  remote semantics, cache/expiry, redaction rules, and how clients fall back
  when an endpoint fails.
- Parallel safety: unsafe with client-surface, playback/cast transport, or
  Public Client SDK work unless one planner owns the contract.

### Operator Diagnostics

**Status**: good first read model shipped; drill-down and reachability are open.

Current interface:

- `/admin/v1/system/config` is the safe read model;
- Admin Web settings renders high-level network readiness;
- no raw external URL, trusted proxy source, origin, tunnel URL, token,
  forwarded header, local path, or provider secret is returned.

Candidate tasks:

1. **admin-network-diagnostics-drilldown**
   - Type: ready bounded implementation if kept read-only.
   - Scope: extend Admin diagnostics or add a focused Admin network page that
     shows the existing readiness checks with operator copy, suggested next
     action, and links to cookbook sections. Avoid new mutation routes.
   - Parallel safety: medium; serializes with Admin DTO, generated contract,
     and Admin Web settings changes.

2. **remote-access-operator-incident-bundle**
   - Type: architecture audit before implementation.
   - Scope: decide how much network readiness, request trace, config-check, and
     proxy/tunnel state can be exported without exposing hostnames, private
     addresses, tokens, URLs, or headers.
   - Parallel safety: low with observability/trace-context work.

## Architecture Audit Vs Ready Implementation Vs Fearless Refactor

### Architecture Audit

- Public Client endpoint discovery and LAN/remote endpoint selection.
- Concrete Network Tunnel Provider runtime placement: external docs, Addon
  Sidecar, Addon Manager install guide, or operations runtime.
- Trusted proxy contract expansion beyond the current narrow safe headers.
- Remote playback/cast endpoint policy, especially for Renderer Sessions and
  cast-safe transport tickets.
- Operator incident bundle shape for remote access diagnostics.

### Ready Bounded Implementation

- Self-hosted remote access cookbook.
- Reverse-proxy/tunnel config-check fixture in release gate or docs validation.
- Tunnel provider cookbook/readiness matrix.
- Admin network diagnostics drill-down if it stays read-only and uses existing
  DTO safety rules.

### Fearless Refactor

- `network-policy-classifier-deepening`: extract shared pure policy
  classification before adding endpoint discovery, tunnel health, or more
  readiness states.

Do not refactor the current narrow HTTP proxy enforcement just to make it more
generic. There is one concrete adapter today: trusted `X-Forwarded-*` handling
behind configured proxy sources. A broad proxy abstraction would be a
hypothetical seam until RFC `Forwarded`, multi-hop, or provider-specific
behavior is accepted.

## Parallel Conflict Surfaces

- `crates/nako-server/src/config.rs` and
  `crates/nako-server/src/config/preflight.rs`: serialize config shape,
  config-check, and operations-release validation.
- `crates/nako-server/src/http.rs` and `crates/nako-server/src/http/network.rs`:
  serialize trusted proxy/header, CORS, auth order, and trace middleware work.
- `crates/nako-api/src/admin/network.rs`,
  `crates/nako-api/src/admin_contract.rs`,
  `apps/admin-web/src/adminApi/generated/contract.ts`, and
  `apps/admin-web/src/features/settings/SettingsPage.tsx`: serialize Admin
  network diagnostics and generated contract changes.
- `crates/nako-client-protocol`, `crates/nako-client-core`, `crates/nako-client`,
  generated SDKs, and client apps: serialize endpoint discovery and client
  endpoint selection work.
- `docs/deployment/**`, `deploy/**`, `scripts/package-release.*`, and
  `scripts/release-gate.*`: coordinate operations-release and packaging lanes.
- `crates/nako-addon-protocol`, `crates/nako-official-addon-catalog`, official
  addon repo work, and Addon Manager planning: coordinate any concrete tunnel
  provider runtime or sidecar packaging decision.
- `CONTEXT.md`, ADR 0053, and `docs/architecture/CONTROL_PLANE.md`: serialize
  terminology or baseline changes.

## Recommended Priority

1. **Run ready docs/ops work first**:
   `self-hosted-remote-access-cookbook` plus
   `remote-access-config-fixture-release-gate`.
   This is high value, low conflict, and gives operators usable guidance before
   any risky runtime or Public Client contract expansion.

2. **Run an architecture audit for endpoint discovery next**:
   `client-endpoint-selection-architecture`.
   This should not be implemented casually because it crosses Public Client,
   SDK, auth, playback/cast transport, and client platform behavior.

3. **Queue a small fearless refactor only when new network states are about to
   land**:
   `network-policy-classifier-deepening`.
   It is justified by real drift risk between config preflight and Admin
   readiness, but it does not need to run before docs-only cookbook work.

4. **Defer concrete tunnel runtime**:
   Treat `network-tunnel-provider-runtime-decision` as a product/architecture
   task. The likely direction is an Addon Sidecar or Addon Manager install-guide
   surface for providers needing host networking or relay credentials, not core
   `nako-server` process supervision.

5. **Keep Admin network drill-down as a parallel sidecar only if Admin/API
   contracts are free**:
   it is useful, but it conflicts with other Admin diagnostics, settings, or
   generated contract work.

## Recommended Parallel Queue Shape

- Lane A, operations-release: cookbook and config-check fixtures.
- Lane B, architecture-planning: endpoint discovery and client endpoint
  selection PRD.
- Lane C, addons-automation planning: tunnel provider runtime placement, only
  after endpoint discovery has a direction.
- Optional Lane D, admin-web/API: read-only network diagnostics drill-down, only
  if no other active task is touching Admin generated contracts.

Unsafe to parallelize:

- endpoint discovery implementation with client SDK/playback/cast work;
- trusted proxy/header changes with auth or trace-context middleware changes;
- tunnel provider runtime with Addon Manager or official addon catalog changes;
- Admin diagnostics drill-down with unrelated Admin DTO/generated contract
  changes.

## Documentation Updates Needed

- `docs/architecture/CONTROL_PLANE.md`: clarify that policy/readiness is
  shipped by `network-access-boundary`, while endpoint discovery remains not
  started.
- `docs/architecture/OPERATIONS_RELEASE.md`: make
  `self-hosted-remote-access-cookbook` the next ready operations-release task
  and mention config-check fixture validation.
- `docs/architecture/LANES.md`: remote access should be listed as a safe
  operations/control-plane sidecar only for docs/config-check work; endpoint
  discovery and tunnel runtime should require planner coordination.
