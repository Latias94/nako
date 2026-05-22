# Admin Web Addon Operations TODO

Status: Completed
Last updated: 2026-05-22

## AWAO.0 Contract Baseline

- [x] AWAO-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-addon-operations, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/README.md]
  Goal: Open the workstream, record scope/non-goals, and set the top-level
  goal for Admin Web Addon Operations.
  Validation: workstream docs agree with `CONTEXT.md` language and current
  Git status is clean after the planning commit.
  Evidence: this workstream and updated top-level docs.
  Handoff: Continue with AWAO-020 before UI wiring so the generated Admin API
  contract owns Addon wire DTOs.

- [x] AWAO-020 [owner=codex] [deps=AWAO-010] [scope=crates/nako-api/src/admin_contract.rs, apps/admin-web/src/adminApi/generated/contract.ts]
  Goal: Add generated Admin API TypeScript contract coverage for Addon
  Operations route constants and DTOs.
  Validation: `cargo run -p nako-api --example emit-admin-typescript-contract
  -- --output apps/admin-web/src/adminApi/generated/contract.ts`,
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`, and no
  forbidden sensitive terms in generated contract.
  Evidence: generated contract diff and passing admin contract tests.
  Handoff: Continue with AWAO-030.

## AWAO.1 Frontend Data Seam

- [x] AWAO-030 [owner=codex] [deps=AWAO-020] [scope=apps/admin-web/src/adminApi]
  Goal: Deepen the Admin Web Addon data seam: client methods, live/mock data
  loading, UI-oriented Addon Operations read model, and safe mock fixtures.
  Validation: `npm test -- --run src/adminApi` from `apps/admin-web` and
  focused redaction assertions.
  Evidence: `client.ts`, `dataSource.ts`, `types.ts`, `mockData.ts`, and tests.
  Handoff: Continue with AWAO-040.

## AWAO.2 Operator UI

- [x] AWAO-040 [owner=codex] [deps=AWAO-030] [scope=apps/admin-web/src/App.tsx, apps/admin-web/src/App.test.tsx]
  Goal: Render the Addons operations surface with list/detail facts,
  lifecycle controls, health status, declared surfaces, tokens/grants summary,
  and diagnostic result states.
  Validation: `npm test -- --run src/App.test.tsx`, `npm run build`, and no
  fixture/UI leakage of tokens, raw paths, or payloads.
  Evidence: UI tests and production build.
  Handoff: Continue with AWAO-050.

- [x] AWAO-050 [owner=codex] [deps=AWAO-040] [scope=apps/admin-web/src/adminApi, apps/admin-web/src/App.tsx]
  Goal: Wire safe Addon actions for enable/disable, health check, and
  resource-call diagnostics through the data-source seam.
  Validation: focused client/data-source/UI action tests using deterministic
  fetchers; actions must update UI state without echoing request payloads or
  secrets.
  Evidence: tests covering success and failure paths.
  Handoff: Continue with AWAO-060.

## AWAO.3 Closeout

- [x] AWAO-060 [owner=codex] [deps=AWAO-050] [scope=docs/workstreams/admin-web-addon-operations, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/README.md]
  Goal: Close the workstream with fresh evidence, update roadmap/goal docs,
  and record any split follow-ons.
  Validation: `cargo fmt --all -- --check`, Rust Admin API contract checks,
  Admin Web tests/build, `git diff --check`, and targeted Addon server tests
  if contract or route semantics changed.
  Evidence: `EVIDENCE_AND_GATES.md`, `MILESTONES.md`, `HANDOFF.md`, and close
  journal.
