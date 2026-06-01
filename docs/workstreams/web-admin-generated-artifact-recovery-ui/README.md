# Web Admin Generated Artifact Recovery UI

Status: Active
Last updated: 2026-06-02

This workstream turns the closed GAOR read-only recovery queue into a concrete
Web Admin operator surface.

The backend already exposes redaction-safe recovery entries through
`/admin/v1/automation/generated-artifact-apply-recovery`. This lane keeps the
next slice product-facing and read-only: operators should be able to see what
needs repair, what merely proves replay, and which artifact/outcome/batch rows
are involved before any mutation is introduced.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `WAGR-010` opens the follow-on from GAOR closeout evidence.
- `WAGR-020` is the first executable task: add the Web Admin recovery route,
  filters, table, and tests against the existing read-model seam.

Boundary:

- keep the route read-only;
- do not add repair mutation buttons in this lane;
- do not expose raw payloads, prompts, Source Locators, paths, tokens, secrets,
  or idempotency keys;
- preserve Generated Artifact apply and bulk batch terminology from GAOR.
