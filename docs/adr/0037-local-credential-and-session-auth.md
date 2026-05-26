# 0037: Add Local Credential and Session Authentication

## Status

Accepted.

## Context

Nako's first inbound authentication boundary is a configured Bearer token. That
token maps to the bootstrap administrator and is intentionally separate from
addon, webhook, metadata-provider, automation-provider, and storage secrets.

Nako now also has durable local users, coarse Roles, Library Access, and
Public Client effective-access enforcement. The missing link is authentication
for those local users. Without it, Media Web, desktop clients, native mobile
clients, and management context links either have to share the bootstrap
administrator token or cannot exercise per-user access at all.

Browser playback has a separate short-lived playback ticket transport. That
solves browser media element byte requests, but it does not replace normal user
login and JSON API authentication.

## Decision

Nako will add local password credentials and durable opaque sessions as the
first user-authentication model.

The first backend contract is:

- administrators provision or rotate a local password credential for an
  existing user through the Admin API;
- public self-registration is disabled by default and remains a separate
  onboarding/invitation concern;
- users log in through a Public Client API username/password route;
- successful login returns an opaque session token that clients send as
  `Authorization: Bearer <token>`;
- password credentials are stored only as password hashes;
- session tokens are stored only as token hashes;
- auth middleware accepts either the configured bootstrap administrator token or
  an active local session token;
- local session auth resolves to `AuthenticatedPrincipal` with the durable user
  id, principal id, roles, and bootstrap flag set to `false`;
- disabled users cannot create new sessions and active sessions for disabled
  users cannot authorize protected routes;
- logout revokes the current session when the request was authenticated by a
  local session token.

This ADR deliberately does not require cookie auth for the first slice. Cookies
may be layered later for browser ergonomics, but the server-owned session
authority and Public Client contract must work for web, desktop, mobile, CLI,
and SDK clients without assuming a browser.

The bootstrap administrator token remains supported for setup, automation, and
recovery. It is not stored as a user credential and must not be exposed through
account/session DTOs.

## Consequences

- Public Client API can represent real users instead of only the bootstrap
  administrator.
- Existing Library Access, playback state, and management gating can reuse the
  same `AuthenticatedPrincipal` extension regardless of whether the caller used
  the bootstrap token or a local session.
- Admin Web and Media Web can add login/account UI later without inventing a
  frontend-only account model.
- Session revocation and disabled-user enforcement become server-owned.
- Password policy, login rate limiting, invitation onboarding, account
  recovery, cookie transport, and SSO remain follow-ons that must not be hidden
  inside this first slice.
- Baseline migration edits are acceptable while Nako has no production users;
  production compatibility would require forward-only migrations instead.

## Alternatives Considered

- **Use only the configured admin token:** rejected because it cannot represent
  ordinary users, per-user Library Access, account switching, or revocation.
- **Expose public registration immediately:** rejected because a self-hosted
  media server needs explicit owner-controlled onboarding first.
- **Start with same-site httpOnly cookies:** useful for browser UX, but it is
  not sufficient for desktop, mobile, CLI, SDK, or Tauri transports by itself
  and adds CSRF/reverse-proxy decisions that should be layered after the core
  session authority exists.
- **Use playback tickets as login sessions:** rejected because playback tickets
  are scoped, short-lived media-byte credentials, not account sessions.
- **Adopt OAuth/OIDC, LDAP, or passkeys first:** rejected as too broad before
  local account semantics and Library Access enforcement are stable.

## Related Workstreams

- `docs/workstreams/access-boundary-auth/`
- `docs/workstreams/identity-and-library-access-contract/`
- `docs/workstreams/browser-playback-auth-transport/`
- `docs/workstreams/credential-session-auth/`
