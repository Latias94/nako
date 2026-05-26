# Admin Web V2 Users Access Readiness - Design

Status: Complete
Last updated: 2026-05-26

## Problem

The broader Admin Web V2 goal calls out users, permissions, and Library Access,
but Nako currently has only an inbound bearer-token boundary and Single-Admin
Mode. User Playback State already persists through a stable `local-admin`
principal, but there is no account store, role assignment store, or per-library
access policy model.

Admin Web needs a real operator surface for this boundary. It must show what is
currently true and what is planned, without creating fake user management
controls that cannot be backed by Admin API authority.

## Target State

- Admin API exposes a redaction-safe `GET /admin/v1/access/summary` route.
- The route reports:
  - active access mode;
  - current stable principal;
  - inbound auth status without token values or env var names;
  - role/account/Library Access policy readiness;
  - the effective Library Access for configured Media Libraries in
    Single-Admin Mode.
- Admin Web adds a route-owned Users & Access page using `AdminDataSource`.
- The page renders live/mock state, readiness, current principal, and
  per-library effective access.
- The page does not offer account CRUD, role mutation, or per-library access
  mutation until backend authority exists.

## Scope

- `nako-api` Admin DTOs and generated Admin Web contract.
- `nako-server` Admin route and tests.
- `apps/admin-web` route, client, data source, mock data, tests, and styling.
- HTTP API docs and this workstream's evidence.

## Non-Goals

- User account persistence.
- Password, OAuth/OIDC, LDAP, passkey, or session management.
- Role assignment mutation.
- Per-user or per-role Library Access policy storage.
- Public Client API changes.
- Playback-client account switching.

## Architecture Direction

The first slice is a readiness and effective-access diagnostic. It preserves
the domain language from `CONTEXT.md`: User, Role, Library Access, and
Single-Admin Mode.

Single-Admin Mode is represented as one stable local principal with full access
to all configured Media Libraries. Role and Library Access policy readiness are
explicitly marked as planned, not active. This keeps future account work as a
bounded data-model extension instead of forcing a migration away from a fake
Admin Web-only model.

## Redaction Rules

The route and UI must not expose:

- bearer token values;
- auth token env var names;
- local paths, source URIs, root refs, hosts, URLs, or database URLs;
- provider secrets, addon tokens, webhook secrets, or raw request headers.

Media Library ids, names, presets, backend kinds, and effective access levels
are safe to display.
