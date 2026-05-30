# Generated Artifact Metadata Authority Apply - TODO

Status: Active
Last updated: 2026-05-30

## M0 - Scope And Evidence Freeze

- [x] GAMA-010 [owner=codex] [deps=none] [scope=docs/workstreams/generated-artifact-metadata-authority-apply,docs/architecture/WORKSTREAM_LINKS.md,docs/workstreams/README.md]
  Goal: Open the durable Metadata Authority apply lane from Generated Artifact review closeout.
  Validation: `python -m json.tool docs/workstreams/generated-artifact-metadata-authority-apply/WORKSTREAM.json`; `git diff --check -- docs/workstreams/generated-artifact-metadata-authority-apply docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`.
  Evidence: `APPLY_AUTHORITY_AUDIT.md`, `DESIGN.md`, `JOURNAL/2026-05-29-GAMA-010.md`.
  Handoff: Execution begins at `GAMA-020`.

## M1 - Apply Plan Contract

- [x] GAMA-020 [owner=codex] [deps=GAMA-010] [scope=crates/nako-core,crates/nako-api,crates/nako-server/src/app/automation.rs,crates/nako-server/src/http/admin.rs]
  Goal: Add a redaction-safe, read-only metadata apply-plan contract for accepted metadata Generated Artifacts.
  Validation: `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`; `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`.
  Review: `review-workstream` for contract shape, redaction, and no-mutation invariants.
  Evidence: API DTO test, server app test, and HTTP route test show redacted field summaries and no Canonical Metadata mutation.
  Handoff: Final route is `POST /admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply-plan`, response `AdminGeneratedArtifactMetadataApplyPlanResponse`. Execution continues at `GAMA-030`.

- [x] GAMA-030 [owner=codex] [deps=GAMA-020] [scope=crates/nako-core,crates/nako-db,crates/nako-server/src/app/automation.rs,crates/nako-server/src/app/metadata_application.rs]
  Goal: Add host-owned apply execution for executable plans, preserving field locks and revalidating target freshness before mutation.
  Validation: `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`; `cargo nextest run -p nako-db metadata_application --no-fail-fast`.
  Review: `review-workstream` for stale-target rejection, lock behavior, atomic item/projection commit, and no raw-payload leakage.
  Evidence: Server app tests cover changed fields, skipped locked fields, stale target rejection, idempotent replay, and catalog/search projection; DB contract covers atomic metadata application item/projection commit and rollback.
  Handoff: Durable apply audit/outcome persistence remains `GAMA-040`; final Admin apply route remains `GAMA-050`.

## M2 - Persistence And API Surface

- [x] GAMA-040 [owner=codex] [deps=GAMA-030] [scope=crates/nako-core,crates/nako-db,crates/nako-server]
  Goal: Persist idempotent apply audit/outcome when request-local apply result is not enough for retries, repair, or operations visibility.
  Validation: `cargo nextest run -p nako-db generated_artifact_metadata_apply_outcome --no-fail-fast`; `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`; `cargo nextest run -p nako-db postgres_metadata_catalog_contract_generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic --run-ignored ignored-only --no-fail-fast` with a temporary local PostgreSQL cluster.
  Review: `review-workstream` for SQLite/PostgreSQL parity and migration/backfill risk.
  Evidence: Added durable Generated Artifact metadata apply outcomes with SQLite/PostgreSQL schemas, repository contract coverage for idempotency and atomic metadata application commit, and app tests for durable replay plus failed outcomes.
  Handoff: Execution continues at `GAMA-050`; expose the final Admin route against the request/idempotency-key contract.

- [ ] GAMA-050 [owner=unassigned] [deps=GAMA-030] [scope=crates/nako-api,crates/nako-server/src/http,generated clients]
  Goal: Expose final Admin metadata apply route and keep wire contracts/generated clients synchronized.
  Validation: `cargo nextest run -p nako-server admin_generated_artifact_metadata_apply --no-fail-fast`; generated client contract check used by the repo at that point.
  Review: `review-workstream` for Admin API boundary and redaction.
  Evidence: HTTP tests for auth, route method/path/body, redacted response, idempotent replay, and error mapping.
  Handoff: Web work can begin only after this route is stable.

## M3 - Web Admin Apply Workflow

- [ ] GAMA-060 [owner=unassigned] [deps=GAMA-050] [scope=web/src]
  Goal: Add Web Admin apply-plan and confirm-apply workflow after Generated Artifact review acceptance.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run check`; `npm --prefix web run build:budget`.
  Review: `review-workstream` for live API boundary, no fixture mutation claims, and accessible confirmation states.
  Evidence: Web tests and browser screenshots for desktop/mobile apply-plan and result states.
  Handoff: Keep accept/reject review and Metadata Authority apply visibly separate.

## M4 - Verification And Closeout

- [ ] GAMA-070 [owner=planner] [deps=GAMA-060] [scope=docs/workstreams/generated-artifact-metadata-authority-apply]
  Goal: Verify backend/Web gates, update evidence, close the lane, and split bulk apply/provider mapping follow-ons if needed.
  Validation: `verify-rust-workstream` records fresh final gate evidence; `git diff --check`.
  Review: `review-workstream` has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, optional `CLOSEOUT.md`.
  Handoff: Summarize residual risks and next lane candidates.
