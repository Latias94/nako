# Generated Artifact Metadata Authority Apply

Status: Active
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

Current executable task: `GAMA-060`, which should add the Web Admin apply-plan
and confirm-apply workflow. Backend work through `GAMA-050` has shipped the
read-only apply-plan route, host-owned apply execution, durable idempotency
outcomes, the final Admin metadata apply route, and synchronized generated
Admin TypeScript contracts. Planner reconciliation on 2026-06-01 reran focused
GAMA-050 route gates and advanced the active queue to `GAMA-060`.
