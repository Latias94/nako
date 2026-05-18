# Architecture Review Follow-Ups TODO

Status: Completed
Last updated: 2026-05-18

## M0 - Capture And Routing

- [x] ARF-010 [owner=codex] [deps=none] [scope=docs/workstreams/architecture-review-followups]
  Goal: Capture the 2026-05-18 architecture review findings in a durable
  tracking lane.
  Validation: `git diff --check`.
  Evidence: `docs/workstreams/architecture-review-followups/DESIGN.md`.
  Handoff: Continue with ARF-020 before opening implementation lanes.

- [x] ARF-020 [owner=codex] [deps=ARF-010] [scope=docs/workstreams/architecture-review-followups]
  Goal: Confirm which findings should become execution workstreams and which
  should update existing lanes.
  Validation: DESIGN finding routing table and WORKSTREAM.json agree.
  Evidence: `docs/workstreams/architecture-review-followups/DESIGN.md`.
  Handoff: `metadata-catalog-commit-atomicity` was opened and closed. Continue
  with ARF-040 to open `metadata-merge-policy-unification`.

## M1 - First Execution Lane

- [x] ARF-030 [owner=codex] [deps=ARF-020] [scope=docs/workstreams]
  Goal: Open `metadata-catalog-commit-atomicity` or record why another lane
  should go first.
  Validation: New or updated workstream has DESIGN, TODO, MILESTONES,
  EVIDENCE_AND_GATES, WORKSTREAM, and HANDOFF docs.
  Evidence: `docs/workstreams/metadata-catalog-commit-atomicity/`.
  Handoff: Lane is completed. ARF-001 is closed.

## M2 - Second Execution Lane

- [x] ARF-040 [owner=codex] [deps=ARF-030] [scope=docs/workstreams]
  Goal: Open `metadata-merge-policy-unification` or explicitly merge it into
  another active metadata/NFO lane.
  Validation: Target lane names non-goals for NFO XML preservation and provider
  breadth.
  Evidence: Target lane DESIGN and TODO.
  Handoff: `metadata-merge-policy-unification` was opened and later closed
  after unifying the merge policy boundary in `taru-core`.

## M3 - Remaining Routing

- [x] ARF-050 [owner=codex] [deps=ARF-040] [scope=docs/workstreams]
  Goal: Route Media Library source-of-truth, Public Client Source Locator
  redaction, Addon side effects, HLS request identity, and hardware diagnostics
  to existing or new lanes.
  Validation: Every finding in the routing table has status Assigned,
  Deferred, Closed, or Rejected.
  Evidence: DESIGN finding routing table.
  Handoff: `multi-library-hardening` and
  `public-client-source-locator-redaction` are opened/promoted as focused
  lanes. Addon and playback/transcode follow-ups are recorded in existing
  lanes. Continue with ARF-060 closeout after review and verification.

## M4 - Closeout

- [x] ARF-060 [owner=codex] [deps=ARF-050] [scope=docs/workstreams/architecture-review-followups]
  Goal: Close the tracking lane once all findings are routed.
  Validation: `git diff --check`; all target workstream docs exist or deferral
  reasons are recorded.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Lane closed. Next implementation action is MLH-020 in
  `multi-library-hardening`.
