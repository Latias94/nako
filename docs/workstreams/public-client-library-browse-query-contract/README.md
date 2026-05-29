# Public Client Library Browse Query Contract

Status: Active
Last updated: 2026-05-28

This lane fixes the WMLP browse follow-ons: the new `web/` Media surface can
read libraries and broad catalog items, but Public Client lacks library-scoped
item browse and stable catalog sort/filter contracts for live library pages and
Recently Added-style rails.

## Authoritative Docs

- `DESIGN.md` - problem, scope, non-goals, and architecture direction.
- `CONTRACT.md` - frozen Public Client route/query/DTO contract.
- `CONTRACT_READINESS.md` - route/query contract choices.
- `TODO.md` - executable task ledger.
- `EVIDENCE_AND_GATES.md` - validation commands and evidence log.
- `HANDOFF.md` - current state and next action.

## Current Execution Point

`PLBQ-020` froze the library browse/query contract. Continue with `PLBQ-030`,
the server/API/SDK implementation slice.
