# Architecture Review Follow-Ups TODO

Status: Proposed
Last updated: 2026-05-18

## M0 - Capture And Routing

- [x] ARF-010 [owner=codex] [deps=none] [scope=docs/workstreams/architecture-review-followups]
  Goal: Capture the 2026-05-18 architecture review findings in a durable
  tracking lane.
  Validation: `git diff --check`.
  Evidence: `docs/workstreams/architecture-review-followups/DESIGN.md`.
  Handoff: Continue with ARF-020 before opening implementation lanes.

- [ ] ARF-020 [owner=planner] [deps=ARF-010] [scope=docs/workstreams/architecture-review-followups]
  Goal: Confirm which findings should become execution workstreams and which
  should update existing lanes.
  Validation: DESIGN finding routing table and WORKSTREAM.json agree.
  Evidence: `docs/workstreams/architecture-review-followups/DESIGN.md`.
  Handoff: Open or update the first execution lane after routing is confirmed.

## M1 - First Execution Lane

- [ ] ARF-030 [owner=planner] [deps=ARF-020] [scope=docs/workstreams]
  Goal: Open `metadata-catalog-commit-atomicity` or record why another lane
  should go first.
  Validation: New or updated workstream has DESIGN, TODO, MILESTONES,
  EVIDENCE_AND_GATES, WORKSTREAM, and HANDOFF docs.
  Evidence: `docs/workstreams/metadata-catalog-commit-atomicity/`.
  Handoff: First implementation task should be a narrow proof of consistency
  behavior, not a broad repository rewrite.

## M2 - Second Execution Lane

- [ ] ARF-040 [owner=planner] [deps=ARF-030] [scope=docs/workstreams]
  Goal: Open `metadata-merge-policy-unification` or explicitly merge it into
  another active metadata/NFO lane.
  Validation: Target lane names non-goals for NFO XML preservation and provider
  breadth.
  Evidence: Target lane DESIGN and TODO.
  Handoff: First implementation task should prove one shared merge authority
  path for NFO and provider refresh behavior.

## M3 - Remaining Routing

- [ ] ARF-050 [owner=planner] [deps=ARF-040] [scope=docs/workstreams]
  Goal: Route Media Library source-of-truth, Public Client Source Locator
  redaction, Addon side effects, HLS request identity, and hardware diagnostics
  to existing or new lanes.
  Validation: Every finding in the routing table has status Assigned,
  Deferred, Closed, or Rejected.
  Evidence: DESIGN finding routing table.
  Handoff: Split follow-ons only when the execution lane has its own gate set.

## M4 - Closeout

- [ ] ARF-060 [owner=planner] [deps=ARF-050] [scope=docs/workstreams/architecture-review-followups]
  Goal: Close the tracking lane once all findings are routed.
  Validation: `git diff --check`; all target workstream docs exist or deferral
  reasons are recorded.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Future architecture reviews should open a new review follow-up lane
  or update the assigned execution lanes directly.
