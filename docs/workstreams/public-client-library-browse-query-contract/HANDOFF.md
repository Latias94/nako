# Public Client Library Browse Query Contract - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. PLBQ-040 wired the first Public Client contract for
library-scoped browse through `web/`:

- `GET /libraries/{library_id}/items`
- `LibraryItemsQuery`
- explicit sort/order/facet/watch-state vocabulary
- `LibraryItemsResponse`
- effective Library Access behavior
- SDK methods: generated TypeScript/Kotlin `listLibraryItems(libraryId, query)`
  and Rust `list_library_items`
- first implemented facet: `kind:<ClientMediaKind>`
- `/media/library` uses scoped live browse for supported sort/watch-state
  combinations and keeps readiness messaging for unsupported filters.

## Active Task

- Task ID: PLBQ-050
- Owner: Codex
- Status: READY
- Validation: final backend/frontend gates, `python -m json.tool`, and
  `git diff --check`.

## Next Recommended Action

Start PLBQ-050. Close the lane, record remaining facet/pagination follow-ons,
and decide whether to open the next browse-read-model lane.
