# Metadata Provider Depth And Precision — Milestones

Status: Closed
Last updated: 2026-06-02

## M0 — Lane Opening

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude Generated Artifact apply repair and Public Client API
  changes.

## M1 — TMDB Series Season Graph Preview

Exit criteria:

- TMDB series fetch exposes related season Provider Subjects in
  `MetadataCandidateGraph`;
- parent-child graph relationships use existing TMDB season compound key
  semantics;
- tests prove graph depth is preview evidence and does not require schema,
  Public Client API, Web confirmation, or Generated Artifact apply changes.

## M2 — Non-Mutating Refresh Boundary

Exit criteria:

- refresh and provider mapping flows continue to persist only root Provider
  Mapping behavior;
- no child Media Items or child Provider Mappings are created from the graph
  preview;
- follow-ons are split for TMDB episode depth, Douban subject precision,
  Bangumi relations/episodes, and durable candidate review if needed.

## M3 — Follow-On Split

Exit criteria:

- TMDB episode graph depth, Douban subject precision, Bangumi
  relations/episodes, durable candidate review, and Admin/Web confirmation are
  each either explicitly deferred or opened as separate lanes;
- ambiguous search remains non-mutating;
- provider graph preview remains separate from Generated Artifact apply repair.

## M4 — Closeout

Exit criteria:

- fresh evidence is recorded;
- follow-ons for durable candidate review, Admin governance detail, or Web
  confirmation are split;
- architecture maps no longer route active work to a closed lane.

Status: complete after `MPDP-050`.
