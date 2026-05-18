# Architecture Review Follow-Ups Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The 2026-05-18 architecture review findings have been captured in a durable
tracking lane. Routing has been confirmed for all findings.

The highest-priority follow-up, metadata refresh/catalog/search commit
atomicity, was implemented and closed in
`metadata-catalog-commit-atomicity`. The second execution lane,
`metadata-merge-policy-unification`, is now open.

## Active Task

- Task ID: ARF-050
- Owner: planner
- Files: `docs/workstreams/architecture-review-followups/*`
- Validation: Remaining assigned/deferred findings have clear target lanes or
  documented split decisions.
- Status: READY
- Review: pending
- Evidence: `docs/workstreams/architecture-review-followups/DESIGN.md`

## Decisions Since Last Update

- This lane is a planning and routing lane, not an execution lane.
- Direct code changes are out of scope for this lane.
- The first recommended execution lane is
  `metadata-catalog-commit-atomicity`.
- The second recommended execution lane is
  `metadata-merge-policy-unification`.
- ARF-001 is closed after `metadata-catalog-commit-atomicity`.
- ARF-002 is open as `metadata-merge-policy-unification`.
- ARF-003 through ARF-007 are assigned to focused existing or new lanes.
- ARF-008 and ARF-009 remain deferred.

## Blockers

- Public Client Source Locator redaction may need an ADR or public contract
  design step before implementation.

## Next Recommended Action

Continue ARF-050: check whether the remaining assigned findings need new
follow-up lanes or only updates to existing workstreams. The next implementation
action outside this routing lane is MMP-020 in
`metadata-merge-policy-unification`.
