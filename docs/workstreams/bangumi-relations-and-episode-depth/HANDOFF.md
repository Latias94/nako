# Bangumi Relations And Episode Depth - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from the closed MPDP follow-on split after TMDB episode
graph depth closeout. `BRED-020` narrowed Bangumi capability claims to
endpoint-backed subject-level behavior before episode endpoint support exists.

## Active Task

- Task ID: `BRED-030`
- Owner: codex
- Files: `crates/nako-metadata/src/providers/bangumi.rs`,
  `crates/nako-metadata/src/mapping/bangumi.rs`,
  `crates/nako-metadata/src/tests.rs`, and this workstream
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
- Keep durable candidate review, schema changes, Admin/Web confirmation, and
  child Provider Mapping writes out of this lane.

## Blockers

- None for `BRED-030`.

## Next Recommended Action

- Run `BRED-030`: add endpoint-backed Bangumi episode graph preview for series
  fetches while keeping related nodes preview-only and non-mutating.
