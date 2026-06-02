# Provider Governance Durable Batch Execution - Design

Status: Active
Last updated: 2026-06-02

## Why This Lane Exists

`provider-governance-bulk-review` shipped a bounded synchronous Admin batch
apply route for durable Metadata Candidate Reviews. That route is intentionally
safe for small selections, but PGBR closeout explicitly routes retry, cancel,
progress, and larger selections through ADR 0053 control-plane job/runtime
boundaries.

This lane creates that durable execution boundary without widening the domain
mutation semantics.

## Relevant Authority

- ADRs:
  - `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
  - `docs/adr/0053-application-control-plane-boundary.md`
- Architecture maps:
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/LANES.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Related workstreams:
  - `docs/workstreams/provider-governance-bulk-review/`
  - `docs/workstreams/admin-web-provider-depth-governance/`
  - `docs/workstreams/accepted-review-provider-mapping-application/`
  - `docs/workstreams/durable-job-queue-and-resource-classes/`

## Problem

The current Candidate Review batch apply route:

- is capped at 50 reviews and runs synchronously in the HTTP request;
- returns useful partial results but does not persist batch/item execution
  state;
- cannot provide durable progress, cancellation, replayable status reads, or
  retry-ready evidence;
- would violate ADR 0053 if expanded by simply raising limits or hiding a raw
  background task in the route handler.

Generated Artifact bulk apply has a durable batch/job precedent, but PGBR
explicitly rejected reusing those outcome tables as Candidate Review state.

## Target State

When this workstream closes:

- Admin can create a durable Metadata Candidate Review batch from selected
  review IDs and an idempotency key.
- The server persists batch selection, plan snapshots, items, per-item
  idempotency keys, job ID, status, execution summary, redacted errors, and
  item outcomes.
- Batch execution runs through `DurableJobRuntime` and `RuntimeSupervisor`
  resource policy, with a Candidate Review job kind mapped to the metadata
  shared runtime budget class.
- The execution loop calls the existing
  `MetadataCandidateReviewApplicationService` per item, preserving stale guard,
  replay, root-only Provider Subject / Provider Mapping application, and
  redaction semantics.
- Admin API exposes create/status reads and later Web Admin can queue and poll
  durable batches without rendering raw provider/local/secret/idempotency facts.
- Follow-ons for related hierarchy application, Public Client API exposure,
  provider endpoint depth, and audit/undo remain separate.

## In Scope

- Candidate Review durable batch domain records in `nako-core`.
- `JobKind::MetadataCandidateReviewBatchApply` or equivalent explicit job kind.
- Metadata repository contracts for batch commit, lookup, status transition, and
  item outcome commit.
- SQLite and PostgreSQL persistence plus repository contract tests.
- Server metadata application service for durable batch create/read/execute.
- Runtime resource-class mapping to a metadata budget class.
- Admin API DTOs/routes for durable batch creation and status reads.
- Web Admin integration only after backend semantics and generated contracts are
  stable.
- Workstream docs, gates, and closeout.

## Out Of Scope

- Related Provider Subject, child Provider Mapping, or Media Item hierarchy
  application.
- Public Client API routes.
- Douban TV/episode endpoint depth or any provider endpoint breadth.
- Generated Artifact apply outcome table reuse.
- New global job priority policy, distributed scheduling, or remote workers.
- Audit/undo governance beyond the minimal batch/item outcome evidence needed
  for status reads.
- Raising or replacing the existing bounded synchronous route before the
  durable route proves itself.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing single-review application service is the only apply authority this lane should call. | High | `docs/workstreams/provider-governance-bulk-review/CLOSEOUT.md`; `crates/nako-metadata/src/candidate_review.rs` | If wrong, this lane needs an ADR-level metadata application redesign. |
| Durable execution can follow the Generated Artifact bulk apply shape without reusing its tables. | High | `crates/nako-core/src/automation.rs`; `crates/nako-server/src/app/automation.rs`; PGBR non-goals | If wrong, first slice should stop at a smaller repository contract proof. |
| Candidate Review batch jobs belong to metadata shared runtime budget. | Medium | `crates/nako-server/src/app/runtime.rs`; ADR 0053 | If wrong, PGDBE-020 must split a resource-class policy decision before execution. |
| Web can wait until backend status semantics exist. | High | PGBR already shipped synchronous Web governance | If wrong, PGDBE-030 must include a minimal Web route contract earlier. |

## Architecture Direction

Ownership should be explicit:

- `nako-core` owns durable Candidate Review batch records, item status, batch
  status, execution summary, job resource constant, and repository trait shape.
- `nako-db` owns schema, idempotent commit, outcome updates, status transitions,
  and SQLite/PostgreSQL parity.
- `nako-server::app::metadata` owns orchestration: plan snapshot creation,
  job-backed execution, per-item application, cancellation checkpoints, and
  redacted error classification.
- `nako-api::admin` owns DTOs that expose batch status and item results without
  leaking raw provider payloads, local paths, source fingerprints, provider
  response bodies, bearer tokens, or raw idempotency keys.
- Web Admin consumes only Admin API read models and should not infer hidden
  success from fixture fallback.

The first vertical task is deliberately core/DB-heavy. Without durable batch
state, a server route would only create another transient executor.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | COVERED | `CONTEXT.md` | Uses Candidate Review, Provider Subject, Provider Mapping, Admin API, Public Client API. |
| Control-plane ADR | COVERED | `docs/adr/0053-application-control-plane-boundary.md` | Requires durable job or supervised runtime boundary for important background work. |
| Durable job ADR | COVERED | `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md` | Requires persisted inputs and explicit retry policy. |
| PGBR closeout | COVERED | `docs/workstreams/provider-governance-bulk-review/CLOSEOUT.md` | Defines durable execution as focused follow-on and preserves non-goals. |
| Single-review apply authority | COVERED | `docs/workstreams/accepted-review-provider-mapping-application/`; `docs/workstreams/admin-web-provider-depth-governance/` | Existing service remains the mutation boundary. |
| Generated Artifact bulk precedent | COVERED | `docs/workstreams/generated-artifact-bulk-metadata-apply/`; `crates/nako-server/src/app/automation.rs` read as precedent | Pattern only; do not reuse outcome tables. |
| Planner inventory scripts | MISSING | `scripts/workstream_inventory.py`, `scripts/program_status.py`, `scripts/validate_orchestration_state.py` absent in checkout | Manual JSON scan found no active workstreams; script restoration is a separate tooling follow-on. |

## Closeout Condition

This lane can close when:

- durable batch create/status/execute is implemented and tested;
- batch execution proves cancellation checkpoints and redacted per-item outcome
  persistence;
- Admin API and Web behavior, if included, reflect the durable status model;
- architecture maps route the shipped behavior as evidence;
- related hierarchy application, Public Client API exposure, provider endpoint
  breadth, and audit/undo remain split or get their own workstreams.
