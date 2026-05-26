# Credential Session Auth

Status: Active
Last updated: 2026-05-26

## Why This Lane Exists

Nako already has durable local users, Role assignments, Library Access policies,
bootstrap administrator behavior, and Public Client effective-access
enforcement. What is still missing is a way for those users to authenticate
without sharing the configured bootstrap administrator token.

That gap blocks Media Web account switching, desktop client sign-in, native
mobile sign-in, and permission-gated management context links. It also makes
Admin Web account controls incomplete because administrators can create users
and policy rows but cannot provision usable local credentials.

## Relevant Authority

- ADRs:
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
  - `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
  - `docs/adr/0037-local-credential-and-session-auth.md`
- Existing docs:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/identity-and-library-access-contract/`
  - `docs/workstreams/browser-playback-auth-transport/`
  - `docs/workstreams/client-surface-and-access-product-architecture/`

## Problem

The current inbound Bearer token is an instance-level bootstrap administrator
secret. It is useful for first setup and automation, but it cannot represent
ordinary users, per-user Library Access, account switching, session revocation,
or user-facing playback state.

Adding login superficially at the frontend would be misleading unless the
server has a durable credential/session model, redaction-safe API contracts, and
auth middleware that resolves session tokens into the same `AuthenticatedPrincipal`
used by Public Client and Admin API routes.

## Target State

When this lane closes:

- Local password credentials are stored as password hashes, never plaintext.
- Session tokens are opaque, returned only once, and stored only as token
  hashes.
- Administrators can set or rotate a local password for an existing user through
  the Admin API.
- A user can log in with username/password through the Public Client API and
  receive a bearer-compatible session token.
- Existing protected routes accept either the configured bootstrap admin token
  or an active local session token.
- Disabled users cannot create new sessions, and active sessions for disabled
  users cannot authorize protected routes.
- Public Client account DTOs expose only client-safe current-user/session
  information, not password hashes, policy internals, raw role mutation details,
  or bootstrap token state.
- OpenAPI and generated SDK surfaces expose the stable login/me/logout contract.

## In Scope

- Credential/session domain records and repository contracts.
- SQLite/PostgreSQL baseline schema updates for local credentials and sessions.
- Password hash and session token hash handling.
- Admin API local password set/rotate/delete contract for existing users.
- Public Client API login, current account, and logout contract.
- Auth middleware session resolution and principal insertion.
- Focused server, database, API, and SDK tests.

## Out Of Scope

- Public self-registration by default.
- Invitation redemption and onboarding flows.
- Admin Web or Media Web UI implementation.
- Desktop Tauri playback or native hardware decode.
- Native mobile UI.
- OAuth/OIDC, LDAP, SSO, passkeys, WebAuthn, or password reset email.
- Recommendation, social, or cloud account behavior.
- Refresh-token rotation unless the short-lived session MVP proves insufficient.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing local users and Library Access are the correct identity base. | High | `identity-and-library-access-contract` is complete and closed. | Reopen only if the identity contract itself is wrong. |
| The first client session should be usable as `Authorization: Bearer <token>`. | High | ADR 0024 and all generated clients already model Bearer auth. | Cookie/session auth can be layered later without replacing the backend session authority. |
| Public registration should stay disabled. | High | Prior identity lane explicitly deferred public registration. | Add a separate onboarding/invitation workstream before exposing registration. |
| Baseline migration consolidation remains acceptable. | Medium | User confirmed Nako has no production users yet; current DB uses one baseline per backend. | If production compatibility appears, stop rewriting baselines and add forward-only migrations. |
| Password hashing belongs in server application logic, not database adapters. | High | Repository adapters should persist records and avoid crypto policy ownership. | Move to a dedicated auth crate only if reuse across binaries becomes real. |

## Architecture Direction

The accepted shape is a narrow credential/session authority that plugs into the
existing identity and inbound auth boundary:

1. Admin API provisions a local password credential for an existing `User`.
2. Public Client login accepts username/password and returns an opaque session
   token plus a client-safe account summary.
3. The server stores only password hashes and session token hashes.
4. Auth middleware resolves a Bearer token in this order:
   - configured bootstrap admin token;
   - active local session token.
5. Session resolution loads the user, status, and roles, then inserts
   `AuthenticatedPrincipal` into request extensions.
6. Public Client route handlers continue using the same principal and effective
   Library Access checks as existing browse/playback/user-state routes.

The first slice intentionally does not choose cookie defaults. Browser media
transport already uses short-lived playback tickets, and JSON clients can carry
Bearer tokens today. A future frontend or reverse-proxy lane can add same-site
httpOnly cookies on top of this authority if that improves UX and CSRF posture.

## Security Requirements

- Never persist plaintext passwords or raw session tokens.
- Do not log password input, password hashes, raw session tokens, or token
  hashes.
- Use a slow password hash suitable for local accounts.
- Use high-entropy opaque session tokens.
- Treat session token lookup as credential verification, not as a user id.
- Return a generic unauthorized error for failed login attempts.
- Reject session auth for disabled users.
- Keep the bootstrap admin token path explicit for first setup and automation.
- Keep Admin API account mutation separate from Public Client account/session
  routes.

## Closeout Condition

This lane can close when:

- credential/session storage exists in both SQLite and PostgreSQL baselines;
- admin password provisioning, login, current account, logout, and session auth
  behavior are implemented and tested;
- OpenAPI and generated SDKs are current;
- docs and evidence reflect the shipped behavior;
- follow-ons for UI, invitations, cookies, account recovery, and SSO are either
  split or explicitly deferred.
