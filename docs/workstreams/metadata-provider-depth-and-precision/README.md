# Metadata Provider Depth And Precision

Status: Active
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
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `MPDP-010` opens the lane.
- `MPDP-020` is the first executable task: add a TMDB series -> season
  provider graph preview while proving it remains non-mutating evidence.

Boundary:

- no new provider integration beyond TMDB, Douban, and Bangumi;
- no raw provider payload, token, path, or proxy exposure;
- no Public Client API changes in the first slice;
- no schema migration until durable candidate review is proven necessary;
- no Generated Artifact apply changes in this lane.
