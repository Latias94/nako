# Access Boundary And Token Authentication Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M31 is closed. Taru now has a minimal inbound bearer-token authentication
boundary for server HTTP routes.

## Completed Scope

- `taru-client-protocol` owns public `unauthorized` and `forbidden` error
  codes.
- `taru-server` owns `[auth]` config with auth enabled by default and
  `TARU_ADMIN_TOKEN` as the default token environment reference.
- HTTP middleware protects every non-health route when auth is enabled.
- `GET /health` remains unauthenticated.
- Auth failures use the M30 error envelope, return `401`, include
  `WWW-Authenticate: Bearer`, and avoid token leakage.
- HTTP API and local setup docs describe the boundary.

## Decisions Since Last Update

- Inbound client auth is separate from addon/webhook/provider outbound secrets.
- Auth is enabled by default through a token environment reference.
- `GET /health` remains unauthenticated.
- Users, sessions, OAuth/OIDC, and RBAC are follow-ons.

## Blockers

- None.

## Follow-Ons

- OpenAPI/SDK generation should include a bearer auth scheme instead of
  documenting auth only in prose.
- User accounts, login sessions, OAuth/OIDC, LDAP, Passkey, and RBAC are still
  out of scope.
- Tunnel/NAT traversal should build on this inbound boundary rather than
  exposing unauthenticated routes.
- Future public route expansion should decide which admin/internal routes stay
  outside the permissive client contract.
