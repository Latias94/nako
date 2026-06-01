# TMDB Season Episode Graph Depth — Milestones

Status: Closed
Last updated: 2026-06-02

## M0 — Lane Opening

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude schema, Public Client API, Admin/Web, and Generated
  Artifact apply changes.

## M1 — Season Episode Graph Preview

Exit criteria:

- TMDB season fetch exposes related episode Provider Subjects in
  `MetadataCandidateGraph`;
- episode Provider Subject keys use existing TMDB compound key semantics;
- tests prove root season metadata remains compatible.

## M2 — Non-Mutating Season Refresh Boundary

Exit criteria:

- season refresh persists only the root season Provider Subject;
- no episode Media Items, episode Provider Subjects, or child Provider
  Mappings are created from graph preview data;
- raw response caching remains root-keyed.

## M3 — Closeout

Exit criteria:

- fresh evidence is recorded;
- any durable review/Admin/Web follow-ons are split;
- architecture maps no longer route active work to a closed lane.

Result: completed on 2026-06-02. See `CLOSEOUT.md`.
