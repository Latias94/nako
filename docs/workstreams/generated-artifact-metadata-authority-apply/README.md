# Generated Artifact Metadata Authority Apply

Status: Active
Last updated: 2026-05-29

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
- `HANDOFF.md`

First executable task: `GAMA-020`, which should add a backend metadata apply
plan contract/read model for accepted metadata Generated Artifacts. It must not
mutate Canonical Metadata.
