# Bangumi Relations And Episode Depth - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from the closed MPDP follow-on split after TMDB episode
graph depth closeout. `BRED-020` narrowed Bangumi capability claims to
endpoint-backed subject-level behavior before episode endpoint support exists.
`BRED-030` added endpoint-backed Bangumi episode graph preview for series
fetches.

## Active Task

- Task ID: `BRED-040`
- Owner: codex
- Files: `crates/nako-metadata/src/tests.rs` and this workstream
- Validation: focused `nako-metadata` Bangumi / candidate graph gates, plus
  `cargo fmt --all -- --check`
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
- Keep durable candidate review, schema changes, Admin/Web confirmation, and
  child Provider Mapping writes out of this lane.

## Blockers

- None for `BRED-040`.

## Next Recommended Action

- Run `BRED-040`: prove refresh persists only the root Bangumi Provider
  Subject and ignores related episode preview nodes.
