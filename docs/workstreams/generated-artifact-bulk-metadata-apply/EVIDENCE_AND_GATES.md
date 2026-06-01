# Generated Artifact Bulk Metadata Apply - Evidence And Gates

Status: Active
Last updated: 2026-06-01

## Always Run For Docs Changes

- `python -m json.tool docs/workstreams/generated-artifact-bulk-metadata-apply/WORKSTREAM.json`
- `git diff --check -- docs/workstreams/generated-artifact-bulk-metadata-apply docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md`

## Backend Gates

Use focused gates for each slice, then broaden if public contracts, schema, or
runtime behavior changes:

- `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-db generated_artifact_bulk_metadata_apply --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

Run matching PostgreSQL ignored contracts when schema or PostgreSQL repository
behavior changes.

## Web Gates

Only run after Admin contract support exists:

- `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
- `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`
- `npm --prefix web run test`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- Browser/Playwright smoke for desktop and mobile bulk plan/result states.

## Evidence Log

- `GABMA-010`: Opened the lane from GAMA closeout after confirming the shipped
  one-artifact apply workflow and bulk-apply follow-on boundary.
- `GABMA-020`: Added the read-only Admin bulk metadata apply-plan contract at
  `POST /admin/v1/automation/generated-artifacts/metadata-apply-plan`.
  Evidence: per-artifact planned/missing items, aggregate ready/blocked/stale
  and field-action counters, duplicate selection accounting, selection bound
  enforcement, generated Admin contract sync, and no Canonical Metadata mutation
  in app/HTTP tests.
  Gates: `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`;
  `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`;
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
- `GABMA-030`: Added durable bulk metadata apply batch persistence with batch
  identity, idempotency-key replay, selection and summary snapshots, per-item
  idempotency keys, queued/running state transition guard, and transactional
  rollback when item persistence fails.
  Gates: `cargo nextest run -p nako-db generated_artifact_bulk_metadata_apply --no-fail-fast`;
  `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
- `GABMA-040`: Added durable job-backed batch execution through the existing
  one-artifact Metadata Authority apply path. Batch rows now link to a queued
  `generated_artifact_metadata_bulk_apply` job, item rows persist terminal
  applied/noop/stale/failed outcomes, execution summary counters are derived
  from item state, and replay returns terminal batch state without duplicate
  mutations.
  Gates: `cargo nextest run -p nako-db generated_artifact_bulk_metadata_apply --no-fail-fast`;
  `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`.
  PostgreSQL ignored contract was attempted with
  `cargo nextest run -p nako-db --run-ignored only postgres_metadata_catalog_contract_generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic --no-fail-fast`
  but did not run because `NAKO_TEST_POSTGRES_URL` is not configured locally.
- `GABMA-050`: Exposed Admin bulk metadata apply confirm and status/result
  routes at
  `POST /admin/v1/automation/generated-artifacts/metadata-apply-batches` and
  `GET /admin/v1/automation/generated-artifacts/metadata-apply-batches/{batch_id}`.
  The Admin batch read model includes batch/job identity, queued/completed
  status, selection and plan snapshots, execution counters, and per-item
  outcome facts without exposing batch or item idempotency keys, raw prompts,
  raw artifact JSON, Source Locators, paths, tokens, or provider payloads.
  Gates: `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`;
  `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`;
  `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`;
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`;
  `cargo fmt --all -- --check`.
- `GABMA-060`: Added Web Admin bulk metadata apply planning, live-only
  confirmation, and batch status/result display on the Generated Artifacts
  Admin route. The Web data sources now map the bulk plan, confirmation, and
  status routes without exposing raw prompts, provider payloads, Source
  Locators, paths, raw artifact JSON, idempotency keys, or tokens. The route
  only allows accepted Generated Artifacts into the bulk selection, blocks
  fixture/fallback mutation, shows aggregate plan counters, and renders
  per-item batch outcomes. The Generated Artifacts page is lazy-loaded from
  the Admin surface so `admin-route-js` remains below the route budget; the
  total JS gzip budget was explicitly adjusted from 335 KiB to 340 KiB for
  this new async Admin workflow.
  Gates: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`;
  `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`;
  `npm --prefix web run check`; `npm --prefix web run build:budget`.
  Browser smoke: local Vite dev server at
  `http://127.0.0.1:3000/admin/automation/generated-artifacts` with mocked
  live Admin API completed selection, plan, confirm, and completed-batch
  status flows at desktop `1440x1000` and mobile `390x844`; screenshots:
  `.tmp/gabma060-desktop.png`, `.tmp/gabma060-mobile.png`; page text did not
  expose unsafe prompt/generated title/provider/idempotency/token strings.

## Final Evidence Checklist

Record before closeout:

- exact Admin route method/path/body/response for bulk plan, confirm, and
  status/results;
- no-mutation evidence for read-only bulk plan;
- selection bound and redaction evidence;
- durable batch idempotency and per-item idempotency evidence;
- partial failure and replay behavior;
- SQLite/PostgreSQL parity evidence if schema changes;
- Web live/fallback honesty and screenshots;
- closeout review result and follow-on split decisions.
