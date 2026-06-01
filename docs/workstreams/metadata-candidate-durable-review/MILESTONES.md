# Metadata Candidate Durable Review - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude schema, Admin/Web, Public Client API, Generated Artifact
  apply, and Provider Mapping mutation in the first slice.

## M1 - Redaction-Safe Review Plan Contract

Status: Complete after `MCDR-020`.

Exit criteria:

- Candidate Graph review plans include root and related Provider Subject
  summaries and relationships;
- review facts exclude raw provider payloads, secrets, proxy URLs, and headers;
- plan generation does not write Provider Subjects or Provider Mappings.

## M2 - Durable Review Repository Shape

Status: Complete after `MCDR-030`.

Exit criteria:

- retention, stale-review invalidation, and idempotency rules are explicit;
- repository/schema contracts are tested before Admin/Web depends on them;
- Generated Artifact apply outcome tables are not reused as a candidate queue.

## M3 - Idempotent Accept/Reject Backend Semantics

Status: Ready after `MCDR-030`.

Exit criteria:

- accept/reject transitions are idempotent;
- accepted Provider Mapping writes, if introduced, go through named backend
  services and do not change automatic refresh behavior;
- stale candidate review decisions cannot mutate the wrong Media Item state.

## M4 - Closeout

Exit criteria:

- fresh evidence is recorded;
- Admin/Web provider depth governance is split if UI work remains;
- architecture maps no longer route active work to a closed lane.
