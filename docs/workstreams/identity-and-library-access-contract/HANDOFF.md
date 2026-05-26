# Identity And Library Access Contract - Handoff

Status: Draft
Last updated: 2026-05-26

## Current State

ILA-010 is complete. The user explicitly accepted fearless refactoring and
database migration consolidation because Nako currently has no users. The
runtime migrators now use one baseline migration per backend:

- `crates/nako-db/migrations/baseline.sql`
- `crates/nako-db/migrations/postgres/baseline.sql`

The baseline includes the current schema plus identity/access tables for
`users`, `user_role_assignments`, `user_library_access_policies`, and
`role_library_access_policies`. Core domain records and repository skeletons
exist in `nako-core`.

Old numbered migration files have been removed. The baseline files keep
source-file comments for auditability.

## Next Task

ILA-020: implement repository adapters and principal resolution.

Suggested first steps:

1. Implement `IdentityAccessRepository` for SQLite and PostgreSQL adapters.
2. Add backend-neutral contract tests for users, roles, Library Access policies,
   and effective access resolution.
3. Decide whether Single-Admin Mode maps to a reserved `local-admin` principal
   or creates a deterministic bootstrap administrator user.
4. Update auth/principal resolution without storing bearer token values as user
   ids.
5. Keep User Playback State stable by principal and prove existing playback
   state tests still pass.

## Important Constraints

- Do not store bearer token values as user ids.
- Do not expose Admin policy internals through Public Client API.
- Do not build Admin Web user CRUD before backend authority exists.
- Do not add public registration by default.
- If production database compatibility becomes required, stop baseline
  consolidation and switch to forward-only migrations.

## Follow-Ons

- Admin Web Users & Access management UI.
- Media Web login and account switching.
- Invitation-based onboarding.
- Management Context Links implementation.
- OAuth/OIDC/LDAP/passkeys after local account mode is stable.
