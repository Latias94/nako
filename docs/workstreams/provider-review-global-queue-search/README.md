# Provider Review Global Queue Search

Status: Closed
Last updated: 2026-06-02

This workstream turns durable Metadata Candidate Reviews from item-scoped
navigation into an operator-facing global Admin queue/filter surface.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `PRGQ-010` opened the lane.
- `PRGQ-020` shipped the read-only Admin API global Candidate Review queue
  route and repository query contract.
- `PRGQ-030` shipped Web Admin global queue navigation into the existing
  detail/apply page.
- `PRGQ-040` closes the lane and keeps batch governance, related hierarchy
  application, and provider endpoint depth split as follow-ons.

Boundary:

- global queue/filter is read-only until a later lane owns bulk governance;
- no Public Client API expansion;
- no related hierarchy application;
- no batch accept/apply;
- no raw provider payload, path, token, header, proxy URL, source fingerprint,
  or raw idempotency-key exposure.
