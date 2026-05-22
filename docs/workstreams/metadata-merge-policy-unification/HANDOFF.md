# Metadata Merge Policy Unification Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

The workstream is open from ARF-002 / ARF-040. MMP-020, MMP-030, and MMP-040
are complete. MMP-050 closes the lane.

The concrete risk is duplicated Canonical Metadata merge authority between
`nako-metadata` and `nako-nfo`. The shared merge boundary now lives in
`nako-core`.

## Active Task

- Task ID: none
- Owner: planner
- Files: `docs/workstreams/metadata-merge-policy-unification`
- Validation: closeout evidence recorded in `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: no blocking findings
- Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`

## Decisions Since Last Update

- Keep NFO XML preservation out of this lane.
- Keep provider breadth and provider priority configuration out of the first
  slice.
- Characterize behavior before moving the shared policy boundary.
- The likely shared boundary must be usable by both NFO Import and provider
  refresh without creating a dependency cycle.
- Source-aware field locks are part of the behavior contract: a source should
  be able to refresh its own locked fields while respecting locks written by
  other sources.
- The shared policy belongs in `nako-core` because it is pure Canonical
  Metadata authority logic and avoids making NFO depend on metadata workflow
  implementation details.
- Provider refresh intentionally still respects all locked fields; hierarchy
  confirmation and NFO import use source-aware lock scopes.

## Blockers

- None known.

## Next Recommended Action

Stop this lane. Reopen only if provider priority configuration, merge
diagnostics, or new Canonical Metadata fields require a focused follow-up.
