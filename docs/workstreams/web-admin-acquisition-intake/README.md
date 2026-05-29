# Web Admin Acquisition Intake

Status: Active
Last updated: 2026-05-29

This lane brings the completed Acquisition Intake operator workflow into the
new `web/` frontend. It is the reentry path for the old Downloads placeholder:
downloads remain an Admin operation that reads safe intake diagnostics and
Managed Import linkage, not Media client chrome.

## Authoritative Docs

- `DESIGN.md` - problem, scope, non-goals, and architecture direction.
- `ROUTE_API_READINESS.md` - Admin API contract inventory and route readiness.
- `TODO.md` - executable task ledger.
- `EVIDENCE_AND_GATES.md` - validation commands and evidence log.
- `HANDOFF.md` - current state and next action.

## Current Execution Point

`WAAI-030` implemented the route-first `/admin/acquisition/intake` page in the
new `web/` shell. Continue with `WAAI-040`, the mutation boundary decision for
watch-folder discovery.
