# Identity And Library Access Contract - Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

ILA-040 is complete for the first Public Client API effective-access slice. The
user explicitly accepted fearless refactoring and database migration consolidation
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

Public Client API library/catalog/source/image/playback/User Playback State
routes now enforce effective Library Access from `AuthenticatedPrincipal`:

- `browse` can list/view client-safe library and catalog data.
- `play` is required for playback decisions, streams, remux/HLS,
  playback-session lookup/cancel, and User Playback State writes.
- `manage` is required for legacy public library management commands outside
  `/admin/v1`.
- Continue Watching stays principal-scoped and filters returned items by
  current access.
- Public DTOs do not expose Admin policy rows, Role assignments, policy
  reasons, credentials, or account internals.

## Next Task

This lane is closed. Start a new focused workstream for the next product slice.

Suggested first steps:

1. Split follow-ons for Admin Web account UI, Media Web login/account
   switching, invitation onboarding, and Management Context Links.
2. Keep credential/session/login UX separate from this persistence and access
   enforcement lane.
3. Do not reopen this lane unless the identity/access contract itself is found
   to be wrong.

## Important Constraints

- Do not store bearer token values as user ids.
- Do not expose Admin policy internals through Public Client API.
- Do not build Admin Web user CRUD before Admin API mutation contracts exist.
- Do not expose Admin Web edit controls until credential/login UX and lockout
  behavior are accepted.
- Do not add public registration by default.
- Do not expose Admin policy internals through Public Client DTOs.
- If production database compatibility becomes required, stop baseline
  consolidation and switch to forward-only migrations.

## Follow-Ons

- Admin Web Users & Access management UI.
- Media Web login and account switching.
- Invitation-based onboarding.
- Management Context Links implementation.
- OAuth/OIDC/LDAP/passkeys after local account mode is stable.

## Recommended Next Goal

Open a Media Web foundation lane that consumes Public Client API browse,
playback, and User Playback State routes with login/account switching. If
admin-created local users need a browser surface first, split a smaller Admin
Web account UI lane before Media Web.
