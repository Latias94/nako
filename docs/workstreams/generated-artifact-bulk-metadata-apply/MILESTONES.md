# Generated Artifact Bulk Metadata Apply - Milestones

Status: Closed
Last updated: 2026-06-01

## GABMA-M0 - Lane Opened

Outcome: The bulk apply lane is ready for implementation.

Deliverables:

- workstream docs;
- active lane registry entry;
- first executable task.

Exit criteria:

- `WORKSTREAM.json` validates;
- active queue points to `GABMA-020`.

## GABMA-M1 - Bulk Plan Contract

Outcome: Admin can request a read-only plan for a bounded selection of accepted
metadata Generated Artifacts.

Exit criteria:

- plan route is Admin-only;
- route performs no Canonical Metadata mutation;
- response includes aggregate and per-item redacted facts;
- unsupported/stale/non-executable artifacts are visible as skipped or blocked.

## GABMA-M2 - Durable Batch Execution

Outcome: Confirmed bulk apply runs durably and idempotently outside the request
path.

Exit criteria:

- confirmed batch has a stable identity;
- repeated confirm requests replay or return the same batch;
- per-item idempotency keys prevent duplicate mutations;
- partial failure is persisted and redacted.

## GABMA-M3 - Admin And Web Product Surface

Outcome: Operators can plan, confirm, observe, and understand a bulk metadata
apply batch from Admin Web.

Exit criteria:

- generated Admin contracts are synchronized;
- Web fixture/fallback mode cannot claim live mutation;
- Web shows aggregate and per-item results without leaking raw payloads;
- per-route bundle budgets remain within limits, and any total budget change
  is explicit evidence for closeout review.

## GABMA-M4 - Closeout

Outcome: The bulk apply lane is verified and either closed or split.

Exit criteria:

- focused Rust/Web gates pass;
- residual provider mapping breadth and operations repair are not hidden in
  this lane;
- architecture and workstream docs reflect the shipped state.

Closeout: satisfied by `GABMA-070` on 2026-06-01. The lane is closed; provider
mapping breadth, apply operations repair, and Admin settings restoration remain
separate proposed follow-ons.
