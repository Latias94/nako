# Metadata Candidate Durable Review - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from provider-depth follow-on selection after TMDB, Bangumi,
and Douban provider precision closeouts. Candidate Graph previews are useful,
but automatic refresh intentionally persists only root Provider Mapping
behavior. `MCDR-020` added a pure, redaction-safe review plan before any schema
or mutation work. `MCDR-030` added durable review snapshot persistence without
Provider Mapping writes.

## Active Task

- Task ID: `MCDR-040`
- Owner: codex
- Files: `crates/nako-core`, `crates/nako-metadata`, `crates/nako-db`, and this
  workstream
- Validation: focused `nako-metadata` and `nako-db` gates; `cargo fmt --all
  -- --check`; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/metadata-candidate-durable-review/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Start with a pure review plan contract before schema.
- `MCDR-020` keeps review planning pure: no repository, schema, Admin/Web,
  Public Client API, Generated Artifact apply, or Provider Mapping mutation.
- `MCDR-030` stores only `MetadataCandidateReviewPlan` JSON in
  `metadata_candidate_reviews`; it does not store raw provider payloads.
- Candidate review snapshots are idempotent by `item_id`, source, and
  `source_key`.
- Keep accept/reject mutation in `MCDR-040`.
- Keep Admin/Web provider depth governance split until backend review semantics
  are durable and redaction-safe.
- Do not reuse Generated Artifact apply outcomes as a generic candidate queue.

## Blockers

- None for planner review of `MCDR-030`.

## Next Recommended Action

- Run `MCDR-040`: add backend-only idempotent accept/reject transitions for
  durable candidate reviews, then decide whether accepted reviews create
  `ProviderMappingStatus` records through a named service.
