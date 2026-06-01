# Generated Artifact Apply Operations Repair — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

This lane is newly opened from GAMA/GABMA/GAPM closeouts. The current source
already has:

- one-artifact Metadata Authority apply with durable outcomes and idempotent
  replay;
- bulk batch persistence with selection snapshots, summaries, and per-item
  statuses;
- Web Admin plan and batch views for the currently selected artifact(s);
- no dedicated operator-facing repair queue, outcome search, or bounded repair
  surface for stale/noop/failed/skipped apply results.

GAOR-020 audit conclusion:

- bulk batch visibility is already materially present across API and Web;
- one-artifact durable outcomes are persisted but not queryable through Admin;
- the most leveraged first repair surface is an Admin outcome-oriented read
  path with optional batch context, not another batch-centric route.

## Active Task

- Task ID: `GAOR-030`
- Owner: codex
- Files: `crates/nako-core`, `crates/nako-db`, `crates/nako-api`, `crates/nako-server`, `web/src/api/admin`
- Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo check -p nako-server --tests`; `npm --prefix web run check`
- Status: READY
- Review: not started
- Evidence: `docs/workstreams/generated-artifact-apply-operations-repair/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Opened a new lane instead of reopening `generated-artifact-provider-mapping-breadth`.
- Chose a read-path audit as the first executable slice.
- Kept repair automation, provider-depth precision, and generic job retry UI
  out of scope for the first slice.
- Landed the first `GAOR-030` implementation slice as outcome-oriented Admin
  list/detail read paths instead of another batch-centric surface.

## Blockers

- None yet.

## Next Recommended Action

- Continue `GAOR-030` by deciding whether to add a dedicated Admin route/view
  that groups repairable outcomes (`failed` plus batch-linked `stale/skipped`)
  or to split a narrower Web/Admin product follow-on before any repair
  mutation is introduced.
