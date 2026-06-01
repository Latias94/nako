# Bangumi Relations And Episode Depth - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude schema, Public Client API, Admin/Web, and Generated
  Artifact apply changes.

## M1 - Endpoint-Backed Capability Claims

Exit criteria:

- Bangumi capabilities no longer claim Season/Episode support unless the
  adapter has endpoint-backed behavior for those kinds;
- unsupported search/fetch requests fail explicitly;
- tests cover capability claims and unsupported kinds.

## M2 - Episode Graph Preview

Exit criteria:

- Bangumi series fetch can expose related episode Provider Subjects from
  endpoint-backed episode data;
- relationships use Candidate Graph preview semantics;
- root series metadata remains compatible.

## M3 - Non-Mutating Refresh Boundary

Exit criteria:

- refresh persists only the root Bangumi Provider Subject;
- no episode Media Items, episode Provider Subjects, or child Provider
  Mappings are created from graph preview data;
- raw response caching remains root-keyed.

## M4 - Closeout

Exit criteria:

- fresh evidence is recorded;
- durable review/Admin/Web follow-ons are split;
- architecture maps no longer route active work to a closed lane.
