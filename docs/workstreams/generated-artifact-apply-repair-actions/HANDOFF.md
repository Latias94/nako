# Generated Artifact Apply Repair Actions — Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

This lane is closed. The seam proof selected the existing Metadata Authority
single/bulk apply routes as the repair execution boundary. A backend recovery
mutation wrapper is not justified for the current product shape.

Operators can inspect recovery entries, open the current Metadata Authority
apply plan for the row's artifact, and confirm live mutation with a newly
generated Web idempotency key. The flow remains preparation-first: the recovery
queue itself does not execute mutation or replay an old plan snapshot.

## Active Task

- Task ID: `GAARA-050`
- Owner: planner
- Files: `docs/workstreams/generated-artifact-apply-repair-actions`, `docs/architecture`, `docs/GOALS.md`, `docs/ROADMAP.md`
- Validation: fresh gate evidence, JSON/JSONL validation, and `git diff --check`
- Status: DONE
- Evidence: `docs/workstreams/generated-artifact-apply-repair-actions/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Opened a new `library-metadata-control-plane` follow-on instead of reopening
  GAOR or WAGR.
- First task is a seam proof, not a mutation implementation.
- Existing Metadata Authority apply and bulk apply are the preferred execution
  kernels.
- Read-only explorer audit concluded no new metadata mutation core is needed;
  a narrow wrapper is justified only for recovery-context guards or one-click
  repair UX.
- `GAARA-020` proved the current Web recovery-row-to-apply-plan path is enough:
  it fetches the current apply plan, does not mutate before confirmation, uses
  a fresh Web idempotency key, and does not expose or reuse recovery-row
  idempotency data.
- `GAARA-030` and `GAARA-040` are deferred, not the next automatic tasks.
  Reopen them only for a product-approved one-click wrapper or explicit UX
  polish.
- `GAARA-050` closed the lane and recorded follow-ons in `CLOSEOUT.md`.

## Blockers

- None.

## Next Recommended Action

- Do not resume this workstream for ordinary repair action work. Open a new
  workstream if product chooses one-click repair, Web repair copy polish,
  provider-depth precision, or Admin settings restoration.
