# User Playlists Public Client Contract

Status: Frozen for UPCW-020
Last updated: 2026-05-29

## Purpose

This document freezes the first Public Client contract for current-user
playlists. A **User Playlist** is private user-owned media state. It is not a
catalog **Collection**, not an HLS transport playlist, and not **User Playback
State**.

## Route Namespace

The first route set is current-user only:

```text
GET    /users/me/playlists?limit={limit}&offset={offset}
POST   /users/me/playlists
GET    /users/me/playlists/{playlist_id}
PATCH  /users/me/playlists/{playlist_id}
DELETE /users/me/playlists/{playlist_id}
GET    /users/me/playlists/{playlist_id}/items?limit={limit}&offset={offset}
PUT    /users/me/playlists/{playlist_id}/items/{item_id}
DELETE /users/me/playlists/{playlist_id}/items/{item_id}
PUT    /users/me/playlists/{playlist_id}/items/reorder
```

There are no arbitrary-user, public-share, admin-management, or source-specific
playlist routes in the first slice.

## Ownership And Visibility

The authenticated Public Client principal owns all playlist state. The server
stores principal identity, never bearer token values. All playlists are private
for the first slice and expose `visibility: "private"` so future public/shared
playlist work can extend the vocabulary without reusing catalog Collections.

## Item Semantics

Playlist membership targets Media Item IDs only. Media Sources, Source
Variants, playback sessions, HLS segment names, and local Source Locators are
not playlist membership targets.

The first slice enforces one membership row per playlist/media item. Adding the
same item through `PUT /users/me/playlists/{playlist_id}/items/{item_id}` is
idempotent and does not create duplicates.

Membership has an explicit zero-based integer `position`. New items append by
default, or may request a target position. Reorder replaces the full ordered
membership sequence with an `item_ids` list. The server rejects missing,
duplicate, inaccessible, or foreign item ids in reorder requests.

## Access Filtering

Public Client playlist item responses apply effective Library Access. Items
that are no longer accessible to the current user are omitted from list/detail
item responses. The contract does not return redacted tombstones in the first
slice because tombstones would reveal playlist membership facts outside current
Library Access.

Internal membership rows may remain so access restoration can reveal the item
again in its prior order. Public `item_count` is the accessible item count.

## DTO Shape

Summary responses use:

```typescript
interface UserPlaylistDto {
  id: string;
  name: string;
  visibility: "private";
  item_count: number;
  created_at: string;
  updated_at: string;
  version: number;
}
```

Item responses use:

```typescript
interface UserPlaylistItemDto {
  playlist_id: string;
  item_id: string;
  position: number;
  added_at: string;
  item: MediaItemDto;
  images: PublicImageRefDto[];
}
```

Mutation request bodies are bounded:

- create: `{ "name": string }`;
- update: `{ "name": string, "expected_version"?: number }`;
- add item: `{ "position"?: number, "expected_version"?: number }`;
- reorder: `{ "item_ids": string[], "expected_version"?: number }`.

`expected_version` enables stale-write detection once the server implementation
lands. When provided and stale, the server returns `409 conflict`.

## SDK Expectations

OpenAPI and generated TypeScript/Kotlin SDKs expose User Playlist DTOs and
route helpers. Rust client convenience methods are implemented with the public
route inventory in UPCW-040, alongside server route implementation.

## Non-Goals

- catalog Collection authoring;
- HLS transport playlist changes;
- shared/public playlists;
- smart playlists and recommendation-generated lists;
- music queue semantics;
- offline sync or collaboration conflicts;
- Admin playlist management;
- playlist UI before the route contract is implemented.
