# Public Client Library Browse Query Contract - Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

This lane is closed. PLBQ-040 wired the first Public Client contract for
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

- Task ID: none
- Owner: Codex
- Status: DONE
- Validation: final backend/frontend gates, `python -m json.tool`, and
  `git diff --check` passed for PLBQ-050.

## Follow-Ons

- Broaden browse facet implementation for `genre:`, `tag:`, `collection:`,
  `studio:`, `year:`, and `content_rating:` when those read models are needed.
- Add `/media/library` pagination or infinite-scroll UX on top of the existing
  `limit`/`offset` contract.
- Open a separate home-rails/read-model lane if Recently Added or Continue
  Watching needs richer server-side rail composition than the current browse
  route provides.

## Next Recommended Action

Return to the active Public Client follow-on queue. `public-client-browser-playback-session-identity`
is the next open playback/web contract lane unless product priority changes.
