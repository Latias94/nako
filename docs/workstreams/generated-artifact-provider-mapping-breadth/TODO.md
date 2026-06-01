# Generated Artifact Provider Mapping Breadth - TODO

Status: Active
Last updated: 2026-06-01

## M0 - Scope And Evidence Freeze

- [x] GAPM-010 [owner=planner] [deps=none] [scope=docs/workstreams/generated-artifact-provider-mapping-breadth,docs/architecture/LANES.md,docs/architecture/WORKSTREAM_LINKS.md,docs/architecture/LIBRARY_PIPELINE.md,docs/architecture/CONTROL_PLANE.md,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the Provider Mapping breadth lane from GAMA/GABMA closeout and
  source coverage.
  Validation: `python -m json.tool docs/workstreams/generated-artifact-provider-mapping-breadth/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl` and `CAMPAIGNS.jsonl`; `git diff --check -- docs/workstreams/generated-artifact-provider-mapping-breadth docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md docs/architecture/CONTROL_PLANE.md docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md`.
  Evidence: `DESIGN.md`, `MILESTONES.md`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `GAPM-020`.

## M1 - Read-Only Provider Mapping Plan

- [x] GAPM-020 [owner=codex] [deps=GAPM-010] [scope=crates/nako-core,crates/nako-api,crates/nako-server/src/app/automation.rs,crates/nako-server/src/http/admin.rs,docs/api/HTTP_API.md,generated clients]
  Goal: Extend the existing Generated Artifact metadata apply plan with
  redaction-safe, read-only Provider Mapping proposal entries and counters.
  Validation: `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`; `cargo fmt --all -- --check`.
  Review: accepted-artifact requirement, target freshness, no Provider Mapping
  mutation, parser strictness, plan redaction, and generated contract sync.
  Evidence: API/server tests show supported provider subject proposals,
  unsupported/invalid proposal reasons, existing mapping comparison, and no
  Canonical Metadata or Provider Mapping writes during plan.
  Handoff: DONE. Continue at `GAPM-030`; persistence must preserve idempotent
  outcome replay and avoid review-acceptance mutation.

## M2 - Durable Provider Mapping Apply

- [ ] GAPM-030 [owner=codex] [deps=GAPM-020] [scope=crates/nako-core,crates/nako-db,crates/nako-server/src/app/automation.rs]
  Goal: Make final Generated Artifact metadata apply upsert Provider Subjects
  and accepted Provider Mappings idempotently through host-owned repositories.
  Validation: `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`; `cargo nextest run -p nako-db provider_mapping generated_artifact_metadata_apply --no-fail-fast`; PostgreSQL ignored contract or harness when transaction/repository behavior changes; `cargo fmt --all -- --check`.
  Review: atomic outcome persistence, idempotent replay, existing
  candidate/rejected mapping handling, source/provenance choice, and no review
  acceptance mutation.
  Evidence: server/db tests prove first apply creates or updates the expected
  Provider Mapping, replay does not duplicate it, stale targets fail before
  mutation, and mixed metadata-field/provider-mapping apply records one durable
  outcome.
  Handoff: Continue at `GAPM-040` for bulk/Admin surface reconciliation.

## M3 - Bulk/Admin Surface Reconciliation

- [ ] GAPM-040 [owner=codex] [deps=GAPM-030] [scope=crates/nako-core,crates/nako-api,crates/nako-server,generated clients]
  Goal: Surface Provider Mapping counters and outcomes through bulk plan,
  batch result, Admin DTOs, HTTP routes, and generated TypeScript contracts.
  Validation: `cargo nextest run -p nako-api generated_artifact_metadata_apply admin_contract --no-fail-fast`; `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply generated_artifact_metadata_apply --no-fail-fast`; `cargo fmt --all -- --check`.
  Review: no duplicate bulk implementation, partial-failure reporting,
  redacted batch snapshots, route auth, and no Public Client route leakage.
  Evidence: bulk tests prove provider mapping plans/results flow through the
  one-artifact apply path and batch summaries count mapping apply/noop/skip
  facts.
  Handoff: Continue at `GAPM-050` for Web Admin.

- [ ] GAPM-050 [owner=codex] [deps=GAPM-040] [scope=web/src]
  Goal: Add Web Admin Provider Mapping plan/result display to Generated
  Artifact single and bulk metadata apply workflows.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke for desktop and mobile plan/result states.
  Review: live API boundary, fixture/fallback honesty, accessible mapping
  display, redaction, and bundle-budget pressure.
  Evidence: Web tests and screenshots prove provider mapping proposal display,
  confirmation, result rendering, disabled fallback mutation, and no unsafe
  payload/path/token leakage.
  Handoff: Continue at `GAPM-060`.

## M4 - Verification And Closeout

- [ ] GAPM-060 [owner=planner] [deps=GAPM-050] [scope=docs/workstreams/generated-artifact-provider-mapping-breadth,docs/architecture]
  Goal: Verify backend/Web/PostgreSQL/docs gates, close the lane, and split
  provider-depth, conflict repair, or operations repair follow-ons if needed.
  Validation: fresh focused Rust/Web gates, PostgreSQL parity if repository
  transaction behavior changed, JSON/JSONL validation, and `git diff --check`.
  Review: workstream compliance and code-quality review.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, optional
  `CLOSEOUT.md`.
  Handoff: DONE or split follow-ons.
