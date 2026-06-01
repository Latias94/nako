# Metadata Provider Depth And Precision — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is active in the `library-metadata-control-plane` queue.

`MPDP-020` shipped the first vertical slice: TMDB series fetch exposes season
Provider Subjects in the candidate graph. That graph depth is preview evidence
only; it must not create Media Items, child Provider Mappings, schema changes,
Public Client API changes, Web confirmation UI, or Generated Artifact apply
behavior.

## Active Task

- Task ID: `MPDP-030`
- Owner: codex
- Files: `crates/nako-metadata/src/provider_attempt.rs`, `crates/nako-metadata/src/strategy.rs`, `crates/nako-metadata/src/tests.rs`, `crates/nako-server/src/app/tests/metadata.rs`, and this workstream
- Validation: focused `nako-metadata` refresh / candidate graph gates, plus server metadata refresh tests if persistence behavior changes
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

- None for `MPDP-030`.

## Next Recommended Action

- Run `MPDP-030`: prove refresh and Provider Mapping persistence remain
  root-only even when the fetched TMDB graph includes season preview nodes.
