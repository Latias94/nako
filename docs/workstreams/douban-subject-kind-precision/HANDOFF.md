# Douban Subject Kind Precision - Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

The lane is opened from the closed MPDP follow-on split after Bangumi provider
depth closeout. Current Douban code searches and fetches movie subjects, but
capability claims now match the endpoint-backed movie search/detail contract.
`DSKP-020` prevents Series, Season, and Episode requests from reaching Douban
movie endpoints.

## Active Task

- Task ID: none
- Owner: none
- Status: DONE
- Evidence: `docs/workstreams/douban-subject-kind-precision/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Open a new focused lane instead of reopening
  `metadata-provider-depth-and-precision`.
- Start by making Douban capability claims truthful before any future TV or
  episode breadth.
- Keep current Douban behavior movie-endpoint-backed; future TV/episode support
  needs a separate endpoint-backed lane.
- Keep durable candidate review, schema changes, Admin/Web confirmation, graph
  preview, and child Provider Mapping writes out of this lane.

## Blockers

- None.

## Next Recommended Action

- Open the next focused provider-depth follow-on. Recommended candidates are
  durable candidate review, Admin/Web provider depth governance, or future
  Douban TV/episode endpoint depth.
