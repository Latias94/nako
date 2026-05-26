# Identity And Library Access Contract - Milestones

Status: Active
Last updated: 2026-05-26

## M0 - Workstream Open

Exit criteria:

- Workstream docs exist.
- Migration consolidation is explicitly documented as allowed only because
  Nako has no production user/database compatibility burden.
- First executable task is clear.

## M1 - Baseline Schema Accepted

Exit criteria:

- User, role, Library Access, and effective-access records are defined.
- SQLite and PostgreSQL baseline migration strategy is accepted.
- Deletion/replacement criteria for old migrations are documented.
- Contract-test targets are identified before migration files are rewritten.

Status: Complete for ILA-010. Runtime migrators use one baseline per backend,
identity/access domain records and repository skeletons exist, and old numbered
SQLite/PostgreSQL migration files have been removed after baseline generation.

## M2 - Persistence And Principal Resolution

Exit criteria:

- Repository contracts exist and pass for SQLite.
- PostgreSQL support is implemented or explicitly deferred behind the existing
  optional Postgres test policy.
- Auth principal resolution maps credentials to stable user principals.
- User Playback State remains stable and does not store token values.

Status: Complete for ILA-020. SQLite and PostgreSQL implement
`IdentityAccessRepository`; backend-neutral contract coverage exercises users,
role replacement, Library Access policy upsert/delete/list, and effective
access. Startup creates a deterministic bootstrap administrator user for
`local-admin`, and inbound bearer auth inserts both the stable principal id and
an authenticated bootstrap administrator principal without storing token
material.

## M3 - Admin Contract

Exit criteria:

- Admin API can safely report users, roles, Library Access, and readiness.
- Mutations use explicit request models and validation.
- Responses are redaction-safe and do not expose credentials or token values.

Status: Complete for the first ILA-030 slice. Admin API now has explicit
contracts for listing/creating local user records, replacing roles, updating
user status, and listing/upserting/deleting Library Access policies. These
contracts intentionally do not create password credentials, sessions,
invitations, or public registration.

## M4 - Public Client Enforcement

Exit criteria:

- Public browse/playback/user-state routes respect effective Library Access.
- Public DTOs expose only client-safe access summaries when needed.
- Public SDK/OpenAPI inventories remain free of admin-only policy internals.

Status: Complete for ILA-040. Public library/catalog/source/image/playback
and User Playback State handlers resolve effective Library Access from the
authenticated principal. Browse routes filter inaccessible libraries/items,
playback and User Playback State write routes require `play`, legacy public
library management commands require `manage`, and Continue Watching filters
returned items by current access. Public DTOs were not expanded with Admin
policy internals.

## M5 - Closeout

Exit criteria:

- Fresh verification evidence is recorded.
- Admin Web account UI, Media Web login, invitation flow, and Management
  Context Links are split or explicitly deferred.
