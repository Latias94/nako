# Metadata Candidate Durable Review - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from provider-depth follow-on selection after TMDB, Bangumi,
and Douban provider precision closeouts. Candidate Graph previews are useful,
but automatic refresh intentionally persists only root Provider Mapping
behavior. The first task defines a pure, redaction-safe review plan before any
schema or mutation work.

## Active Task

- Task ID: `MCDR-020`
- Owner: codex
- Files: `crates/nako-core/src/media/candidate.rs`,
  `crates/nako-metadata/src`, `crates/nako-metadata/src/tests.rs`, and this
  workstream
- Validation: focused `nako-metadata` candidate review / candidate graph gates,
  plus `cargo fmt --all -- --check`
- Status: READY
- Evidence: `docs/workstreams/metadata-candidate-durable-review/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Start with a pure review plan contract before schema.
- Keep durable repository/schema and accept/reject mutation in later tasks.
- Keep Admin/Web provider depth governance split until backend review semantics
  are durable and redaction-safe.
- Do not reuse Generated Artifact apply outcomes as a generic candidate queue.

## Blockers

- None for `MCDR-020`.

## Next Recommended Action

- Run `MCDR-020`: define a redaction-safe review plan from
  `MetadataCandidateGraph` that captures root/related Provider Subject facts and
  relationships without raw provider payloads or Provider Mapping writes.
