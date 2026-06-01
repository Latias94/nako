# TMDB Season Episode Graph Depth — Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

The lane is newly opened from the closed MPDP follow-on split. The baseline is
TMDB series -> season graph preview with root-only refresh persistence. This
lane extends the same preview-only model to TMDB season -> episode graph depth.
`TSEG-020` shipped season -> episode graph preview. `TSEG-030` added refresh
guard evidence that related episode preview nodes remain non-mutating.

## Closed Task

- Task ID: `TSEG-040`
- Owner: planner
- Files: this workstream, architecture links, `docs/GOALS.md`, and `docs/ROADMAP.md`
- Validation: fresh gate evidence, JSON/JSONL validation, and `git diff --check`
- Status: DONE
- Evidence: `docs/workstreams/tmdb-season-episode-graph-depth/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Use a new focused lane instead of reopening
  `metadata-provider-depth-and-precision`.
- Start with TMDB season -> episode graph preview only.
- Keep durable candidate review, schema changes, Admin/Web confirmation, and
  child Provider Mapping writes out of this lane.

## Blockers

- None for `TSEG-040`.

## Next Recommended Action

- Open a new focused follow-on for Bangumi relations/episode depth, Douban
  subject precision, durable candidate review, or Admin/Web provider depth
  governance. Do not reopen this lane unless TMDB preview semantics change.
