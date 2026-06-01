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
