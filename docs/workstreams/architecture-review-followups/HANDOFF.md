# Architecture Review Follow-Ups Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The 2026-05-18 architecture review findings have been captured in a durable
tracking lane. Routing has been confirmed for all findings.

The highest-priority follow-up, metadata refresh/catalog/search commit
atomicity, was implemented and closed in
`metadata-catalog-commit-atomicity`. The next execution lane is metadata merge
policy unification across NFO import and provider refresh.

## Active Task

- Task ID: ARF-040
- Owner: planner
- Files: `docs/workstreams/architecture-review-followups/*`
- Validation: Open `metadata-merge-policy-unification` or document why it is
  merged into an existing lane.
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
- ARF-002 through ARF-007 are assigned to focused existing or new lanes.
- ARF-008 and ARF-009 remain deferred.

## Blockers

- Public Client Source Locator redaction may need an ADR or public contract
  design step before implementation.

## Next Recommended Action

Open `metadata-merge-policy-unification` as the next execution lane. Keep NFO
XML preservation and provider breadth out of that first slice.
