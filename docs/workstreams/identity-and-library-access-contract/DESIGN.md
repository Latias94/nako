# Identity And Library Access Contract

Status: Active
Last updated: 2026-05-26

## Why This Lane Exists

Nako currently has a safe inbound bearer-token boundary and truthful
Single-Admin Mode, but it does not yet have real users, account persistence,
roles, or Library Access storage. Media Web, desktop playback clients,
family/small-group usage, and permission-gated media-to-admin links all need a
durable identity model before UI can grow responsibly.

The project is still pre-user and pre-production. That changes the right
database strategy: instead of adding more small migrations on top of the
current long chain, this lane may consolidate backend migrations into a new
baseline while preserving repository behavior through tests.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `docs/adr/0024-inbound-token-authentication-boundary.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- `docs/adr/0029-postgresql-ready-persistence-boundary.md`
- `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
- `docs/workstreams/admin-web-v2-users-access-readiness/`
- `docs/workstreams/client-surface-and-access-product-architecture/`
- `docs/workstreams/public-client-api/`
- `docs/workstreams/user-playback-state-contract/`
- `crates/nako-core`
- `crates/nako-db`
- `crates/nako-api`
- `crates/nako-server`

## Problem

Current access state is deliberately shallow:

- all authenticated non-health requests use one configured bearer-token
  boundary;
- Single-Admin Mode resolves to the stable `local-admin` principal;
- User Playback State already stores by principal, but the principal is not a
  real user row;
- Admin Web can show readiness, but cannot create users, assign roles, or
  configure Library Access;
- Public Client API cannot yet expose user-specific browse/play behavior beyond
  the current principal;
- Media Web cannot safely decide who may see admin context links;
- database migrations are now numerous, and adding identity on top of that
  chain preserves compatibility weight that Nako does not currently need.

## Target State

When this lane closes:

- Nako has a durable local user model.
- Nako has coarse role assignment for `administrator`, `library_manager`, and
  `viewer`.
- Nako has Library Access policy storage for users or role-derived effective
  access.
- Single-Admin Mode is either preserved as a bootstrap compatibility mode or
  migrated into a real bootstrap administrator user/principal.
- User Playback State still resolves through a stable principal and does not
  store token values.
- Admin API can report and, when accepted, mutate users/roles/Library Access
  through redaction-safe DTOs.
- Public Client API browse/playback/user-state behavior can resolve effective
  Library Access without learning admin internals.
- SQLite and PostgreSQL migrations are consolidated into backend-owned
  baseline migrations where safe, with schema behavior proven by contract
  tests.

## In Scope

- Local user/account domain records and IDs.
- Role vocabulary and assignment model.
- Library Access storage and effective-access rules.
- Bootstrap administrator semantics.
- Authentication principal resolution changes needed to map credentials to
  users.
- Repository traits and SQLite/PostgreSQL adapters.
- Backend-owned migration baseline consolidation for SQLite and PostgreSQL.
- Admin API read/mutation contract for account and access management when
  backend authority is ready.
- Public Client API authorization hooks needed for browse/playback decisions,
  without exposing admin DTOs.
- Focused Rust tests and docs.

## Out Of Scope

- Admin Web account CRUD UI.
- Media Web login or account switching UI.
- Open public registration.
- Email delivery, password reset, invitation emails, OAuth/OIDC, LDAP,
  passkeys, SSO, or external identity provider integration.
- Fine-grained field-level permissions.
- Parental controls, content rating restrictions, sharing links, guest links,
  or per-device policy.
- Migrating existing real user data. There is no supported production user
  base yet.
- Recommendation or personalization features beyond access and playback state.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Nako has no production user base to preserve. | High | User explicitly confirmed current lack of users and accepted migration consolidation. | If production data appears, stop consolidation and write forward migrations plus data migration tests. |
| Baseline migration consolidation is acceptable now. | High | ADR 0030 makes migrations backend-owned, and Nako is pre-compatibility. | Keep old migrations and add identity incrementally if release compatibility becomes required. |
| Coarse roles plus Library Access are enough for the first multi-user slice. | High | Product direction targets self-hosted family/small-group usage first. | Split finer permissions only after a concrete workflow requires them. |
| Public registration should stay disabled by default. | High | Nako is private self-hosted software with auth enabled by default. | Open registration requires abuse/rate-limit/invite/email/recovery design first. |
| User Playback State should continue to use stable principals, not tokens. | High | ADR 0028 already accepted stable principal storage. | Update ADR 0028 only if the new user model changes principal semantics. |
| Public clients need effective access, not admin policy internals. | High | ADR 0027 separates Admin API from Public Client API. | Add public capability/readiness summaries rather than leaking Admin DTOs. |

## Architecture Direction

### Identity Model

Introduce local users as first-class domain records:

```text
User
  id
  username or display name
  status: active | disabled
  created_at
  updated_at

UserCredential
  user_id
  credential kind
  password hash or token reference, once accepted
  created_at
  rotated_at

RoleAssignment
  user_id
  role: administrator | library_manager | viewer

LibraryAccessPolicy
  principal scope: user or role
  library_id
  access: none | browse | play | manage
```

Do not store bearer token values as user ids. If the first credential slice
keeps the existing admin token, it should resolve to a bootstrap administrator
principal, then later credentials can resolve to concrete user ids.

### Bootstrap Semantics

Single-Admin Mode remains the safe bootstrap. The first implementation should
choose one of these explicitly:

1. keep `local-admin` as a reserved principal until an administrator creates
   the first local user; or
2. create a bootstrap administrator user during migration/startup and map the
   existing admin token to that user.

The lane should prefer option 2 if it simplifies future Library Access and
User Playback State, but only if migration/startup behavior is deterministic
and tests can prove it.

### Role And Library Access Rules

Initial access levels:

- `none`: no visibility.
- `browse`: can list and view metadata for assigned libraries.
- `play`: can browse and start playback for assigned libraries.
- `manage`: can perform library-scoped management workflows for assigned
  libraries, subject to Admin API confirmation rules.

Initial role defaults:

- `administrator`: global manage access and Admin Web access.
- `library_manager`: manage access only for assigned libraries.
- `viewer`: play access only for assigned libraries.

Effective access should be computed server-side from user assignments and
policies. Public Client API should receive only the result needed for browse,
playback, and safe capability display.

### Database Baseline Consolidation

Because Nako has no existing real users and no compatibility burden for
production databases, this lane may consolidate migration history:

- SQLite may replace the current numbered chain with a smaller baseline
  migration that creates the current schema plus identity/access tables.
- PostgreSQL may replace its current backend-local chain with a matching
  backend-owned baseline, not a textual copy of SQLite.
- Migration version numbers are allowed to restart per backend only if the
  repository has no compatibility requirement for existing databases.
- Contract tests, not textual migration parity, prove behavior.
- The old migrations should be removed only after a diff confirms no unrelated
  user work is being discarded and focused migration/repository tests pass.

If any existing database compatibility requirement appears, stop and switch to
forward-only migrations.

ILA-010 accepted the runtime baseline consolidation:

- SQLite migrator now registers one backend-owned migration:
  `crates/nako-db/migrations/baseline.sql`.
- PostgreSQL migrator now registers one backend-owned migration:
  `crates/nako-db/migrations/postgres/baseline.sql`.
- Both baselines include the current pre-existing schema plus `users`,
  `user_role_assignments`, `user_library_access_policies`, and
  `role_library_access_policies`.
- Old numbered migration files were removed after the new baselines were
  generated and focused migration/repository gates passed. The baseline files
  keep source-file comments for auditability.
- `UserPlaybackState` still stores stable `principal_id` values and has not
  been linked by foreign key to users. ILA-020 owns the deterministic mapping
  from credentials/bootstrap administrator behavior to concrete user principals.

ILA-020 accepted the first runtime identity mapping:

- SQLite and PostgreSQL implement `IdentityAccessRepository`.
- Startup creates a deterministic bootstrap administrator user for
  `local-admin` when missing and ensures that user has the `administrator`
  Role.
- The existing configured bearer token still gates inbound requests, but it
  resolves to `UserPrincipalId::local_admin()` plus an `AuthenticatedPrincipal`
  for the bootstrap administrator.
- Raw bearer token values are not stored as user ids or credential material.
- Admin access summary may report identity/Role/Library Access storage
  readiness as active. Account and policy mutation routes were deferred from
  ILA-020 and accepted in ILA-030 below.

ILA-030 accepted the first Admin API access-management contract:

- Admin API exposes redaction-safe routes for listing/creating local user
  records, replacing Role assignments, updating user status, and
  listing/upserting/deleting Library Access policies.
- These routes mutate identity and access-control records only. They do not
  create password credentials, sessions, invitation tokens, OAuth/OIDC links,
  public registration, or Public Client API DTOs.
- The bootstrap administrator user cannot be disabled and cannot lose the
  `administrator` Role through these routes.
- Admin Web generated contracts now know these routes, but edit controls remain
  hidden until credential creation, login, and account lockout UX are accepted.
- ILA-040 owns enforcement of effective Library Access on Public Client API
  browse/playback/user-state flows.

ILA-040 accepted the first Public Client API effective-access enforcement:

- Public media-library, catalog item, selected image, source probe, playback,
  and User Playback State routes now read `AuthenticatedPrincipal` and resolve
  effective Library Access before returning data or starting playback.
- `browse` access can list/view client-safe library and catalog data.
- `play` access is required for playback decisions, direct streams, remux/HLS
  entrypoints, playback-session lookup/cancel, and User Playback State writes.
- `manage` access is required for legacy public library management commands
  that remain outside `/admin/v1`.
- Continue Watching keeps the stable principal-scoped storage model but filters
  returned items by current effective Library Access.
- Public DTOs were not expanded with Admin policy rows, Role assignments,
  policy reasons, or credential/account details.

### API Boundaries

Admin API owns:

- user list/detail;
- create/disable local user;
- role assignment;
- Library Access policy read/update;
- bootstrap/access readiness;
- redaction-safe audit/readiness facts.

Public Client API owns:

- current authenticated user/profile summary if needed by clients;
- effective library visibility;
- playback and User Playback State for the resolved principal.

Public Client API must not expose Admin policy rows, password hashes, credential
references, token values, internal migration state, or raw role-management
diagnostics.

## Closeout Condition

This lane can close when:

- identity/access schema is accepted and implemented behind backend-owned
  migrations or an explicit docs-only follow-on;
- SQLite and PostgreSQL migration strategy is consolidated or deliberately
  left forward-only with rationale;
- repository contracts cover local users, roles, Library Access, and effective
  access;
- principal resolution and User Playback State semantics are updated or proven
  compatible;
- Admin API contracts expose truthful access management/readiness;
- Public Client API behavior is either unchanged with evidence or updated
  through a dedicated public contract task;
- docs, HTTP API notes, and workstream evidence are current.
