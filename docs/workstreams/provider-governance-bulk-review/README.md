# Provider Governance Bulk Review

Status: Active
Last updated: 2026-06-02

This workstream turns the shipped Metadata Candidate Review queue and direct
apply surface into a guarded batch governance workflow.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `TASKS.jsonl`
- `CAMPAIGNS.jsonl`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`

Current execution:

- `PGBR-010` opens the lane.
- `PGBR-020` shipped the read-only Admin API batch application plan for
  selected Metadata Candidate Reviews.
- `PGBR-030` shipped bounded confirmed backend batch apply through the
  existing single-review application authority.
- `PGBR-040` is ready: add Web Admin selection, plan inspection,
  confirmation, and partial-result rendering.

Boundary:

- batch governance must reuse existing single-review stale guard and
  idempotency semantics;
- read-only planning and bounded backend confirmation are already shipped;
- no Public Client API expansion;
- no related Provider Subject, child Provider Mapping, or Media Item hierarchy
  mutation;
- no raw provider payloads, secrets, proxy URLs, headers, paths, source
  fingerprints, provider bodies, or raw idempotency keys.
