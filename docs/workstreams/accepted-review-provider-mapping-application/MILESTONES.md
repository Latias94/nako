# Accepted Review Provider Mapping Application - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

Status: Complete after `ARPMA-010`.

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude Admin/Web, Public Client API, refresh side effects, related
  graph node hierarchy application, and Generated Artifact executor changes.

## M1 - Read-Only Application Plan

Status: Complete after `ARPMA-020`.

Exit criteria:

- accepted review application actions and reasons are explicit;
- unsupported source conversion is visible as a plan reason;
- planning does not write Provider Subjects or Provider Mappings;
- existing accepted/rejected mapping behavior is tested.

## M2 - Root Provider Mapping Apply Service

Status: Complete after `ARPMA-030`.

Exit criteria:

- accepted review application is idempotent;
- only the root Provider Subject and root Provider Mapping can be applied;
- related review nodes and relationships remain preview evidence;
- rejected mappings are not silently overwritten.

## M3 - Surface Split Review

Status: Ready after `ARPMA-030`.

Exit criteria:

- Admin API/Web mutation scope is either explicitly accepted here or split;
- generated client and Web scopes are not entered accidentally;
- architecture maps agree with the next lane.

## M4 - Closeout

Status: Pending after `ARPMA-040`.

Exit criteria:

- fresh evidence is recorded;
- follow-ons are split for Admin/Web governance and related-node hierarchy
  application;
- architecture maps no longer route active work to a closed lane.
