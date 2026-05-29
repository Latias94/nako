# User Playlists Contract And Web Slice - Design

Status: Closed
Last updated: 2026-05-29

## Problem

The imported v0 frontend had Playlist/My List mock surfaces, but Nako has no
server-owned user playlist contract. The current public protocol contains
catalog **Collections** and HLS playlist transport routes; neither is a
user-owned ordered list of media items. Restoring a playlist UI now would create
fixture-only behavior and blur user state with catalog metadata.

WDRP-050 decides that playlist work is ready to become a backend/Public Client
contract lane because the prerequisites are now in place:

- `user-playback-state-contract` closed with `/users/me/...` principal-scoped
  public routes.
- `identity-and-library-access-contract` closed with durable identity/access
  storage and Public Client effective-access filtering.
- `web-media-live-public-client-parity` closed with web playback-state reads
  and writes wired through Public Client data sources.

## Target State

When this lane closes:

- Nako has a clear **User Playlist** domain that is distinct from catalog
  **Collection** and HLS transport playlist semantics.
- Public Client API exposes current-user playlist list/detail/mutation routes
  under `/users/me/playlists`.
- Playlist storage is principal-scoped and does not store bearer token values.
- Playlist item membership is ordered, idempotent, and references public media
  item identity without copying canonical metadata.
- Public responses filter or annotate inaccessible items using effective
  Library Access.
- TypeScript/Rust SDKs expose the playlist contract.
- The new `web/` Media surface restores playlist UI only after the contract is
  frozen and tested.
- Route contracts, data-source contract tests, TypeScript check, bundle budget,
  and browser smoke evidence pass for the first web slice.

## Closed Result

This lane closed after delivering the backend/Public Client contract,
persistence, SDKs, access-filtered HTTP routes, and first read-oriented web
playlist slice. Product mutation UI for create, rename, add, remove, and
reorder is intentionally split to a follow-on because the first web slice
proved the contract and restored live list/item browsing without widening the
lane into full playlist management.

## Scope

In scope:

- Contract design for current-user playlists.
- Core domain records, repository traits, and SQLite/PostgreSQL adapters.
- Public Client DTOs, OpenAPI, TypeScript SDK, and Rust client support.
- Server route/app-service implementation with effective Library Access checks.
- First `web/` slice for playlist list/detail and minimal item add/remove once
  the contract is proven.
- Tests proving playlist state is user-scoped and not canonical metadata.

Out of scope:

- Social/shared playlists.
- Public playlist URLs, invites, or collaboration.
- Smart playlists, recommendations, or rules-based lists.
- Offline sync conflict resolution.
- Admin playlist management.
- Music-specific queues or podcasts.
- Reusing removed fixture-only UI before the public contract exists.

## Architecture Direction

Model **User Playlist** as user-owned media state:

```text
nako-core
  Owns UserPlaylistId, playlist records, ordered membership records, repository
  traits, and domain validation.

nako-db
  Owns SQLite/PostgreSQL schema, adapters, and contract tests for principal
  scope, ordering, idempotency, and deletion semantics.

nako-api / nako-client-protocol
  Own public DTOs and adapters. Admin DTOs are not involved.

nako-server
  Owns `/users/me/playlists` route orchestration, authenticated principal
  resolution, and effective Library Access filtering.

web/src/api/public
  Owns DTO-to-UI mapping and fixture/live fallback only after the public
  contract is frozen.
```

First slice should be private current-user playlists only. A playlist item
references a Media Item; playback still goes through normal playback routes and
permissions. Playlist mutation must not write canonical metadata, NFO sidecars,
media sources, or library files.

## Risk Plan

- Contract confusion: freeze the vocabulary before implementation and update
  docs wherever "collection" could be mistaken for "playlist".
- Permission leaks: every list/detail response must apply effective Library
  Access before returning item details.
- Duplicate/order ambiguity: first contract must choose whether duplicate item
  membership is rejected or explicitly modeled.
- Frontend drift: no `web/` playlist UI before UPCW-020 freezes route and DTO
  shape.
