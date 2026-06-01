# Metadata Candidate Durable Review - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from provider-depth follow-on selection after TMDB, Bangumi,
and Douban provider precision closeouts. Candidate Graph previews are useful,
but automatic refresh intentionally persists only root Provider Mapping
behavior. `MCDR-020` added a pure, redaction-safe review plan before any schema
or mutation work. `MCDR-030` added durable review snapshot persistence without
Provider Mapping writes. `MCDR-040` added backend-only idempotent review
decision status transitions without Admin/Web/API or Provider Mapping
application.

## Active Task

- Task ID: `MCDR-050`
- Owner: planner
- Files: this workstream, `docs/architecture`, `docs/GOALS.md`,
  `docs/ROADMAP.md`
- Validation: fresh evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL
  validation; `git diff --check`
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
- `MCDR-040` accepts/rejects durable review status only. It has stale
  `item_id`/`expected_updated_at_ms` guards, marks expired pending reviews, and
  does not write Provider Mapping rows.
- Keep Admin/Web provider depth governance split until backend review semantics
  are durable and redaction-safe.
- Do not reuse Generated Artifact apply outcomes as a generic candidate queue.

## Blockers

- None for closeout review.

## Next Recommended Action

- Run `MCDR-050`: close the lane or split follow-ons for Admin/Web provider
  depth governance and explicit accepted-review Provider Mapping application.
