# Web Admin Acquisition Intake

Status: Closed
Last updated: 2026-05-29

This lane brings the completed Acquisition Intake operator workflow into the
new `web/` frontend. It is the reentry path for the old Downloads placeholder:
downloads remain an Admin operation that reads safe intake diagnostics and
Managed Import linkage, not Media client chrome.

## Authoritative Docs

- `DESIGN.md` - problem, scope, non-goals, and architecture direction.
- `ROUTE_API_READINESS.md` - Admin API contract inventory and route readiness.
- `MUTATION_BOUNDARY_DECISION.md` - WAAI-040 watch-folder discovery mutation decision.
- `TODO.md` - executable task ledger.
- `EVIDENCE_AND_GATES.md` - validation commands and evidence log.
- `HANDOFF.md` - current state and next action.
- `CLOSEOUT.md` - final closeout summary and follow-ons.

## Current Execution Point

This lane closed with `WAAI-050` on 2026-05-29. The new `web/` shell now owns
the read-only `/admin/acquisition/intake` Admin route, route query state,
fixture/live Admin API data-source behavior, redaction-sensitive rendering, and
desktop/mobile browser smoke evidence.

Watch-folder discovery mutation controls remain split to a future guarded
mutation lane.
