# Generated Artifact Apply Operations Repair

Status: Active
Last updated: 2026-06-02

## Why This Lane Exists

Nako already has guarded Generated Artifact review, one-artifact Metadata
Authority apply, bulk apply, and Provider Mapping breadth. Operators can run
the workflow, but they still lack an explicit recovery surface when outcomes
are stale, skipped, noop, or failed across many accepted artifacts.

Today the system records enough state to prove something happened:

- one-artifact apply persists durable outcomes and idempotent replay facts;
- bulk apply persists batch identity, selection snapshots, summary counters,
  and per-item statuses;
- Web Admin can inspect the current plan and the latest submitted batch.

What is missing is the operations layer between "the mutation exists" and "the
operator can safely recover from imperfect outcomes". Without that layer:

- stale artifacts are visible only as counters or row statuses, not as a
  repair queue;
- repeated blind retries risk confusion around idempotent replay versus truly
  executable recovery work;
- there is no bounded Admin workflow for searching recent apply outcomes,
  filtering repairable states, or preparing recovery actions without exposing
  raw internal records.

## Relevant Authority

- ADRs:
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
  - `docs/adr/0053-application-control-plane-boundary.md`
- Existing docs:
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Related workstreams:
  - `docs/workstreams/generated-artifact-metadata-authority-apply/`
  - `docs/workstreams/generated-artifact-bulk-metadata-apply/`
  - `docs/workstreams/generated-artifact-provider-mapping-breadth/`

## Problem

Generated Artifact apply has become production-shaped enough that its failure
and recovery modes are now a product boundary:

- operators need to understand which accepted artifacts were applied, skipped,
  noop, stale, or failed;
- they need bounded repair actions that respect freshness, idempotency, and
  Metadata Authority policy;
- they should not need raw database access, log scraping, or blind replays to
  recover from partial apply outcomes.

The current product gap is not broad new mutation capability. The gap is
repair-oriented visibility and safe recovery semantics.

## Target State

When this lane closes:

1. Nako has an explicit Admin read model for Generated Artifact apply outcomes
   and/or batches that supports operator filtering by repair-relevant status.
2. The API and Web surfaces distinguish replayable success from actionable
   repair work.
3. Repair actions, if added in this lane, are bounded, confirmation-backed,
   idempotent, redacted, and reuse the existing one-artifact or batch apply
   semantics instead of creating a hidden parallel mutation path.
4. Stale, failed, skipped, and noop states are explained in operator terms and
   do not require raw payload, prompt, path, token, or secret exposure.
5. Follow-ons remain explicit for deeper provider identity precision,
   background automation policy, or large-scale batch orchestration.

## In Scope

- Audit existing durable outcome and batch records for repair-oriented
  operator surfaces.
- Define Admin API read-model shape for apply outcome or batch recovery views.
- Add Web/Admin workflow design for searching, filtering, or opening repairable
  apply outcomes.
- Add bounded repair actions only if they can reuse existing apply semantics
  and keep target freshness and idempotency intact.
- Add focused docs/tests/evidence for recovery-oriented redaction and replay
  honesty.

## Out Of Scope

- No raw database or provider payload dump in Admin/Web.
- No background auto-repair or autonomous mutation policy.
- No provider-depth or Provider Mapping conflict-resolution expansion.
- No broad automation rule engine for saved repair searches.
- No Public Client API changes.
- No generic job retry UI for all control-plane workflows.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing batch and outcome records are already rich enough to drive a first repair read model. | Medium | `generated-artifact-bulk-metadata-apply` and `generated-artifact-provider-mapping-breadth` closeouts; `crates/nako-core/src/automation.rs`; `web/src/api/admin/read-models-data-source.ts` | If false, the first slice must deepen persistence/DTO shape before any Web route exists. |
| Operators mainly need bounded inspection and targeted repair, not a broad automation/retry engine. | Medium | Follow-on notes in GAMA/GABMA/GAPM closeouts; control-plane boundaries in ADR 0053 | If false, this lane may need to split again into audit surface vs. durable repair execution. |
| Repair should stay inside the library-metadata-control-plane lane because it reuses Metadata Authority semantics. | High | `docs/architecture/LANES.md`; `LIBRARY_PIPELINE.md`; `CONTROL_PLANE.md` | If false, the lane routing must be reconsidered before implementation. |

## Architecture Direction

### Repair Is A First-Class Admin Workflow

Generated Artifact apply already has durable outcomes. The next step is not to
hide recovery in logs or ad hoc scripts. It is to expose an Admin-only,
redaction-safe, recovery-oriented control-plane surface.

### Read Paths First

The first executable slice should be read-only:

- what records exist today;
- which statuses matter for operator recovery;
- what can be safely shown without raw payload leakage;
- which actions are genuinely repairable instead of merely replayable.

This mirrors the earlier plan-first pattern from review/apply/bulk lanes and
keeps mutation scope honest.

## GAOR-020 Audit Snapshot

The current system is asymmetric:

- one-artifact apply persists durable outcome records in `nako-core` and the
  server app layer records `Applied`, `Noop`, and `Failed` outcomes with
  durable plan snapshots and safe error fields;
- bulk apply persists batch records, per-item execution status, counters, and
  links back to one-artifact outcome ids;
- Admin HTTP and Web already expose create/get batch routes and read models;
- there is still no Admin route, DTO, or Web read model for querying durable
  one-artifact apply outcomes directly.

That asymmetry matters because repair is outcome-driven, not batch-driven.
`stale`, `failed`, `skipped`, and `noop` are split across two persistence
layers:

- `Failed` and `Noop` exist as one-artifact outcome statuses;
- `Stale`, `Skipped`, and per-item failure state exist on bulk batch items;
- batches can reference outcome ids, but the product has no operator-facing
  outcome index to answer "what needs repair now?" outside the latest batch
  detail.

## Recommended First Read Surface

The smallest useful repair-oriented Admin surface is an outcome-first read path
with optional batch context, not a new batch-only screen.

Why this is the right first slice:

1. batch detail already exists, so another batch-centric route would mostly
   duplicate existing visibility;
2. one-artifact apply is the canonical mutation boundary, so repair semantics
   should be anchored to durable apply outcomes before inventing broader queue
   abstractions;
3. batch items already point to outcome ids, which lets a future route enrich
   outcome rows with replay/batch provenance without duplicating mutation
   semantics.

Recommended API shape for `GAOR-030`:

- add an Admin list/read route for Generated Artifact apply outcomes;
- support filtering by repair-relevant states derived from persisted data;
- expose redaction-safe summaries only: artifact id, item id, apply status,
  created/updated timestamps, safe error code/message, replay hints, and
  optional batch references when present;
- keep raw payload JSON, prompt text, provider secrets, and raw path diagnostics
  out of the response.

## Replay Versus Repair Semantics

The audit makes one boundary explicit:

- replayable success: a persisted `Applied` or `Noop` outcome that can explain
  what already happened, including idempotent replay, but does not itself imply
  repair work;
- actionable repair: failed/stale/skipped state that indicates either the
  target moved, the plan became non-executable, or execution ended with a safe
  failure that an operator may need to inspect before retrying through the
  existing mutation path.

This means the first route should not flatten everything into one generic
`retryable` flag. It should preserve operator clarity about whether they are
looking at history, replay evidence, or unresolved repair work.

### Reuse Existing Apply Semantics

If the lane introduces a repair action, it must not invent a second
Metadata Authority executor. Recovery should route through the same final apply
semantics, target freshness checks, idempotency rules, and durable outcome
policy already established by GAMA/GABMA/GAPM.

### Differentiate Replay From Repair

An idempotent replay result is not the same thing as a repairable failure.
Operator UX and API semantics should make that distinction explicit, otherwise
the product will encourage meaningless retries.

## Closeout Condition

This lane can close when:

- the current repair visibility gap is implemented or explicitly split;
- evidence proves operators can inspect repair-relevant outcome state through
  safe Admin surfaces;
- any repair mutation added in-lane is bounded, confirmation-backed,
  idempotent, and redacted;
- docs reflect the shipped behavior and any remaining larger operations work is
  split into explicit follow-ons.
