# TMDB Season Episode Graph Depth — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is newly opened from the closed MPDP follow-on split. The baseline is
TMDB series -> season graph preview with root-only refresh persistence. This
lane extends the same preview-only model to TMDB season -> episode graph depth.

## Active Task

- Task ID: `TSEG-020`
- Owner: codex
- Files: `crates/nako-metadata/src/providers/tmdb.rs`, `crates/nako-metadata/src/mapping/tmdb.rs`, `crates/nako-metadata/src/tests.rs`, and this workstream
- Validation: focused `nako-metadata` TMDB provider / candidate graph gates, plus `cargo fmt --all -- --check`
- Status: READY
- Evidence: `docs/workstreams/tmdb-season-episode-graph-depth/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Use a new focused lane instead of reopening
  `metadata-provider-depth-and-precision`.
- Start with TMDB season -> episode graph preview only.
- Keep durable candidate review, schema changes, Admin/Web confirmation, and
  child Provider Mapping writes out of this lane.

## Blockers

- None for `TSEG-020`.

## Next Recommended Action

- Run `TSEG-020`: implement TMDB season -> episode provider graph preview and
  tests that prove it remains non-mutating evidence.
