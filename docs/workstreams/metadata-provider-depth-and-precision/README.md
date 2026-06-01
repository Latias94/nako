# Metadata Provider Depth And Precision

Status: Closed
Last updated: 2026-06-02

This workstream sharpens Nako's built-in metadata provider depth and identity
precision after the Generated Artifact apply lanes closed.

The current provider foundation is useful: provider capabilities, matching
policy, Provider Subjects, Provider Mappings, raw response cache, and candidate
review already exist. The remaining risk is that provider adapters can still
overstate what kind/depth they truly support, and future provider work can
silently turn "some candidate matched" into weak Canonical Metadata or broad
Provider Mapping writes.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `FOLLOW_ONS.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `MPDP-010` opens the lane.
- `MPDP-020` added TMDB series -> season provider graph preview as
  non-mutating evidence.
- `MPDP-030` proved refresh and Provider Mapping persistence remain root-only
  when graph preview nodes are present.
- `MPDP-040` split follow-ons for TMDB episode depth, Bangumi
  relations/episodes, Douban precision, durable candidate review, and
  Admin/Web confirmation in `FOLLOW_ONS.md`.
- `MPDP-050` closed this lane.

Boundary:

- no new provider integration beyond TMDB, Douban, and Bangumi;
- no raw provider payload, token, path, or proxy exposure;
- no Public Client API changes in the first slice;
- no schema migration until durable candidate review is proven necessary;
- no Generated Artifact apply changes in this lane.
