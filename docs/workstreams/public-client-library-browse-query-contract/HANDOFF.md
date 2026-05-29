# Public Client Library Browse Query Contract - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

This lane is open. PLBQ-020 froze the first Public Client contract for
library-scoped browse:

- `GET /libraries/{library_id}/items`
- `LibraryItemsQuery`
- explicit sort/order/facet/watch-state vocabulary
- `LibraryItemsResponse`
- effective Library Access behavior
- SDK method expectation: `listLibraryItems(libraryId, query)`

## Active Task

- Task ID: PLBQ-030
- Owner: Codex
- Status: READY
- Validation: focused catalog/library route tests, SDK generation check, and
  `cargo nextest run -p nako-server catalog --no-fail-fast`.

## Next Recommended Action

Start PLBQ-030. Implement the frozen contract in server/API/SDK without changing
`web/` readiness behavior until the generated SDK exists.
