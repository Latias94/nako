# Public Client Library Browse Query Contract - Contract Readiness

Status: Frozen
Last updated: 2026-05-29

## WDRP-065 Decision

Decision: open this Public Client contract lane now.

WMLP recorded two closely related gaps:

- `/media/library` needs library-scoped item browse.
- home rails and list views need stable catalog sort/filter keys.

These should be solved together so the public query shape is coherent.

## Contract Decisions

PLBQ-020 freezes:

| Question | Initial recommendation |
| --- | --- |
| Route shape | Use `GET /libraries/{library_id}/items` for scoped browse. Do not add `library_id` to `GET /items` in the first slice. |
| Page shape | Reuse `PageInfo` with `limit` and `offset`. |
| Sort keys | `title`, `release_date`, `date_added`, and `last_played`; `title` uses sort-title fallback. |
| Sort order | `asc` or `desc`, defaulting to `date_added desc`. |
| Filters | Explicit public facet tokens plus `watch_state=any|watched|unwatched|in_progress`. |
| Access | Require effective `browse` access. Return `404 not_found` for inaccessible libraries. |
| Response | New `LibraryItemsResponse { library, items, page }` reusing `MediaItemDto`. |
| SDK | Add `listLibraryItems(libraryId, query)` to generated TypeScript SDK and equivalent Rust client helper. |
| Web | `/media/library` may show scoped live items only after this contract lands. |

The full frozen contract lives in `CONTRACT.md`.

## Required Gates

```bash
cargo nextest run -p nako-client-protocol catalog --no-fail-fast
cargo nextest run -p nako-api catalog --no-fail-fast
cargo nextest run -p nako-server catalog --no-fail-fast
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```
