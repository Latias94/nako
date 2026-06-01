# Generated Artifact Provider Mapping Breadth - Evidence And Gates

Status: Active
Last updated: 2026-06-01

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
