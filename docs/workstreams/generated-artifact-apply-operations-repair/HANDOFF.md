# Generated Artifact Apply Operations Repair — Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

This lane is newly opened from GAMA/GABMA/GAPM closeouts. The current source
already has:

- one-artifact Metadata Authority apply with durable outcomes and idempotent
  replay;
- bulk batch persistence with selection snapshots, summaries, and per-item
  statuses;
- Web Admin plan and batch views for the currently selected artifact(s);
- a dedicated Admin recovery queue for stale/noop/failed/skipped apply state,
  including outcome-only records and bulk batch terminal item records.

GAOR-020 audit conclusion:

- bulk batch visibility is already materially present across API and Web;
- one-artifact durable outcomes are now queryable through Admin list/detail
  read paths;
- the first repair surface is a read-only Admin recovery route that classifies
  outcome and batch item state by operator attention level.

## Active Task

- Task ID: `GAOR-040`
- Owner: planner
- Files: `docs/workstreams/generated-artifact-apply-operations-repair`, `docs/architecture`
- Validation: fresh closeout verification and `git diff --check`
- Status: DONE
- Review: GAOR-030 self-review completed with no blocking findings
- Evidence: `docs/workstreams/generated-artifact-apply-operations-repair/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Opened a new lane instead of reopening `generated-artifact-provider-mapping-breadth`.
- Chose a read-path audit as the first executable slice.
- Kept repair automation, provider-depth precision, and generic job retry UI
  out of scope for the first slice.
- Landed the first `GAOR-030` implementation slice as outcome-oriented Admin
  list/detail read paths instead of another batch-centric surface.
- Completed `GAOR-030` by adding a recovery queue route/read model that groups
  outcome-only and batch-linked terminal states into operator attention levels.
- Moved recovery classification into `nako-core` so SQLite/Postgres adapters do
  not duplicate repair semantics.

## Blockers

- None yet.

## Next Recommended Action

- Open a focused follow-on for `web-admin-generated-artifact-recovery-ui` or
  `generated-artifact-apply-repair-actions`; do not reopen this read-path lane.
