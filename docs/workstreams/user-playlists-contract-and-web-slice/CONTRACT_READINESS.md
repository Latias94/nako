# User Playlists Contract And Web Slice - Contract Readiness

Status: Active
Last updated: 2026-05-28

## WDRP-050 Decision

Decision: open a backend/Public Client contract lane now, but keep frontend
playlist UI deferred until the contract is frozen.

Rationale:

- `user-playback-state-contract` established current-user route semantics under
  `/users/me/...` and stable principal storage.
- `identity-and-library-access-contract` established effective Library Access
  enforcement for Public Client browse/playback/user-state routes.
- The new `web/` Public Client data source already handles server-owned user
  playback state after WMLP closeout.
- No user playlist route, DTO, repository, or SDK method currently exists.
- Existing catalog `Collection` data and HLS `playlist.m3u8` routes are not a
  user-created playlist feature.

## Required Contract Decisions

UPCW-020 must freeze these before implementation:

| Decision | Initial recommendation |
| --- | --- |
| Route namespace | `/users/me/playlists` only. No arbitrary-user routes. |
| Ownership | Authenticated principal owns playlists; token values are never stored. |
| Visibility | Private playlists only in the first slice. |
| Item target | Media Item IDs only; Source-specific playlists are deferred. |
| Ordering | Explicit integer order with deterministic pagination. |
| Duplicate membership | Prefer one row per playlist/item in the first slice. |
| Access filtering | Inaccessible items are omitted or returned as redacted tombstones; UPCW-020 must choose. |
| Mutations | Create/rename/delete playlist; add/remove/reorder items. No media/library writes. |
| Concurrency | Use version or updated-at fields for stale mutation detection if needed. |
| SDK surface | Rust and TypeScript SDK methods generated from Public Client contract. |
| Web entry | `/media/playlists` only after route/DTO contract exists. |

## Candidate Route Inventory

UPCW-020 should validate or revise this route set:

| Method | Route | Semantics |
| --- | --- | --- |
| `GET` | `/users/me/playlists?limit={limit}&offset={offset}` | List current user's playlists. |
| `POST` | `/users/me/playlists` | Create a private playlist. |
| `GET` | `/users/me/playlists/{playlist_id}` | Return playlist detail and summary. |
| `PATCH` | `/users/me/playlists/{playlist_id}` | Rename/update playlist metadata. |
| `DELETE` | `/users/me/playlists/{playlist_id}` | Delete playlist and membership rows. |
| `GET` | `/users/me/playlists/{playlist_id}/items?limit={limit}&offset={offset}` | Return ordered accessible items. |
| `PUT` | `/users/me/playlists/{playlist_id}/items/{item_id}` | Add or idempotently keep an item. |
| `DELETE` | `/users/me/playlists/{playlist_id}/items/{item_id}` | Remove an item. |
| `PUT` | `/users/me/playlists/{playlist_id}/items/reorder` | Reorder membership. |

## Non-Goals For First Slice

- shared/public playlists;
- smart playlist rules;
- recommendation-generated lists;
- music queue semantics;
- watch-party queues;
- Admin-only playlist management;
- importing playlists from external providers.

## Validation Baseline

Future implementation tasks should include:

```bash
cargo nextest run -p nako-db playlist --no-fail-fast
cargo nextest run -p nako-server user_playlist --no-fail-fast
cargo nextest run -p nako-api playlist --no-fail-fast
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run check
npm --prefix web run build:budget
```
