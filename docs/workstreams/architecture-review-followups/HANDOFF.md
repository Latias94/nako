# Architecture Review Follow-Ups Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The 2026-05-18 architecture review findings have been captured in a durable
tracking lane. No implementation work has started.

The highest-priority proposed follow-up is metadata refresh, catalog graph, and
search projection atomicity. The second proposed follow-up is metadata merge
policy unification across NFO import and provider refresh.

## Active Task

- Task ID: ARF-020
- Owner: planner
- Files: `docs/workstreams/architecture-review-followups/*`
- Validation: DESIGN finding routing table and WORKSTREAM.json agree.
- Status: NEEDS_CONTEXT
- Review: pending
- Evidence: `docs/workstreams/architecture-review-followups/DESIGN.md`

## Decisions Since Last Update

- This lane is a planning and routing lane, not an execution lane.
- Direct code changes are out of scope for this lane.
- The first recommended execution lane is
  `metadata-catalog-commit-atomicity`.
- The second recommended execution lane is
  `metadata-merge-policy-unification`.

## Blockers

- User confirmation is needed before opening the first execution lane.
- Public Client Source Locator redaction may need an ADR or public contract
  design step before implementation.

## Next Recommended Action

Confirm routing, then open `metadata-catalog-commit-atomicity` with a narrow
first task focused on consistency behavior and validation evidence.
