# Identity And Library Access Contract - TODO

Status: Draft
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

- [ ] ILA-020 [owner=unassigned] [deps=ILA-010] [scope=crates/nako-core,crates/nako-db,crates/nako-server]
  Goal: Implement repository contracts and principal resolution for local users, roles, Library Access, and bootstrap administrator semantics.
  Validation: focused `cargo nextest run -p nako-db user --no-fail-fast`; focused `cargo nextest run -p nako-server auth --no-fail-fast`; `cargo fmt --all -- --check`.
  Review: review-workstream for persistence contracts and auth boundary.
  Evidence: Repository contract tests and auth/principal tests.
  Handoff: Do not store raw bearer tokens or password material in user ids.

## M3 - Admin API Access Management Contract

- [ ] ILA-030 [owner=unassigned] [deps=ILA-020] [scope=crates/nako-api,crates/nako-server,docs/api]
  Goal: Add redaction-safe Admin API read/mutation contracts for users, role assignments, Library Access policies, and bootstrap readiness.
  Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`; focused `cargo nextest run -p nako-server admin_access --no-fail-fast`; generated Admin contract parity if touched.
  Review: review-workstream for Admin/Public boundary leakage.
  Evidence: HTTP API docs and Admin contract tests.
  Handoff: Admin Web UI remains a follow-on until these contracts are stable.

## M4 - Public Client Effective Access

- [ ] ILA-040 [owner=unassigned] [deps=ILA-020] [scope=crates/nako-api,crates/nako-server,crates/nako-client-protocol]
  Goal: Apply effective Library Access to public browse/playback/user-state flows and expose only client-safe current-user/access summaries if needed.
  Validation: focused Public Client API route tests; generated public SDK/OpenAPI leakage checks if DTOs change.
  Review: review-workstream for protocol compatibility.
  Evidence: Public route tests and SDK/OpenAPI checks.
  Handoff: Do not expose Admin policy rows through public DTOs.

## M5 - Closeout And Follow-Ons

- [ ] ILA-050 [owner=planner] [deps=ILA-030,ILA-040] [scope=docs/workstreams/identity-and-library-access-contract,docs/workstreams/client-surface-and-access-product-architecture]
  Goal: Close the lane or split Admin Web account UI, Media Web login, invitations, and context-link implementation.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md.
  Handoff: Recommend the next implementation goal.
