# TMDB Season Episode Graph Depth — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is newly opened from the closed MPDP follow-on split. The baseline is
TMDB series -> season graph preview with root-only refresh persistence. This
lane extends the same preview-only model to TMDB season -> episode graph depth.
`TSEG-020` shipped season -> episode graph preview.

## Active Task

- Task ID: `TSEG-030`
- Owner: codex
- Files: `crates/nako-metadata/src/tests.rs` and this workstream
- Validation: focused `nako-metadata` refresh / candidate graph gates, plus `cargo fmt --all -- --check`
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

- Run `TSEG-030`: prove season refresh persists only the root season Provider
  Subject and ignores related episode preview nodes.
