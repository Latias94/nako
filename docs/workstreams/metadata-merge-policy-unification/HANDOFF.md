# Metadata Merge Policy Unification Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The workstream is open from ARF-002 / ARF-040. No implementation changes have
started.

The concrete risk is duplicated Canonical Metadata merge authority between
`taru-metadata` and `taru-nfo`. The first implementation step must characterize
current behavior before moving policy code.

## Active Task

- Task ID: MMP-020
- Owner: unassigned
- Files: `crates/taru-metadata`, `crates/taru-nfo`
- Validation: `cargo nextest run -p taru-metadata merge --no-fail-fast`;
  `cargo nextest run -p taru-nfo nfo_service --no-fail-fast`
- Status: READY
- Review: required before accepting completion
- Evidence: `crates/taru-metadata/src/merge.rs`, `crates/taru-nfo/src/import.rs`

## Decisions Since Last Update

- Keep NFO XML preservation out of this lane.
- Keep provider breadth and provider priority configuration out of the first
  slice.
- Characterize behavior before moving the shared policy boundary.
- The likely shared boundary must be usable by both NFO Import and provider
  refresh without creating a dependency cycle.

## Blockers

- None known.

## Next Recommended Action

Run MMP-020: add or identify focused characterization tests for provider
full-refresh, provider missing-only, NFO local-first, NFO remote-first, and
cross-source field locks. Then proceed to MMP-030 only after those tests make
the expected behavior explicit.

