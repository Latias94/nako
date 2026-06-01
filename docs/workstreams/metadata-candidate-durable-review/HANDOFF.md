# Metadata Candidate Durable Review - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from provider-depth follow-on selection after TMDB, Bangumi,
and Douban provider precision closeouts. Candidate Graph previews are useful,
but automatic refresh intentionally persists only root Provider Mapping
behavior. `MCDR-020` added a pure, redaction-safe review plan before any schema
or mutation work.

## Active Task

- Task ID: `MCDR-030`
- Owner: planner
- Files: `crates/nako-core`, `crates/nako-db`, and this workstream
- Validation: focused `nako-db` contract tests if schema starts; JSON/JSONL
  validation and `git diff --check` for planner-only refinement
- Status: READY
- Evidence: `docs/workstreams/metadata-candidate-durable-review/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Start with a pure review plan contract before schema.
- `MCDR-020` keeps review planning pure: no repository, schema, Admin/Web,
  Public Client API, Generated Artifact apply, or Provider Mapping mutation.
- Keep durable repository/schema and accept/reject mutation in later tasks.
- Keep Admin/Web provider depth governance split until backend review semantics
  are durable and redaction-safe.
- Do not reuse Generated Artifact apply outcomes as a generic candidate queue.

## Blockers

- None for planner review of `MCDR-030`.

## Next Recommended Action

- Run `MCDR-030`: decide the durable repository/schema boundary for review
  snapshots, including retention, stale-review invalidation, and idempotency
  rules before implementation.
