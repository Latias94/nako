# Admin API TypeScript Contract TODO

Status: Active
Last updated: 2026-05-19

## AATC.0 Scope And Contract Decision

- [x] AATC-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-api-typescript-contract, docs/workstreams/admin-web-console]
  Goal: Open the workstream, close the completed admin-web baseline, and
  freeze the Admin API TypeScript contract problem statement.
  Validation: workstream docs exist and `git diff --check` passes.
  Evidence: this workstream and `docs/workstreams/admin-web-console`.
  Handoff: Continue with AATC-020 before editing Rust generator code.

- [x] AATC-020 [owner=codex] [deps=AATC-010] [scope=crates/taru-api, apps/admin-web/src/adminApi, docs/workstreams/admin-api-typescript-contract]
  Goal: Audit current hand-written admin-web wire DTOs and choose the exact
  first artifact shape: generated interfaces only, generated route constants,
  or generated tiny client.
  Validation: documented decision plus route/DTO inventory; no code generation
  behavior changes yet unless the audit exposes a trivial test-only guard.
  Review: review-workstream before accepting implementation tasks.
  Evidence: `ADMIN_CONTRACT_INVENTORY.md`, updated `DESIGN.md`, and updated
  `HANDOFF.md`.
  Handoff: AATC-030 should generate app-local route constants, wire
  interfaces, and query interfaces under
  `apps/admin-web/src/adminApi/generated/contract.ts`, while keeping
  `client.ts` as the hand-written fetch/runtime boundary.

## AATC.1 Generator Proof

- [ ] AATC-030 [owner=codex] [deps=AATC-020] [scope=crates/taru-api/src, crates/taru-api/examples, apps/admin-web/src/adminApi]
  Goal: Implement the first generated or mechanically synchronized Admin API
  TypeScript contract for the AWC-070 read-model routes.
  Validation: `cargo check -p taru-api --examples`, focused `taru-api`
  generator tests, and generated artifact sync check.
  Review: review-workstream for public/admin boundary leakage.
  Evidence: generator source, example command, generated app-local artifact,
  and Rust tests.
  Handoff: Do not wire broader UI routes until sync and leakage tests pass.

- [ ] AATC-040 [owner=codex] [deps=AATC-030] [scope=apps/admin-web/src/adminApi, apps/admin-web/src]
  Goal: Make admin-web consume the generated Admin API contract for covered
  wire DTOs while keeping UI-only view models local.
  Validation: `cd apps/admin-web && npm run check`, `npm run test`, and
  `npm run build`.
  Review: review-workstream for DTO duplication and redaction fixture safety.
  Evidence: admin-web imports generated contract; tests keep source fallback
  and redaction behavior.
  Handoff: Jobs/Catalog/Playback detail pages can start after this task.

## AATC.2 Boundary And Closeout

- [ ] AATC-050 [owner=codex] [deps=AATC-040] [scope=crates/taru-api, sdk/typescript, docs/api, docs/workstreams/admin-api-typescript-contract]
  Goal: Add or update public/admin separation tests and docs so Public Client
  SDK generation continues to reject admin routes while admin contract covers
  only `/admin/v1/*`.
  Validation: focused `taru-api` nextest tests, public TypeScript SDK sync
  test, admin-web gates, and `git diff --check`.
  Review: review-workstream and verify-rust-workstream before closeout.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: Split npm packaging or deeper admin UI workflows as follow-ons.
