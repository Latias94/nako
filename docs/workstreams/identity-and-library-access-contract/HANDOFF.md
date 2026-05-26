# Identity And Library Access Contract - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

ILA-030 is complete for the first Admin API access-management slice. The user
explicitly accepted fearless refactoring and database migration consolidation
because Nako currently has no users. The runtime migrators use one baseline
migration per backend:

- `crates/nako-db/migrations/baseline.sql`
- `crates/nako-db/migrations/postgres/baseline.sql`

The baseline includes the current schema plus identity/access tables for
`users`, `user_role_assignments`, `user_library_access_policies`, and
`role_library_access_policies`. Core domain records exist in `nako-core`, and
SQLite/PostgreSQL now implement `IdentityAccessRepository`.

Old numbered migration files have been removed. The baseline files keep
source-file comments for auditability.

Startup now creates a deterministic bootstrap administrator user for the stable
`local-admin` principal if it does not already exist, and ensures that user has
the `administrator` Role. The inbound bearer-token middleware still accepts the
existing configured token, but it inserts both `UserPrincipalId::local_admin()`
and an `AuthenticatedPrincipal` for the bootstrap administrator. Raw bearer
tokens are not stored as user ids or credentials.

Admin API now exposes redaction-safe access-management contracts:

- `GET/POST /admin/v1/access/users`
- `PUT /admin/v1/access/users/{user_id}/roles`
- `PATCH /admin/v1/access/users/{user_id}/status`
- `GET/PUT/DELETE /admin/v1/access/library-policies`

These routes manage user records, Role assignments, and Library Access policy
rows only. They do not create password credentials, sessions, invitation
tokens, OAuth/OIDC links, or public registration.

## Next Task

ILA-040: apply effective Library Access to Public Client API browse/playback
and user-state flows.

Suggested first steps:

1. Identify public library/item/source/playback route query points that must
   check effective Library Access.
2. Add tests proving a viewer cannot browse or play unassigned Media Libraries.
3. Keep Admin policy rows out of public DTOs; expose only client-safe effective
   access summaries if needed.
4. Keep User Playback State stable by principal while enforcing item/library
   visibility.

## Important Constraints

- Do not store bearer token values as user ids.
- Do not expose Admin policy internals through Public Client API.
- Do not build Admin Web user CRUD before Admin API mutation contracts exist.
- Do not expose Admin Web edit controls until credential/login UX and lockout
  behavior are accepted.
- Do not add public registration by default.
- If production database compatibility becomes required, stop baseline
  consolidation and switch to forward-only migrations.

## Follow-Ons

- Admin Web Users & Access management UI.
- Media Web login and account switching.
- Invitation-based onboarding.
- Management Context Links implementation.
- OAuth/OIDC/LDAP/passkeys after local account mode is stable.
