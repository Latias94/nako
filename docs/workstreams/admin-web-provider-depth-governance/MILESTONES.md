# Admin Web Provider Depth Governance - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

Status: Complete after `AWPDG-010`.

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude Public Client API, raw provider payloads, related-node
  hierarchy application, and Generated Artifact outcome reuse.

## M1 - Read-Only Admin API Review Plan

Status: Complete after `AWPDG-020`.

Exit criteria:

- Admin DTOs expose durable Candidate Review facts and application plan reasons;
- read-only route tests prove no Provider Mapping writes;
- generated Admin contract remains synchronized;
- redaction expectations are explicit.

## M2 - Confirmed Admin API Apply Mutation

Status: Ready after `AWPDG-020`.

Exit criteria:

- mutation calls the backend application service;
- stale guards and idempotency are visible;
- only root Provider Subject / Provider Mapping state can change;
- conflicts and noops are test-visible.

## M3 - Web Admin Governance Surface

Status: Pending after `AWPDG-030`.

Exit criteria:

- Web shows review evidence, plan reasons, and result/noop/conflict facts;
- preview graph nodes are not presented as applied hierarchy;
- route-state, data-source, type-check, test, bundle, and browser smoke gates
  pass.

## M4 - Closeout And Follow-On Split

Status: Pending after `AWPDG-040`.

Exit criteria:

- fresh evidence is recorded;
- follow-ons are split for related-node hierarchy application and provider
  endpoint depth;
- architecture maps route no active work to a closed lane.
