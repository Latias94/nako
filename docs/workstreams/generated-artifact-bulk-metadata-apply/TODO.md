# Generated Artifact Bulk Metadata Apply - TODO

Status: Active
Last updated: 2026-06-01

## M0 - Scope And Evidence Freeze

- [x] GABMA-010 [owner=planner] [deps=none] [scope=docs/workstreams/generated-artifact-bulk-metadata-apply,docs/architecture/LANES.md,docs/architecture/WORKSTREAM_LINKS.md,docs/workstreams/README.md]
  Goal: Open the durable bulk Metadata Authority apply lane from GAMA closeout.
  Validation: `python -m json.tool docs/workstreams/generated-artifact-bulk-metadata-apply/WORKSTREAM.json`; `git diff --check -- docs/workstreams/generated-artifact-bulk-metadata-apply docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`.
  Evidence: `DESIGN.md`, `MILESTONES.md`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `GABMA-020`.

## M1 - Read-Only Bulk Apply Plan

- [x] GABMA-020 [owner=codex] [deps=GABMA-010] [scope=crates/nako-core,crates/nako-api,crates/nako-server/src/app/automation.rs,crates/nako-server/src/http/admin.rs,docs/api/HTTP_API.md]
  Goal: Add a redaction-safe, read-only bulk metadata apply-plan contract for
  selected accepted metadata Generated Artifacts.
  Validation: `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`; `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`; `cargo fmt --all -- --check`.
  Review: contract shape, selection limits, no-mutation behavior, and redaction.
  Evidence: API/server tests show per-artifact plan summaries, aggregate
  counters, bounded selection, and no Canonical Metadata mutation.
  Handoff: Continue at `GABMA-030` only after the read-only plan route is
  stable.

## M2 - Durable Batch Request And Persistence

- [ ] GABMA-030 [owner=codex] [deps=GABMA-020] [scope=crates/nako-core,crates/nako-db,crates/nako-server]
  Goal: Persist a confirmed bulk apply request with batch identity, selection
  snapshot, aggregate plan snapshot, state, and per-item idempotency seeds.
  Validation: `cargo nextest run -p nako-db generated_artifact_bulk_metadata_apply --no-fail-fast`; PostgreSQL ignored contract when schema changes.
  Review: SQLite/PostgreSQL parity, idempotency-key uniqueness, batch state
  transitions, and rollback behavior.
  Evidence: repository contracts prove idempotent batch creation and safe
  replay without item mutation.
  Handoff: Continue at `GABMA-040` for execution.

- [ ] GABMA-040 [owner=codex] [deps=GABMA-030] [scope=crates/nako-server/src/app/automation.rs,crates/nako-server/src/app/runtime*,crates/nako-db]
  Goal: Execute confirmed batches through the existing one-artifact apply path
  with per-item outcomes, partial-failure accounting, and durable terminal
  state.
  Validation: `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`; `cargo nextest run -p nako-db generated_artifact_bulk_metadata_apply --no-fail-fast`.
  Review: no request-path unbounded mutation, cancellation/retry semantics,
  per-item idempotency, and redacted failure diagnostics.
  Evidence: server tests cover applied/noop/failed/stale mixes and replay.
  Handoff: Continue at `GABMA-050` for Admin read model and route exposure.

## M3 - Admin Read Models And Web Workflow

- [ ] GABMA-050 [owner=codex] [deps=GABMA-040] [scope=crates/nako-api,crates/nako-server/src/http,generated clients]
  Goal: Expose final Admin bulk apply confirm/status/result routes and keep
  generated contracts synchronized.
  Validation: `cargo nextest run -p nako-api admin_contract generated_artifact_metadata_apply --no-fail-fast`; `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`.
  Review: Admin API redaction, route auth, generated client sync, and no Public
  Client API leakage.
  Evidence: Admin route tests cover auth, request body, replay, status, partial
  results, and error mapping.
  Handoff: Continue at `GABMA-060` for Web.

- [ ] GABMA-060 [owner=codex] [deps=GABMA-050] [scope=web/src]
  Goal: Add Web Admin bulk metadata apply planning, confirmation, live-only
  mutation, and partial-result display.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke for desktop and mobile plan/result states.
  Review: live API boundary, fixture/fallback honesty, accessible selection and
  result states, and bundle-budget pressure.
  Evidence: Web tests and screenshots prove selection, confirm, disabled
  fallback mutation, partial results, and redaction.
  Handoff: Continue at `GABMA-070`.

## M4 - Verification And Closeout

- [ ] GABMA-070 [owner=planner] [deps=GABMA-060] [scope=docs/workstreams/generated-artifact-bulk-metadata-apply,docs/architecture]
  Goal: Verify backend/Web gates, update evidence, close the lane, and split
  provider mapping breadth or operations repair if needed.
  Validation: fresh focused Rust/Web gates, JSON validation, and `git diff --check`.
  Review: workstream compliance and code-quality review.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, optional
  `CLOSEOUT.md`.
  Handoff: DONE or split follow-ons.
