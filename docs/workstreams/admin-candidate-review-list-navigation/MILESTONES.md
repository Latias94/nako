# Admin Candidate Review List Navigation - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

Status: Complete after `ACRN-010`.

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude Public Client API, raw provider payloads, related hierarchy
  application, global queues, and batch governance.

## M1 - Item-Scoped Admin API List

Status: Complete after `ACRN-020`.

Exit criteria:

- Admin DTOs expose item-scoped Candidate Review list entries and pagination;
- route tests prove no Provider Subject, Provider Mapping, Canonical Metadata,
  or related hierarchy writes;
- generated Admin contract remains synchronized;
- redaction expectations are explicit.

## M2 - Web Admin List And Navigation

Status: Ready at `ACRN-030`.

Exit criteria:

- Web shows item-scoped Candidate Review rows with review status, source, root
  summary, application action, and safe navigation to the existing detail/apply
  route;
- route-state and data-source tests prove item_id and review_id transitions;
- type-check, tests, bundle, and browser smoke gates pass.

## M3 - Closeout And Follow-On Split

Status: Pending `ACRN-030`.

Exit criteria:

- fresh evidence is recorded;
- global queue/search, batch governance, and hierarchy application follow-ons
  are split or deferred;
- architecture maps route no active work to a closed lane.
