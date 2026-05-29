# Public Client Library Browse Query Contract - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. PLBQ-030 implemented the first Public Client contract for
library-scoped browse:

- `GET /libraries/{library_id}/items`
- `LibraryItemsQuery`
- explicit sort/order/facet/watch-state vocabulary
- `LibraryItemsResponse`
- effective Library Access behavior
- SDK methods: generated TypeScript/Kotlin `listLibraryItems(libraryId, query)`
  and Rust `list_library_items`
- first implemented facet: `kind:<ClientMediaKind>`

## Active Task

- Task ID: PLBQ-040
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`.

## Next Recommended Action

Start PLBQ-040. Wire `/media/library` and selected rails to the implemented
Public Client browse route while preserving truthful readiness states for
unsupported facet prefixes.
