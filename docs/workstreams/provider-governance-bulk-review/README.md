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
- `PGBR-020` is ready: add a read-only Admin API batch application plan for
  selected Metadata Candidate Reviews.

Boundary:

- batch governance must reuse existing single-review stale guard and
  idempotency semantics;
- the first executable slice is read-only;
- no Public Client API expansion;
- no related Provider Subject, child Provider Mapping, or Media Item hierarchy
  mutation;
- no raw provider payloads, secrets, proxy URLs, headers, paths, source
  fingerprints, provider bodies, or raw idempotency keys.
