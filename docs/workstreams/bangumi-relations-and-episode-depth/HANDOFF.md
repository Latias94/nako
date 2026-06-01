# Bangumi Relations And Episode Depth - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from the closed MPDP follow-on split after TMDB episode
graph depth closeout. Current Bangumi code searches and fetches subjects, but
capabilities overclaim Season/Episode support before endpoint-backed behavior
exists.

## Active Task

- Task ID: `BRED-020`
- Owner: codex
- Files: `crates/nako-metadata/src/providers/bangumi.rs`,
  `crates/nako-metadata/src/tests.rs`, and this workstream
- Validation: focused `nako-metadata` Bangumi / candidate graph gates, plus
  `cargo fmt --all -- --check`
- Status: READY
- Evidence: `docs/workstreams/bangumi-relations-and-episode-depth/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Open a new focused lane instead of reopening
  `metadata-provider-depth-and-precision`.
- Start by making Bangumi capability claims truthful before adding graph depth.
- Keep durable candidate review, schema changes, Admin/Web confirmation, and
  child Provider Mapping writes out of this lane.

## Blockers

- None for `BRED-020`.

## Next Recommended Action

- Run `BRED-020`: narrow Bangumi capabilities and add regression coverage for
  unsupported Season/Episode behavior before endpoint-backed episode support is
  implemented.
