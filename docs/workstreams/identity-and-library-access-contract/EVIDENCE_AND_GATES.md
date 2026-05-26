# Identity And Library Access Contract - Evidence And Gates

Status: Active
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
| 2026-05-26 | ILA-020 repository/principal | Implemented SQLite/PostgreSQL `IdentityAccessRepository` adapters, facade dispatch, backend-neutral identity/access contract coverage, deterministic bootstrap administrator startup, and inbound auth `AuthenticatedPrincipal` insertion. Updated Admin access summary readiness/copy to reflect backend identity storage while keeping mutations hidden. | `cargo nextest run -p nako-core identity --no-fail-fast` passed; `cargo nextest run -p nako-db user --no-fail-fast` passed; `cargo nextest run -p nako-server auth --no-fail-fast` passed; `cargo nextest run -p nako-server app_startup_creates_deterministic_bootstrap_admin_user admin_v1_access_summary --no-fail-fast` passed; `cargo fmt --all -- --check` passed; `python -m json.tool docs\workstreams\identity-and-library-access-contract\WORKSTREAM.json > $null` passed; `npm run test -- App.test.tsx -t "Users & Access"` passed; `git diff --check` passed. |
| 2026-05-26 | ILA-030 Admin API access management | Added explicit Admin API DTOs, generated contract route constants/types, server handlers, validation, and system tests for local users, Role assignments, user status, and Library Access policy rows. Updated Admin Web copy to keep edit controls hidden until credential/login/lockout UX is accepted. | `cargo nextest run -p nako-api admin_contract_includes_route_constants --no-fail-fast` passed; `cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output --no-fail-fast` passed; `cargo nextest run -p nako-server admin_v1_access_management admin_v1_access_summary --no-fail-fast` passed; `npm run generate:admin-api` passed; `npm run check` passed; `cargo fmt --all` and `cargo fmt --all -- --check` passed; `python -m json.tool docs\workstreams\identity-and-library-access-contract\WORKSTREAM.json > $null` passed; `git diff --check` passed with line-ending warnings only. |
| 2026-05-26 | Goal verification for ILA-020/ILA-030 | Re-ran focused gates covering bootstrap identity semantics, identity/access repository contracts, auth principal resolution, deterministic startup, Admin API generated contracts, Admin API access-management round trips, Admin Web typecheck, workstream JSON, formatting, and whitespace checks. | `cargo nextest run -p nako-core identity --no-fail-fast` passed; `cargo nextest run -p nako-db user --no-fail-fast` passed; `cargo nextest run -p nako-server auth app_startup_creates_deterministic_bootstrap_admin_user --no-fail-fast` passed; `cargo nextest run -p nako-api admin_contract_includes_route_constants --no-fail-fast` passed; `cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output --no-fail-fast` passed; `cargo nextest run -p nako-server admin_v1_access_management admin_v1_access_summary --no-fail-fast` passed; `npm run check` in `apps/admin-web` passed. |
| 2026-05-26 | ILA-040 Public Client effective access | Added an HTTP access gate that resolves effective Library Access from `AuthenticatedPrincipal`; applied it to Public Client library, catalog, source probe, selected image, playback, playback-session, and User Playback State routes. Added route tests for filtered browse, play-only playback, User Playback State write denial, and Continue Watching filtering. | `cargo nextest run -p nako-server public_browse_routes_filter_libraries_and_items_by_effective_access playback_routes_require_play_library_access user_playback_write_routes_require_play_library_access continue_watching_filters_items_without_current_library_access --no-fail-fast` passed; `cargo nextest run -p nako-server catalog playback user_playback bearer_auth --no-fail-fast` passed; `cargo fmt --all -- --check` passed; `python -m json.tool docs\workstreams\identity-and-library-access-contract\WORKSTREAM.json > $null` passed; `git diff --check` passed with line-ending warnings only. |

## Verification Notes

- ILA-030 added generated Admin Web route constants/types for Admin-only access
  management routes. Public Client OpenAPI/SDK artifacts were not changed.
- Admin Web mutation controls remain hidden after ILA-030 because password,
  login, invitation, and lockout UX are still unaccepted follow-ons.
- ILA-040 did not change `nako-client-protocol` DTOs, generated Public Client
  SDKs, or OpenAPI schemas; enforcement is server-side filtering/authorization
  against existing public response shapes.
- Focused clippy was attempted with
  `cargo clippy -p nako-core -p nako-db -p nako-api -p nako-server --all-targets -- -D warnings -A clippy::double_must_use -A clippy::derivable_impls`.
  After fixing the identity/access findings, clippy remains blocked by
  existing unrelated warnings in crates such as `nako-catalog`,
  `nako-transcode`, `nako-vfs`, and pre-existing `nako-db` modules/tests.
  The reported failures are outside the new identity/access files and should
  be handled by a separate lint-cleanup lane.
- PostgreSQL runtime integration was not run because this environment does not
  provide `NAKO_TEST_POSTGRES_URL`; the local Postgres baseline test verifies
  the registered migration inventory and identity/access SQL presence, and the
  PostgreSQL adapter compiles through the ignored backend-neutral contract.

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
