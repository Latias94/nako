# Public Client Library Browse Query Contract - Contract Readiness

Status: Active
Last updated: 2026-05-28

## WDRP-065 Decision

Decision: open this Public Client contract lane now.

WMLP recorded two closely related gaps:

- `/media/library` needs library-scoped item browse.
- home rails and list views need stable catalog sort/filter keys.

These should be solved together so the public query shape is coherent.

## Contract Questions

PLBQ-020 must freeze:

| Question | Initial recommendation |
| --- | --- |
| Route shape | Prefer `GET /libraries/{library_id}/items` for scoped browse. |
| Page shape | Reuse `PageInfo` with `limit` and `offset`. |
| Sort keys | Start with `title`, `sort_title`, `release_date`, `date_added`, and maybe `last_played` only if User Playback State joins are ready. |
| Filters | Start with item kind, genre/tag/collection/studio facets, and watched/unwatched only if user-state filtering is explicitly supported. |
| Access | Enforce effective Library Access before item rows are returned. |
| SDK | Add generated TypeScript/Rust SDK methods and tests. |
| Web | `/media/library` may show scoped live items only after this contract lands. |

## Required Gates

```bash
cargo nextest run -p nako-client-protocol catalog --no-fail-fast
cargo nextest run -p nako-api catalog --no-fail-fast
cargo nextest run -p nako-server catalog --no-fail-fast
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```
