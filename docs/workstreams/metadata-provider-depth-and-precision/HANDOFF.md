# Metadata Provider Depth And Precision — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is active in the `library-metadata-control-plane` queue.

`MPDP-020` shipped the first vertical slice: TMDB series fetch exposes season
Provider Subjects in the candidate graph. `MPDP-030` proved refresh persists
only the root Provider Subject/Mapping and does not create child Media Items or
child Provider Subjects from graph preview data.

## Active Task

- Task ID: `MPDP-040`
- Owner: planner
- Files: this workstream, `docs/architecture`, `docs/GOALS.md`, and `docs/ROADMAP.md`
- Validation: JSON/JSONL validation and `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/metadata-provider-depth-and-precision/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Use `metadata-provider-depth-and-precision` rather than reopening
  `metadata-provider-breadth` or Generated Artifact lanes.
- Start with TMDB series -> season graph preview before Admin/Web
  confirmation. Completed in `MPDP-020`.
- Keep durable candidate review, schema changes, and child Provider Mapping
  writes out of this lane unless explicit follow-on evidence justifies them.

## Blockers

- None for `MPDP-040`.

## Next Recommended Action

- Run `MPDP-040`: split follow-ons for TMDB episode graph depth, Bangumi
  relations/episodes, Douban subject precision, durable candidate review, and
  Admin/Web confirmation.
