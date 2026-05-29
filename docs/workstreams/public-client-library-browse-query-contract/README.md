# Public Client Library Browse Query Contract

Status: Completed
Last updated: 2026-05-29

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
- `CLOSEOUT.md` - final closeout summary and follow-ons.

## Current Execution Point

Closed by `PLBQ-050`. The first Public Client library-scoped browse contract is
implemented through server/API/SDK/web and remaining facet/pagination breadth is
split into follow-ons.
