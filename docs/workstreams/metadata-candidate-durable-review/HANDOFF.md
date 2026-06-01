# Metadata Candidate Durable Review - Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

The lane was opened from provider-depth follow-on selection after TMDB, Bangumi,
and Douban provider precision closeouts. Candidate Graph previews are useful,
but automatic refresh intentionally persists only root Provider Mapping
behavior. `MCDR-020` added a pure, redaction-safe review plan before any schema
or mutation work. `MCDR-030` added durable review snapshot persistence without
Provider Mapping writes. `MCDR-040` added backend-only idempotent review
decision status transitions without Admin/Web/API or Provider Mapping
application. `MCDR-050` closed the lane and split the remaining product and
application work into follow-ons.

## Closed State

- Closed task: `MCDR-050`
- Status: DONE
- Evidence: `docs/workstreams/metadata-candidate-durable-review/EVIDENCE_AND_GATES.md`
  and `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md`

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

## Follow-Ons

- `proposed:admin-web-provider-depth-governance`
- `proposed:accepted-review-provider-mapping-application`
- `proposed:douban-tv-episode-endpoint-depth`

## Blockers

- None for closeout review.

## Next Recommended Action

- Select one focused follow-on. Do not reopen this lane for Admin/Web routes or
  Provider Mapping application behavior.
