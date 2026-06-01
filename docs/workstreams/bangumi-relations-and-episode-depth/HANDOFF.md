# Bangumi Relations And Episode Depth - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from the closed MPDP follow-on split after TMDB episode
graph depth closeout. `BRED-020` narrowed Bangumi capability claims to
endpoint-backed subject-level behavior before episode endpoint support exists.
`BRED-030` added endpoint-backed Bangumi episode graph preview for series
fetches. `BRED-040` proved refresh keeps related episode nodes non-mutating.

## Active Task

- Task ID: `BRED-050`
- Owner: planner
- Files: this workstream, architecture links, `docs/GOALS.md`, and `docs/ROADMAP.md`
- Validation: fresh gate evidence, JSON/JSONL validation, and `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/bangumi-relations-and-episode-depth/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Open a new focused lane instead of reopening
  `metadata-provider-depth-and-precision`.
- Start by making Bangumi capability claims truthful before adding graph depth.
- Season/Episode search/fetch now returns `Unsupported` before provider HTTP
  until endpoint-backed behavior is implemented.
- Bangumi series fetch may expose related Episode Provider Subjects from
  `/v0/episodes`, but those nodes remain graph preview evidence.
- Refresh persists only the root Bangumi Provider Subject and Provider Mapping
  from graph preview data.
- Keep durable candidate review, schema changes, Admin/Web confirmation, and
  child Provider Mapping writes out of this lane.

## Blockers

- None for `BRED-050`.

## Next Recommended Action

- Run `BRED-050`: close this focused lane or explicitly split any remaining
  durable candidate review and Admin/Web confirmation follow-ons.
