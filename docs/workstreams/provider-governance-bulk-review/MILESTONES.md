# Provider Governance Bulk Review - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

Status: Complete after `PGBR-010`.

Exit criteria:

- workstream docs exist and agree;
- architecture lane maps route Provider Governance bulk review as active;
- first executable task is read-only;
- non-goals exclude Public Client API, provider endpoint depth, related
  hierarchy application, unbounded batches, and hidden background work.

## M1 - Read-Only Batch Plan

Status: Complete after `PGBR-020`.

Exit criteria:

- Admin API exposes a read-only batch plan for selected Metadata Candidate
  Review IDs;
- selection size is bounded and duplicate IDs are handled deterministically;
- each row is classified through existing single-review application planning
  semantics;
- route tests prove no writes and redaction-safe output;
- generated Admin TypeScript contract is synchronized.

## M2 - Confirmed Backend Batch Apply

Status: Ready at `PGBR-030`.

Exit criteria:

- batch confirmation accepts only an explicit plan/selection and operator
  idempotency key;
- each review preserves stale guard, replay, conflict, and partial-failure
  behavior;
- root Provider Subject / Provider Mapping application remains the only
  mutation scope;
- no raw provider/local/secret/idempotency facts leak in results;
- durable job execution is split if bounded synchronous execution is not
  sufficient.

## M3 - Web Admin Batch Governance

Status: Planned after `PGBR-030`.

Exit criteria:

- Web Admin global Candidate Review queue supports explicit review selection;
- operators can inspect a batch plan before confirming;
- confirmation uses live Admin API behavior and does not fake fixture success;
- route state, data-source contracts, bundle budget, and browser smoke pass.

## M4 - Closeout And Follow-On Split

Status: Planned at `PGBR-050`.

Exit criteria:

- target state and gates are complete;
- workstream ledgers and architecture maps agree;
- related hierarchy application, provider endpoint depth, Public Client API,
  durable job expansion, and broader provider governance are closed, split, or
  deferred explicitly.
