# 0024: Add an Inbound Token Authentication Boundary

## Status

Accepted.

## Context

M29 and M30 made Taru's public client API contract useful for future Flutter,
web, CLI, and SDK consumers. That also makes the absence of inbound
authentication more important: without a server-side access boundary, future
clients, remote access, and tunnel/NAT traversal work would grow around an
unsafe default.

Taru already has several secret-bearing integration paths:

- addon resource auth, where Taru calls an addon with bearer or shared-secret
  credentials;
- webhook signing, where Taru signs outbound event deliveries;
- metadata and automation provider secrets, where Taru resolves provider API
  keys from environment references;
- WebDAV credentials, where Taru authenticates to a remote storage backend.

Those are outbound integration secrets. They must not be treated as inbound
client authentication.

## Decision

Taru will add a separate inbound HTTP authentication boundary for client and
admin access.

The first implementation is bearer-token based:

- server config owns an `[auth]` section;
- auth is enabled by default;
- the default token source is an environment variable reference,
  `TARU_ADMIN_TOKEN`;
- `GET /health` remains unauthenticated as a preflight/readiness endpoint;
- all other HTTP routes require `Authorization: Bearer <token>` when auth is
  enabled;
- authentication failures use the public v1 error envelope and stable
  `unauthorized` error code;
- resolved token values must not appear in debug output, API responses, logs,
  job payloads, or diagnostics.

Local development and tests may explicitly disable inbound auth through config,
but production-facing examples should keep auth enabled and use a secret
environment reference.

## Consequences

- Future Flutter, web, CLI, SDK, and tunnel work can assume a real inbound
  access boundary exists.
- The first auth slice remains small and testable without committing to user
  accounts, password storage, OAuth/OIDC, LDAP, passkeys, or RBAC.
- Existing addon, webhook, metadata, automation, and storage secret semantics
  stay separate.
- Route tests must cover no token, wrong token, correct token, and unauthenticated
  health behavior.
- API docs must tell clients to use the bearer token without documenting
  resolved token values.

## Alternatives Considered

- Leave auth to reverse proxies: rejected as the default because Taru plans
  first-party clients and remote access.
- Add user accounts immediately: rejected as too broad for the first access
  boundary.
- Reuse addon bearer auth: rejected because addon auth is outbound and
  resource-scoped, not inbound client access.
- Make auth disabled by default: rejected because remote access and tunnel
  work would inherit an unsafe baseline.

## Related Workstreams

- `docs/workstreams/public-client-api/`
- `docs/workstreams/public-api-contract/`
- `docs/workstreams/access-boundary-auth/`
