# Architecture Review Follow-Ups Milestones

Status: Completed
Last updated: 2026-05-18

## M0 - Capture And Routing

Exit criteria:

- The review findings are recorded in durable docs.
- Each finding has a proposed owner lane or deferral.
- The lane is indexed from `docs/workstreams/README.md`.

Primary evidence:

- `docs/workstreams/architecture-review-followups/DESIGN.md`
- `docs/workstreams/architecture-review-followups/TODO.md`
- `docs/workstreams/README.md`

## M1 - First Execution Lane

Exit criteria:

- The first execution lane is opened or explicitly rejected.
- The lane has its own problem statement, non-goals, task ledger, and gates.
- The first executable task is narrow enough to validate independently.

Primary evidence:

- `docs/workstreams/metadata-catalog-commit-atomicity/`

## M2 - Second Execution Lane

Exit criteria:

- The second execution lane is opened or merged into an existing active lane.
- The lane explicitly separates metadata authority from NFO XML preservation
  and provider breadth.

Primary evidence:

- `docs/workstreams/metadata-merge-policy-unification/` or the chosen existing
  lane's updated docs.

## M3 - Remaining Routing

Exit criteria:

- Media Library source-of-truth, Public Client Source Locator redaction, Addon
  side effects, HLS request identity, and hardware diagnostics all have a
  status and owner lane.
- Any required ADR work is named before implementation starts.

Primary evidence:

- DESIGN finding routing table.
- Target workstream TODO files.

## M4 - Closeout

Exit criteria:

- Every finding is assigned, deferred, rejected, or closed.
- `WORKSTREAM.json` status is updated.
- `HANDOFF.md` names the next recommended action or states that none remains.

Primary evidence:

- `docs/workstreams/architecture-review-followups/WORKSTREAM.json`
- `docs/workstreams/architecture-review-followups/HANDOFF.md`
- target workstream docs named in the routing table
