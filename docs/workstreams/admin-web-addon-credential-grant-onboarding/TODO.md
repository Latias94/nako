# Admin Web Addon Credential and Grant Onboarding TODO

Status: Completed
Last updated: 2026-05-22

## AWACG.0 Planning

- [x] AWACG-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-addon-credential-grant-onboarding, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/README.md]
  Goal: Open the workstream and capture token/grant safety boundaries.
  Validation: docs clearly distinguish credentials/grants from sidecar
  lifecycle automation.
  Evidence: this workstream and tracker updates.
  Handoff: Continue with AWACG-020.

## AWACG.1 Contract and Data Seam

- [x] AWACG-020 [owner=codex] [deps=AWACG-010] [scope=crates/taru-api/src/admin_contract.rs, apps/admin-web/src/adminApi/generated/contract.ts, apps/admin-web/src/adminApi]
  Goal: Add generated contract coverage and Admin Web client/data-source
  actions for token issue/rotate/revoke and grant replacement.
  Validation: admin contract tests, focused Admin Web client/data-source tests,
  and explicit assertions that raw tokens are action-only and not load data.
  Evidence: `cargo nextest run -p taru-api admin_contract --no-fail-fast`,
  `npm test -- --run src/adminApi/client.test.ts`, and
  `npm test -- --run src/adminApi/dataSource.test.ts` passed on 2026-05-22.
  Handoff: Continue with AWACG-030.

## AWACG.2 Operator UI

- [x] AWACG-030 [owner=codex] [deps=AWACG-020] [scope=apps/admin-web/src/App.tsx, apps/admin-web/src/App.test.tsx, apps/admin-web/src/styles.css]
  Goal: Render token issue/rotate/revoke controls, grant replacement editor,
  one-time raw token notice, and enable readiness checklist.
  Validation: UI tests cover one-time token display, revoke status update,
  grant replacement, checklist messaging, and no unsafe render terms.
  Evidence: `npm test -- --run src/App.test.tsx` and `npm run check` passed on
  2026-05-22.
  Handoff: Continue with AWACG-040.

## AWACG.3 Docs and Closeout

- [x] AWACG-040 [owner=codex] [deps=AWACG-030] [scope=docs/api/HTTP_API.md, docs/guides/ADDON_AUTHOR_GUIDE.md, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/admin-web-addon-credential-grant-onboarding]
  Goal: Document the Admin Web credential/grant onboarding flow and close the
  workstream with fresh evidence.
  Validation: `cargo fmt --all -- --check`, `cargo nextest run -p taru-api
  admin_contract --no-fail-fast`, focused server token/grant tests if route
  semantics changed, `cargo check -p taru-api -p taru-server --tests`, Admin Web
  check/test/build, and `git diff --check`.
  Evidence: closeout journal, evidence file, and full validation passed on
  2026-05-22.
