# Public Client Library Browse Query Contract - Closeout

Status: Completed
Last updated: 2026-05-29

## Shipped State

PLBQ is closed. Nako now has a first Public Client contract for library-scoped
item browse:

- `GET /libraries/{library_id}/items`;
- `LibraryItemsQuery` with page, sort/order, facet, and watch-state inputs;
- `LibraryItemsResponse { library, items, page }`;
- effective Library Access hiding for inaccessible libraries;
- OpenAPI, generated TypeScript/Kotlin SDKs, and Rust client helper coverage;
- `web/` scoped live browse for supported `/media/library` states.

## Final Gates

- `python -m json.tool docs/workstreams/public-client-library-browse-query-contract/WORKSTREAM.json`
- `git diff --check -- docs/workstreams/public-client-library-browse-query-contract`
- `cargo nextest run -p nako-server catalog --no-fail-fast`
- `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-contracts.test.tsx`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`

## Follow-Ons

- Implement the remaining frozen facet prefixes:
  `genre:`, `tag:`, `collection:`, `studio:`, `year:`, and
  `content_rating:`.
- Add web pagination or infinite-scroll behavior on top of the implemented
  `limit`/`offset` contract.
- Split a home-rails/read-model lane if Recently Added or other home surfaces
  need server-side rail composition beyond direct library browse.

## Residual Risk

Unsupported facet prefixes intentionally return public `invalid_input` errors
until their backend read models land. The web route keeps readiness messaging
for unsupported filters so the UI does not imply unshipped browse semantics.
