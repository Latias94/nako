# Identity And Library Access Contract - TODO

Status: Active
Last updated: 2026-05-26

## M0 - Workstream Open

- [x] ILA-000 [owner=planner] [deps=none] [scope=docs/workstreams/identity-and-library-access-contract]
  Goal: Open the identity/access workstream with migration consolidation as an accepted first-class option.
  Validation: Workstream docs exist and agree.
  Evidence: DESIGN.md
  Handoff: First executable task is ILA-010.

## M1 - Schema Baseline And Domain Contract

- [x] ILA-010 [owner=codex] [deps=ILA-000] [scope=crates/nako-core,crates/nako-db,docs/adr,docs/workstreams]
  Goal: Define User, RoleAssignment, LibraryAccessPolicy, effective access, and baseline migration consolidation plan for SQLite/PostgreSQL.
  Validation: `cargo fmt --all -- --check`; focused `cargo nextest run -p nako-db migration --no-fail-fast` or narrower available migration/schema tests; updated ADR/design if migration history is rewritten.
  Review: review-workstream before deleting or replacing migration files.
  Evidence: `crates/nako-core/src/identity.rs`; `crates/nako-core/src/repository/identity.rs`; `crates/nako-db/migrations/baseline.sql`; `crates/nako-db/migrations/postgres/baseline.sql`; old numbered SQLite/PostgreSQL migration files removed after baseline generation; `cargo nextest run -p nako-core identity --no-fail-fast`; `cargo nextest run -p nako-db migration --no-fail-fast`; `cargo nextest run -p nako-db user_playback --no-fail-fast`; `cargo fmt --all -- --check`.
  Handoff: Runtime migrations now use one baseline per backend. If any production compatibility requirement appears, switch to forward-only migrations.

## M2 - Repository And Principal Resolution

- [x] ILA-020 [owner=codex] [deps=ILA-010] [scope=crates/nako-core,crates/nako-db,crates/nako-server,docs/api,apps/admin-web/src/i18n]
  Goal: Implement repository contracts and principal resolution for local users, roles, Library Access, and bootstrap administrator semantics.
  Validation: focused `cargo nextest run -p nako-db user --no-fail-fast`; focused `cargo nextest run -p nako-server auth --no-fail-fast`; `cargo fmt --all -- --check`.
  Review: review-workstream for persistence contracts and auth boundary.
  Evidence: `crates/nako-db/src/sqlite/identity.rs`; `crates/nako-db/src/postgres/identity.rs`; `crates/nako-db/src/contract_tests.rs`; `crates/nako-server/src/app/startup.rs`; `crates/nako-server/src/http/auth.rs`; `cargo nextest run -p nako-core identity --no-fail-fast`; `cargo nextest run -p nako-db user --no-fail-fast`; `cargo nextest run -p nako-server auth --no-fail-fast`; `cargo nextest run -p nako-server app_startup_creates_deterministic_bootstrap_admin_user admin_v1_access_summary --no-fail-fast`.
  Handoff: DONE. Existing bearer-token auth now resolves to the stable `local-admin` principal plus an `AuthenticatedPrincipal` for the deterministic bootstrap administrator user. User Playback State remains principal-scoped and raw bearer tokens are not stored as user ids.

## M3 - Admin API Access Management Contract

- [x] ILA-030 [owner=codex] [deps=ILA-020] [scope=crates/nako-api,crates/nako-server,apps/admin-web,docs/api]
  Goal: Add redaction-safe Admin API read/mutation contracts for users, role assignments, Library Access policies, and bootstrap readiness.
  Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`; focused `cargo nextest run -p nako-server admin_access --no-fail-fast`; generated Admin contract parity if touched.
  Review: review-workstream for Admin/Public boundary leakage.
  Evidence: `crates/nako-api/src/admin/access.rs`; `crates/nako-api/src/admin_contract.rs`; `crates/nako-server/src/http/admin.rs`; `crates/nako-server/src/http/tests/system.rs`; `apps/admin-web/src/adminApi/generated/contract.ts`; `cargo nextest run -p nako-api admin_contract_includes_route_constants --no-fail-fast`; `cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output --no-fail-fast`; `cargo nextest run -p nako-server admin_v1_access_management admin_v1_access_summary --no-fail-fast`; `npm run generate:admin-api`; `npm run check`.
  Handoff: DONE. Admin API has backend read/write contracts for users, roles, and Library Access policies. Admin Web edit controls, credential creation, password/login, invitations, and public-client enforcement remain follow-ons.

## M4 - Public Client Effective Access

- [x] ILA-040 [owner=codex] [deps=ILA-020] [scope=crates/nako-server,docs/api,docs/workstreams]
  Goal: Apply effective Library Access to public browse/playback/user-state flows and expose only client-safe current-user/access summaries if needed.
  Validation: focused Public Client API route tests; generated public SDK/OpenAPI leakage checks if DTOs change.
  Review: review-workstream for protocol compatibility.
  Evidence: `crates/nako-server/src/http/access.rs`; `crates/nako-server/src/http/library.rs`; `crates/nako-server/src/http/catalog.rs`; `crates/nako-server/src/http/playback.rs`; `crates/nako-server/src/http/user_playback.rs`; `cargo nextest run -p nako-server public_browse_routes_filter_libraries_and_items_by_effective_access playback_routes_require_play_library_access user_playback_write_routes_require_play_library_access continue_watching_filters_items_without_current_library_access --no-fail-fast`; `cargo nextest run -p nako-server catalog playback user_playback bearer_auth --no-fail-fast`.
  Handoff: DONE. Public browse/playback/user-state routes enforce effective Library Access through the authenticated principal without exposing Admin policy rows through public DTOs. Public client protocol DTOs did not change.

## M5 - Closeout And Follow-Ons

- [ ] ILA-050 [owner=planner] [deps=ILA-030,ILA-040] [scope=docs/workstreams/identity-and-library-access-contract,docs/workstreams/client-surface-and-access-product-architecture]
  Goal: Close the lane or split Admin Web account UI, Media Web login, invitations, and context-link implementation.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md.
  Handoff: Recommend the next implementation goal.
