# Admin Web V2 Settings Mutation Authority - TODO

Status: Closed
Last updated: 2026-05-26

## M0 - Scope And Evidence Freeze

- [x] ASM-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-v2-settings-mutation-authority]
  Goal: Open the settings mutation lane, freeze the non-goals, and record the
  current diagnostic-only baseline.
  Validation: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `ROUTE_API_READINESS.md`, `HANDOFF.md`, and
  `WORKSTREAM.json` agree.
  Evidence: `docs/workstreams/admin-web-v2-settings-mutation-authority/DESIGN.md`
  Handoff: DONE. Next task is ASM-020.

## M1 - Route/API Readiness And First Slice Decision

- [x] ASM-020 [owner=codex] [deps=ASM-010] [scope=crates/nako-server/src/http/admin.rs,crates/nako-server/src/config.rs,apps/admin-web/src/features/settings,apps/admin-web/src/adminApi,docs/workstreams/admin-web-v2-settings-mutation-authority]
  Goal: Audit existing settings diagnostics, config source-of-truth behavior,
  generated Admin Web contract shape, and candidate editable field groups;
  choose the first slice or split the required backend authority task.
  Validation: `rg -n "system/config|update.*config|NetworkAccessConfig|SettingsPage|loadSettings" crates/nako-server/src crates/nako-api/src apps/admin-web/src docs/api/HTTP_API.md`; `git diff --check`
  Review: The decision must distinguish runtime-only, persisted,
  restart-required, and rejected changes. It must not authorize raw config,
  secret, URL, path, root, host, token, credential, or env var rendering.
  Evidence: `ROUTE_API_READINESS.md`
  Handoff: DONE_WITH_CONCERNS. No global settings group is safe for Admin Web
  mutation yet because the backend has no global settings authority and current
  router/runtime state is startup-copied. Split
  `docs/workstreams/admin-settings-configuration-authority/` as the required
  backend predecessor.

## M2 - First Real Mutation Slice Or Backend Split

- [x] ASM-030 [owner=codex] [deps=ASM-020,ASCA-030] [scope=crates/nako-api/src,crates/nako-server/src,docs/api/HTTP_API.md,apps/admin-web/src/adminApi]
  Goal: Consume the Admin Settings Configuration Authority backend route and
  update this lane's generated-contract/API readiness evidence.
  Validation: `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`; `cd apps/admin-web && npm run generate:admin-api`; `git diff --check`
  Review: Route must stay under `/admin/v1/*`, must be redaction-safe, must be
  idempotent or explicitly state conflict semantics, and must not touch Public
  Client API contracts.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. ASCA closed with `GET|PUT
  /admin/v1/settings/metadata/raw-cache`, generated Admin contract types, and
  restart-required effect semantics. Admin Web mutation may continue only for
  metadata raw cache settings.

## M3 - Admin Web Mutation UI

- [x] ASM-040 [owner=codex] [deps=ASM-030] [scope=apps/admin-web/src/features/settings,apps/admin-web/src/adminApi,apps/admin-web/src/App.test.tsx]
  Goal: Add route-owned controls for the implemented settings mutation slice
  with explicit prepare/confirm, success, error, and stale/fallback states.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`
  Review: UI must not show fake success in mock fallback, must not render unsafe
  config material, and must use `AdminDataSource` rather than direct fetches.
  Evidence: `apps/admin-web/src/features/settings/SettingsPage.tsx`
  Handoff: DONE. `/settings` now loads raw-cache settings separately, supports
  prepare/confirm save on live Admin API data, renders success and error
  notices, disables save in mock fallback, and shows `requires_restart` when
  the persisted desired state differs from the active process.

## M4 - Verification And Closeout

- [x] ASM-050 [owner=codex] [deps=ASM-040] [scope=apps/admin-web,crates/nako-api,crates/nako-server,docs/workstreams/admin-web-v2-settings-mutation-authority]
  Goal: Run focused and broad gates, record browser smoke, review the lane, and
  close or split follow-ons.
  Validation: `cargo fmt --all --check`; focused `cargo nextest`; `cd apps/admin-web && npm run generate:admin-api && npm run check && npm run test && npm run build`; `git diff --check`; Playwright desktop/mobile smoke.
  Review: Run `review-workstream` before closeout and record findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `CLOSEOUT.md`
  Handoff: DONE_WITH_CONCERNS. Focused Rust, Admin API, Admin Web, generated
  contract, build, whitespace, JSON, and desktop/mobile browser smoke gates are
  recorded. The ignored PostgreSQL repository contract remains skipped locally
  because `NAKO_TEST_POSTGRES_URL` is not configured. Next Admin Web V2 lane is
  users/permissions/Library Access or another explicitly scoped settings
  authority follow-on.
