# Generated Artifact Provider Mapping Breadth - Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Always Run For Docs Changes

- `python -m json.tool docs/workstreams/generated-artifact-provider-mapping-breadth/WORKSTREAM.json`
- JSONL validation for
  `docs/workstreams/generated-artifact-provider-mapping-breadth/TASKS.jsonl`
  and
  `docs/workstreams/generated-artifact-provider-mapping-breadth/CAMPAIGNS.jsonl`
- `git diff --check -- docs/workstreams/generated-artifact-provider-mapping-breadth docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md docs/architecture/CONTROL_PLANE.md docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md`

## Backend Gates

Use focused gates for each slice, then broaden when public contracts, schema,
or runtime behavior changes:

- `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-db provider_mapping generated_artifact_metadata_apply --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

Run matching PostgreSQL ignored contracts or
`scripts/postgres-contract-harness.ps1 -Suite all-contracts` when repository
transaction behavior, schema, or PostgreSQL provider mapping persistence
changes.

## Web Gates

Only run after Admin contract support exists:

- `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
- `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- Browser smoke for desktop and mobile provider mapping plan/result states.

## Evidence Log

- `GAPM-010`: Opened the lane after GAMA/GABMA closeout and source coverage.
  Current source evidence shows Generated Artifact metadata apply parses only
  Canonical Metadata fields while Provider Subject/Mapping repositories and
  Admin governance summaries already exist. First executable task is
  `GAPM-020`, a read-only plan extension with no Provider Mapping mutation.
- `GAPM-020`: Added read-only Provider Mapping proposal planning to the
  Generated Artifact metadata apply plan. The core plan now carries
  `provider_mappings` plus apply/skip/noop Provider Mapping counters. The
  server parses host-interpreted `provider_subject` and `provider_subjects`
  payload shapes, supports typed TMDB/Douban/Bangumi/IMDb provider subjects,
  reports unsupported provider, unsupported subject kind, missing/invalid
  subject key, duplicate proposal, and existing mapping statuses as explicit
  plan reasons, marks applyable Provider Mapping entries as deferred and
  non-executable until persistence ships, and does not mutate Provider Subjects
  or Provider Mappings during planning. Admin DTOs and generated TypeScript
  contracts were synchronized, and `docs/api/HTTP_API.md` records the
  read-only plan behavior.
  Gates: `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`
  passed 4/4; `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  passed 5/5; `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`
  passed 6/6; `cargo nextest run -p nako-db generated_artifact_bulk_metadata_apply --no-fail-fast`
  passed 1/1 after the shared contract-test helper update; `cargo fmt --all -- --check`
  passed; `npm --prefix web run check` passed; `git diff --check` passed with
  only LF/CRLF normalization warnings.
- `GAPM-030`: Made final single-artifact Generated Artifact metadata apply
  persist Provider Subjects and accepted Provider Mappings only during the
  Metadata Authority apply step. The core outcome commit now carries Provider
  Mapping apply commits; SQLite and PostgreSQL outcome commits write Provider
  Subject/Mapping rows inside the same generated artifact metadata apply
  outcome transaction. Server apply now treats applyable Provider Mapping
  plans as executable, writes new accepted mappings, promotes existing
  candidate mappings, preserves existing rejected mappings as skipped/noop
  work, and returns durable idempotent replays without duplicate mappings.
  Gates: `cargo nextest run -p nako-api generated_artifact_metadata_apply admin_contract --no-fail-fast`
  passed 9/9; `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`
  passed 14/14; `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`
  passed 5/5; `cargo nextest run -p nako-db provider_mapping generated_artifact_metadata_apply --no-fail-fast`
  passed 3/3; `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts`
  passed 46/46 ignored PostgreSQL contracts, including
  `postgres_metadata_catalog_contract_generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic`
  and `postgres_metadata_catalog_contract_generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic`;
  the harness again warned that `pg_ctl stop` failed, but `127.0.0.1:55432`
  was closed afterward and `target/postgres-contract` was removed; `cargo fmt --all -- --check`
  passed; `WORKSTREAM.json` and JSONL validation passed; targeted and
  repository-wide `git diff --check` passed with only LF/CRLF normalization
  warnings.
- `GAPM-040`: Surfaced Provider Mapping apply/skip/noop counters through
  bulk apply plan summaries, persisted batch summaries, Admin DTOs, HTTP route
  responses, and generated TypeScript contracts. Bulk execution continues to
  call the single-artifact Metadata Authority apply path; tests prove Provider
  Mapping writes happen through that path and that redacted batch snapshots
  carry aggregate mapping counters without exposing raw payloads or
  idempotency keys. Web Admin read-model mapping now preserves the counters
  for the `GAPM-050` display slice.
  Gates: `cargo nextest run -p nako-api generated_artifact_metadata_apply admin_contract --no-fail-fast`
  passed 9/9; `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply generated_artifact_metadata_apply --no-fail-fast`
  passed 19/19; `cargo nextest run -p nako-db generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic --no-fail-fast`
  passed 1/1; `npm --prefix web run check` passed; `cargo fmt --all -- --check`
  passed; `WORKSTREAM.json` and JSONL validation passed; targeted and
  repository-wide `git diff --check` passed with only LF/CRLF normalization
  warnings.
- `GAPM-050`: Web Admin now renders Provider Mapping plan/result details from
  the existing Admin read models without adding backend behavior. The
  single-artifact Metadata Authority apply route shows a dedicated Provider
  Mapping plan table plus result replay facts; the bulk apply route exposes
  aggregate mapping counters and per-artifact mapping summaries in both plan
  and batch result states. Fixture data and live contract fixtures now carry
  redaction-safe mapping subjects, confidence, action, and existing-status
  facts so UI tests cover plan/result rendering and fallback honesty without
  exposing provider raw responses, prompts, idempotency keys, local paths, or
  raw payloads.
  Gates: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
  passed 37/37; `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`
  passed 55/55; `npm --prefix web run check` passed; `npm --prefix web run build:budget`
  passed with `admin-route-js` 207.38 KiB raw / 43.76 KiB gzip against the
  260/65 KiB budget; Playwright smoke against `http://127.0.0.1:3000` verified
  desktop `1440x900` and mobile `390x844` Metadata Authority apply states plus
  the bulk apply route render Provider Mapping facts without responsive
  overflow or secret leakage; browser console showed only the existing Vite
  Fast Refresh / React DevTools hook shim warning.

## Final Evidence Checklist

Closeout should record:

- exact accepted Generated Artifact payload shape supported by the host;
- Admin API plan/result fields for Provider Mapping proposals;
- no-mutation evidence for the read-only plan;
- final apply idempotency and atomic outcome evidence;
- SQLite/PostgreSQL parity evidence if persistence changed;
- bulk plan/result inheritance evidence;
- Web live/fallback redaction and screenshots;
- closeout review result and split follow-ons.
