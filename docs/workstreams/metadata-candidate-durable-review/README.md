# Metadata Candidate Durable Review

Status: Active
Last updated: 2026-06-02

This workstream turns provider Candidate Graph previews into a durable review
lane without making preview evidence an implicit Provider Mapping mutation.

TMDB and Bangumi can now expose related season/episode Provider Subjects in
Candidate Graphs, while refresh deliberately persists only the root accepted
Provider Mapping. This lane defines the review contract that can preserve those
previews for an operator before Admin/Web governance depends on them.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `MCDR-010` opens the lane.
- `MCDR-020` is ready: define a redaction-safe Metadata Candidate Review plan
  contract from `MetadataCandidateGraph` without schema or Provider Mapping
  writes.

Boundary:

- no schema migration in the first slice;
- no Admin API, Public Client API, or Web route in the first slice;
- no Generated Artifact apply outcome table reuse;
- no accepted Provider Mapping writes from preview graph nodes;
- no raw provider response, token, proxy URL, header, path, or provider body
  exposure.
