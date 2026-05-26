# Credential Session Auth - TODO

Status: Complete
Last updated: 2026-05-26

## M0 - Workstream Open

- [x] CSA-000 [owner=planner] [deps=none] [scope=docs/workstreams/credential-session-auth,docs/adr]
  Goal: Open the credential/session auth workstream and freeze the first backend-only contract direction.
  Validation: Workstream docs exist, `WORKSTREAM.json` is valid JSON, and ADR 0037 records the auth/session direction.
  Evidence: `DESIGN.md`; `docs/adr/0037-local-credential-and-session-auth.md`.
  Handoff: First executable implementation task is CSA-010.

## M1 - Credential And Session Storage Contract

- [x] CSA-010 [owner=codex] [deps=CSA-000] [scope=crates/nako-core,crates/nako-db,docs/workstreams]
  Goal: Add local password credential and session records, repository methods, and SQLite/PostgreSQL baseline schema support.
  Validation: `cargo nextest run -p nako-core identity --no-fail-fast`; focused `cargo nextest run -p nako-db credential_session --no-fail-fast`; `cargo fmt --all -- --check`.
  Review: review-workstream for persistence contract and migration-baseline safety before accepting completion.
  Evidence: `crates/nako-core/src/identity.rs`; `crates/nako-core/src/repository/identity.rs`; `crates/nako-db/src/sqlite/identity.rs`; `crates/nako-db/src/postgres/identity.rs`; `crates/nako-db/migrations/baseline.sql`; `crates/nako-db/migrations/postgres/baseline.sql`; `cargo nextest run -p nako-core -E 'test(identity)' --no-fail-fast`; `cargo nextest run -p nako-db -E 'test(credential_session)' --no-fail-fast`.
  Handoff: Keep raw password/session token material out of repository records.

## M2 - Admin Credential Provisioning

- [x] CSA-020 [owner=codex] [deps=CSA-010] [scope=crates/nako-api,crates/nako-server,docs/api]
  Goal: Add Admin API route(s) for setting, rotating, and deleting a local password credential for an existing user.
  Validation: focused `cargo nextest run -p nako-server admin_local_password --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
  Review: review-workstream for Admin/Public boundary leakage and redaction.
  Evidence: `crates/nako-api/src/admin/access.rs`; `crates/nako-server/src/http/admin.rs`; `apps/admin-web/src/adminApi/generated/contract.ts`; `cargo nextest run -p nako-server -E 'test(admin_v1_access_management)' --no-fail-fast`; `cargo nextest run -p nako-api -E 'test(admin_contract)' --no-fail-fast`.
  Handoff: Do not add public registration or frontend controls in this task.

## M3 - Public Login And Session Principal

- [x] CSA-030 [owner=codex] [deps=CSA-010,CSA-020] [scope=crates/nako-api,crates/nako-client-protocol,crates/nako-client,crates/nako-server]
  Goal: Add Public Client login/current-account/logout contracts and make active session Bearer tokens resolve to `AuthenticatedPrincipal`.
  Validation: focused `cargo nextest run -p nako-server local_session_auth --no-fail-fast`; `cargo nextest run -p nako-api public_openapi --no-fail-fast`; `cargo nextest run -p nako-client account --no-fail-fast`.
  Review: review-workstream for auth boundary behavior, session redaction, and generated SDK compatibility.
  Evidence: `crates/nako-server/src/http/auth.rs`; `crates/nako-server/src/http/account.rs`; `crates/nako-client-protocol`; `crates/nako-client`; `sdk/typescript/src/index.ts`; `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`; `cargo nextest run -p nako-server -E 'test(local_session_auth) | test(bearer_auth)' --no-fail-fast`; `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk)' --no-fail-fast`; `cargo nextest run -p nako-client -E 'test(account)' --no-fail-fast`.
  Handoff: Existing bootstrap admin token must keep working for setup and automation.

## M4 - Verification And Closeout

- [x] CSA-040 [owner=codex] [deps=CSA-020,CSA-030] [scope=docs/workstreams/credential-session-auth]
  Goal: Run fresh focused gates, record evidence, close or split follow-ons.
  Validation: `cargo fmt --all -- --check`; focused DB/server/API/client gates from this ledger; `python -m json.tool docs/workstreams/credential-session-auth/WORKSTREAM.json`; `git diff --check`.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`; `WORKSTREAM.json`; `HANDOFF.md`; `python -m json.tool docs/workstreams/credential-session-auth/WORKSTREAM.json`; `git diff --check`; `cargo fmt --all -- --check`.
  Handoff: Split UI, cookie, invitation, and SSO work instead of expanding this lane silently.
