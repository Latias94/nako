# Admin Web Addon Onboarding TODO

Status: Completed
Last updated: 2026-05-22

## AWAON.0 Planning

- [x] AWAON-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-addon-onboarding, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/README.md]
  Goal: Open the workstream, record the product boundary, and define the first
  executable onboarding slice.
  Validation: workstream docs use Taru Addon terms from `CONTEXT.md`; no
  lifecycle automation or URL fetch is included in scope.
  Evidence: this workstream and top-level tracker updates.
  Handoff: Continue with AWAON-020.

## AWAON.1 Manifest Registration Slice

- [x] AWAON-020 [owner=codex] [deps=AWAON-010] [scope=apps/admin-web/src/adminApi]
  Goal: Add typed Admin Web data-source/client support for registering an Addon
  from pasted manifest JSON with `status: "disabled"` by default.
  Validation: focused Admin Web tests for request shape, JSON parse failures,
  server validation failures, and no sensitive fields in returned onboarding
  state.
  Evidence: `npm test -- --run src/adminApi/client.test.ts`,
  `npm test -- --run src/adminApi/dataSource.test.ts`, and `npm run check`
  passed on 2026-05-22.
  Handoff: Continue with AWAON-030.

- [x] AWAON-030 [owner=codex] [deps=AWAON-020] [scope=apps/admin-web/src/App.tsx, apps/admin-web/src/App.test.tsx, apps/admin-web/src/styles.css]
  Goal: Render a safe Addon onboarding panel that lets an administrator paste
  manifest JSON, preview key facts, submit registration, and continue into the
  selected Addon Operations / Install Guide path.
  Validation: UI tests cover parse error, successful disabled registration,
  preview facts, and handoff messaging.
  Evidence: `npm test -- --run src/App.test.tsx` passed on 2026-05-22.
  Handoff: Continue with AWAON-040.

## AWAON.2 Contract and Documentation

- [x] AWAON-040 [owner=codex] [deps=AWAON-030] [scope=docs/api/HTTP_API.md, docs/guides/ADDON_AUTHOR_GUIDE.md, docs/GOALS.md, docs/ROADMAP.md]
  Goal: Document the onboarding flow as manifest registration, not sidecar
  installation or lifecycle control, and record follow-ons for token/grant UX
  and URL-based discovery.
  Validation: docs state sidecar reachability is verified by Health Check, not
  by registration; docs state the UI defaults to disabled.
  Evidence: `docs/api/HTTP_API.md` and `docs/guides/ADDON_AUTHOR_GUIDE.md`
  updated on 2026-05-22.
  Handoff: Continue with AWAON-050.

## AWAON.3 Closeout

- [x] AWAON-050 [owner=codex] [deps=AWAON-040] [scope=docs/workstreams/admin-web-addon-onboarding]
  Goal: Close the workstream with fresh evidence, update trackers, and record
  next recommended goal.
  Validation: `cargo fmt --all -- --check`, `cargo nextest run -p taru-api
  admin_contract --no-fail-fast`, `cargo check -p taru-api -p taru-server
  --tests`, `npm run check`, `npm test`, `npm run build`, and
  `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md`, `MILESTONES.md`, `HANDOFF.md`, close
  journal, and full closeout gates passed on 2026-05-22.
