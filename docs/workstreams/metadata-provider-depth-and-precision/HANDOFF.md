# Metadata Provider Depth And Precision — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is active in the `library-metadata-control-plane` queue.

`MPDP-020` shipped the first vertical slice: TMDB series fetch exposes season
Provider Subjects in the candidate graph. `MPDP-030` proved refresh persists
only the root Provider Subject/Mapping and does not create child Media Items or
child Provider Subjects from graph preview data. `MPDP-040` split remaining
depth work into proposed lanes in `FOLLOW_ONS.md`.

## Active Task

- Task ID: `MPDP-050`
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

- None for `MPDP-050`.

## Next Recommended Action

- Run `MPDP-050`: close the lane after validating the evidence and proposed
  follow-on routing.
