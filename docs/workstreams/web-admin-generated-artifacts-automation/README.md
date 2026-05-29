# Web Admin Generated Artifacts Automation

Status: Active
Last updated: 2026-05-28

This lane brings the Generated Artifacts / Automation operator workflow into
the new `web/` Admin frontend. It is the reentry path for the removed
AI/automation Media sidebar prototypes: generated proposals are Admin-reviewed
artifacts with explicit acceptance boundaries, not free-form Media chat UI.

## Authoritative Docs

- `DESIGN.md` - problem, scope, non-goals, and architecture direction.
- `ROUTE_API_READINESS.md` - Admin API contract inventory and mutation guards.
- `TODO.md` - executable task ledger.
- `EVIDENCE_AND_GATES.md` - validation commands and evidence log.
- `HANDOFF.md` - current state and next action.

## Current Execution Point

`WAGA-020` audited generated Admin proposal/review contracts and added the
`web/` read-model boundary for generated artifact proposal lists. Continue with
`WAGA-030`, the read-only `/admin/automation/generated-artifacts` route.
