# Generated Artifact Metadata Authority Apply

Status: Closed
Last updated: 2026-06-01

This workstream turns an accepted `Generated Artifact` metadata proposal into a
bounded `Metadata Authority` apply workflow without letting automation mutate
Canonical Metadata implicitly.

The current backend already has the first guardrail: Generated Artifact review
can accept or reject a proposal, and the accept plan explicitly stages
`MetadataAuthorityApplyRequired` instead of applying immediately. The missing
piece is the next host-owned step: a redacted apply plan, field-lock-aware
diffs, idempotent/audited apply, and a Web Admin confirmation surface that makes
the authority boundary visible.

Authoritative docs:

- `DESIGN.md`
- `APPLY_AUTHORITY_AUDIT.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`
- `CLOSEOUT.md`

Closeout: `GAMA-070` closed the lane on 2026-06-01 after fresh backend/Web
verification. The shipped workflow has a read-only apply-plan route, host-owned
field-lock-aware apply execution, durable idempotency outcomes, a final Admin
metadata apply route, synchronized generated Admin TypeScript contracts, and a
separate Web Admin confirmation surface. Bulk apply, provider-specific mapping
breadth, operations repair tooling, and API-backed restoration of placeholder
Admin settings pages are split to follow-ons.
