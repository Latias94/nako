# Web Admin Generated Artifact Recovery UI

Status: Closed
Last updated: 2026-06-02

This workstream turned the closed GAOR read-only recovery queue into a concrete
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

Closed execution:

- `WAGR-010` opens the follow-on from GAOR closeout evidence.
- `WAGR-020` added the Web Admin recovery route, filters, table, route state,
  and tests against the existing read-model seam.
- `WAGR-030` closed the lane after desktop/mobile browser smoke and Web gates.

Boundary:

- keep the route read-only;
- do not add repair mutation buttons in this lane;
- do not expose raw payloads, prompts, Source Locators, paths, tokens, secrets,
  or idempotency keys;
- preserve Generated Artifact apply and bulk batch terminology from GAOR.
