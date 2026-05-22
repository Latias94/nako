# Access Boundary And Token Authentication

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

Nako now has a public client API contract. Before web, Flutter, CLI, remote
access, or tunnel work builds on that contract, the server needs an explicit
inbound authentication boundary. Otherwise client work would normalize a
network-accessible unauthenticated API.

## Relevant Authority

- ADRs:
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
- Existing docs:
  - `docs/api/HTTP_API.md`
  - `docs/development/LOCAL_SETUP.md`
  - `docs/workstreams/public-client-api/`
  - `docs/workstreams/public-api-contract/`
- Related crates and modules:
  - `crates/nako-client-protocol`
  - `crates/nako-api`
  - `crates/nako-server/src/config.rs`
  - `crates/nako-server/src/http.rs`
  - `crates/nako-server/src/http/error.rs`
  - `crates/nako-server/src/http/tests/*`
  - `crates/nako-addon-protocol`
  - `crates/nako-events`

## Starting Audit

- There is no inbound client authentication on server HTTP routes today.
- Addon auth is outbound: Nako calls sidecar addons with bearer or
  shared-secret credentials.
- Webhook signing is outbound: Nako signs deliveries to webhook receivers.
- Metadata, automation, and WebDAV secrets are outbound provider/backend
  credentials resolved from environment references.
- `SecretString` already redacts resolved secrets in debug/display/serialization
  contexts.
- M30 added public `ClientErrorCode` and `ErrorResponse` contracts, but there
  is no `unauthorized` code yet.

## Problem

- Future clients and remote access need a server-owned inbound access boundary.
- Existing outbound integration secrets cannot safely double as client auth.
- Auth failures must use the same public v1 error envelope as the rest of the
  API.
- The first implementation must be useful without committing Nako to a full
  user account system or RBAC model.

## Target State

- Server config has an `[auth]` section with auth enabled by default and a
  token environment reference.
- `GET /health` remains unauthenticated for readiness/preflight.
- Every other HTTP route requires `Authorization: Bearer <token>` when inbound
  auth is enabled.
- Auth failures return `401 unauthorized` using the M30 error envelope.
- Correct bearer tokens allow existing route behavior.
- Tests prove missing token, wrong token, correct token, health bypass, and no
  token leakage.
- Docs separate inbound client auth from addon/webhook/provider outbound auth.

## In Scope

- Add minimal config and docs for inbound bearer token auth.
- Add protocol-owned `unauthorized` and `forbidden` error codes if needed.
- Add HTTP middleware for bearer-token validation.
- Add route-level tests for the first auth boundary.
- Update HTTP API docs, local setup docs, ADR index, roadmap, and goal map.

## Out Of Scope

- User accounts, registration, login sessions, passwords, OAuth/OIDC, LDAP,
  passkeys, or RBAC.
- Library-level ACLs, sharing, or multi-user permissions.
- Flutter, web, or CLI client implementation.
- Tunnel/NAT traversal implementation.
- Changing addon outbound auth or webhook signing semantics.

## Architecture Direction

Keep inbound auth as a thin HTTP boundary. The application services should not
receive or store bearer tokens. The middleware validates the request before it
reaches route handlers, and route handlers continue to work with the existing
application services.

## Closeout Condition

This lane can close when:

- the M31 auth/access boundary is documented in an ADR and workstream;
- config exposes a safe default token environment reference;
- HTTP middleware protects every non-health route when auth is enabled;
- auth failures use public protocol error codes and do not leak tokens;
- HTTP tests cover missing, wrong, and correct bearer tokens;
- docs distinguish inbound client auth from outbound integration auth;
- full validation gates pass;
- and follow-ons are explicitly recorded.
