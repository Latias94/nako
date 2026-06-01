# Admin Web Provider Depth Governance

Status: Active
Last updated: 2026-06-02

This workstream exposes durable Metadata Candidate Review evidence and accepted
review application through Admin API/Web without weakening the backend
root-only Provider Mapping boundary shipped by
`accepted-review-provider-mapping-application`.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `AWPDG-010` opens the lane.
- `AWPDG-020` is ready: add a read-only Admin API surface for durable Candidate
  Review detail and application plan evidence.

Boundary:

- no Public Client API expansion;
- no raw provider payload, token, header, proxy URL, local path, or provider
  body exposure;
- no related graph node hierarchy mutation in the first surface slices;
- no reuse of Generated Artifact apply outcomes as Candidate Review state;
- no blind mutation without stale guards and an explicit idempotency key.
