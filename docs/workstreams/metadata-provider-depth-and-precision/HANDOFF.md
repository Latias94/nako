# Metadata Provider Depth And Precision — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is newly opened after Generated Artifact apply repair closeout and is
registered as the active `library-metadata-control-plane` queue.

Read-only metadata recon selected the first vertical slice: TMDB series fetch
should expose season Provider Subjects in the candidate graph. That graph depth
is preview evidence only; it must not create Media Items, child Provider
Mappings, schema changes, Public Client API changes, Web confirmation UI, or
Generated Artifact apply behavior.

## Active Task

- Task ID: `MPDP-020`
- Owner: codex
- Files: `crates/nako-core/src/media/candidate.rs`, `crates/nako-metadata/src/providers/tmdb.rs`, `crates/nako-metadata/src/mapping/tmdb.rs`, `crates/nako-metadata/src/tests.rs`, and this workstream
- Validation: focused `nako-metadata` TMDB provider / candidate graph gates, plus `cargo fmt --all -- --check` when Rust changes
- Status: READY
- Evidence: `docs/workstreams/metadata-provider-depth-and-precision/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Use `metadata-provider-depth-and-precision` rather than reopening
  `metadata-provider-breadth` or Generated Artifact lanes.
- Start with TMDB series -> season graph preview before Admin/Web
  confirmation.
- Keep durable candidate review, schema changes, and child Provider Mapping
  writes out of this lane unless explicit follow-on evidence justifies them.

## Blockers

- None for `MPDP-020`.

## Next Recommended Action

- Run `MPDP-020`: implement the TMDB series -> season provider graph preview
  and tests that prove it remains non-mutating evidence.
