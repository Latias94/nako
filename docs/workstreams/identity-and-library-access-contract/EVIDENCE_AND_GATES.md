# Identity And Library Access Contract - Evidence And Gates

Status: Draft
Last updated: 2026-05-26

## Gate Policy

This lane will touch schema, auth, API, and possibly generated contracts. Gates
must scale with each task.

Minimum planning gates:

- `python -m json.tool docs/workstreams/identity-and-library-access-contract/WORKSTREAM.json`
- `git diff --check -- docs/workstreams/identity-and-library-access-contract docs/workstreams/README.md`

Expected implementation gates:

- `cargo fmt --all -- --check`
- focused `cargo nextest run -p nako-db <filter> --no-fail-fast`
- focused `cargo nextest run -p nako-server <filter> --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` when Admin API changes
- public OpenAPI/SDK leakage checks when public protocol changes
- optional PostgreSQL contract tests when `NAKO_TEST_POSTGRES_URL` is available
- Admin Web generate/check/test/build only when generated Admin Web contract or UI is touched

## Migration Consolidation Gate

Before replacing or deleting old migration files:

- confirm there is still no production compatibility requirement;
- record the old and new migration inventory;
- prove an empty SQLite database migrates to the baseline schema;
- prove critical repository contract tests pass on the new schema;
- run `git diff --check`;
- manually inspect the diff to ensure unrelated user changes were not removed.

If any compatibility requirement appears, use forward-only migrations instead.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | ILA-000 workstream open | Read ADR 0024, 0027, 0028, 0029, 0030; inspected SQLite and PostgreSQL migration inventories and migrator code; opened docs-first identity/access execution lane. | Draft lane opened. |
| 2026-05-26 | ILA-010 domain/schema baseline | Added `UserId`, `User`, `UserRole`, `RoleAssignment`, `LibraryAccessPolicy`, `EffectiveLibraryAccess`, and `IdentityAccessRepository`; generated SQLite/PostgreSQL baseline SQL files; rewired SQLite/PostgreSQL migrators to one baseline migration each; removed old numbered SQLite/PostgreSQL SQL files after baseline generation. | `cargo nextest run -p nako-core identity --no-fail-fast` passed; `cargo nextest run -p nako-db baseline_migration --no-fail-fast` passed; `cargo nextest run -p nako-db migration --no-fail-fast` passed; `cargo nextest run -p nako-db user_playback --no-fail-fast` passed; `cargo fmt --all -- --check` passed. |

## Verification Notes

- ILA-010 did not change `nako-server` auth/principal resolution, so the
  server auth gate is deferred to ILA-020.
- Public API/OpenAPI/Admin Web gates were not run because ILA-010 did not
  change API DTOs, generated contracts, or UI code.
- PostgreSQL runtime integration was not run because this environment does not
  provide `NAKO_TEST_POSTGRES_URL`; the local Postgres baseline test verifies
  the registered migration inventory and identity/access SQL presence.

## Redaction Rules

Identity/access routes and tests must not expose:

- bearer token values;
- password hashes;
- password reset or invitation token values;
- credential secret references when avoidable;
- raw request headers;
- local filesystem paths;
- raw Source Locators;
- provider payloads;
- addon tokens;
- webhook secrets.
