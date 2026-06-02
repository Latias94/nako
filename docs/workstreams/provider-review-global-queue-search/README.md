# Provider Review Global Queue Search

Status: Active
Last updated: 2026-06-02

This workstream turns durable Metadata Candidate Reviews from item-scoped
navigation into an operator-facing global Admin queue/search surface.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `PRGQ-010` opens the lane.
- `PRGQ-020` is ready: add a read-only Admin API global Candidate Review queue
  route and repository query contract.

Boundary:

- global queue/search is read-only until a later lane owns bulk governance;
- no Public Client API expansion;
- no related hierarchy application;
- no batch accept/apply;
- no raw provider payload, path, token, header, proxy URL, source fingerprint,
  or raw idempotency-key exposure.
