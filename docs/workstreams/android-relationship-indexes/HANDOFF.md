# Android Relationship Indexes - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane was split from APICI-060 after Android API Contract Integration
completed the Person Detail route and smoke proof. Android still lacks
first-class People, Tags, and Genres index pages; this lane decides and
implements the accepted subset.

## Active Task

- Task ID: ARI-010
- Owner: planner
- Files:
  - `docs/workstreams/android-relationship-indexes/`
  - `docs/workstreams/android-api-contract-integration/API_INTEGRATION_MATRIX.md`
- Validation:
  - `Get-Content -LiteralPath 'docs/workstreams/android-relationship-indexes/WORKSTREAM.json' -Raw | ConvertFrom-Json | Out-Null`
- Status: NEEDS_CONTEXT
- Review: pending
- Evidence: this workstream scaffold

## Decisions Since Last Update

- Person Detail belongs to the completed API contract lane.
- People, Tags, and Genres indexes are a separate product navigation lane.
- Existing related-items routes should be reused; no local filtering.

## Blockers

- ARI-010 must decide whether People, Tags, and Genres are all in the initial
  Android product slice or whether some are deferred.

## Next Recommended Action

- Execute ARI-010, starting from the API integration matrix and current Android
  browse shell IA.
