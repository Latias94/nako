# User Playback State Public Contract

Status: Frozen for UPS-010
Last updated: 2026-05-19

## Purpose

This document freezes the first executable public contract for
server-authoritative **User Playback State**. It is the implementation target
for UPS-020, UPS-030, and UPS-040.

The contract follows:

- `CONTEXT.md` for **User Playback State**, **User Library State**, and
  **Single-Admin Mode** terminology;
- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md` for the
  public v1 compatibility and error envelope;
- `docs/adr/0024-inbound-token-authentication-boundary.md` for bearer
  authentication;
- `docs/adr/0025-openapi-public-client-sdk-contract.md` for protocol-owned
  public DTOs;
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md` for
  principal and route decisions.

## Principal Model

Public routes operate on the authenticated principal resolved by the server.
The principal is an internal service/storage value, not a public DTO field.

In **Single-Admin Mode**, every accepted admin bearer token resolves to the
reserved stable principal `local-admin`. Auth-disabled local development and
tests use the same principal unless a test explicitly injects another
principal. Token values are never stored as user ids.

The public route namespace is `/users/me/...`. First-slice clients cannot read
or mutate arbitrary users' playback state.

## Route Inventory

| Method | Route | Request | Response | Semantics |
| --- | --- | --- | --- | --- |
| `GET` | `/users/me/playback-state/items/{item_id}` | none | `UserPlaybackStateResponse` | Return state for the resolved principal and item. Existing item with no state returns a default state with `version = 0`. Unknown item returns `404 not_found`. |
| `GET` | `/users/me/playback-state/continue-watching?limit={limit}&offset={offset}` | query | `ContinueWatchingResponse` | Return resume candidates for the resolved principal, sorted by `last_played_at desc`. |
| `PUT` | `/users/me/playback-state/items/{item_id}/progress` | `UpdatePlaybackProgressRequest` | `UserPlaybackStateResponse` | Record a playback progress tick or exit position and apply server-owned watched policy. |
| `PUT` | `/users/me/playback-state/items/{item_id}/watched` | `SetWatchedStateRequest` | `UserPlaybackStateResponse` | Explicitly mark an item watched or unwatched. |

All routes require the inbound auth boundary when auth is enabled. All errors
use the public v1 `ErrorResponse` envelope.

## DTO Inventory

DTOs live in `nako-client-protocol` once implemented.

```rust
pub struct UserPlaybackStateResponse {
    pub state: UserPlaybackStateDto,
}

pub struct ContinueWatchingResponse {
    pub items: Vec<ContinueWatchingItemDto>,
    pub page: PageInfo,
}

pub struct ContinueWatchingItemDto {
    pub item: MediaItemDto,
    pub state: UserPlaybackStateDto,
    pub images: Vec<PublicImageRefDto>,
}

pub struct UserPlaybackStateDto {
    pub item_id: String,
    pub source_id: Option<String>,
    pub resume_position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub progress_percent: Option<f32>,
    pub watched: bool,
    pub watched_at: Option<String>,
    pub last_played_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: u64,
}

pub struct UpdatePlaybackProgressRequest {
    pub source_id: Option<String>,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
    pub reported_at: Option<String>,
}

pub struct SetWatchedStateRequest {
    pub watched: bool,
    pub source_id: Option<String>,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub marked_at: Option<String>,
}
```

Timestamp strings are RFC 3339 UTC values. `PageInfo` keeps the existing v1
`limit`, `offset`, and `returned` fields.

## Progress Semantics

Progress reports are idempotent for repeated identical values. The server uses
`reported_at` when supplied and server receipt time otherwise.

For out-of-order progress reports, the service should not move authoritative
state backward when the incoming event time is older than the stored state.
UPS-020 may implement this as event-time last-write-wins with server receipt
time as the tie breaker.

`position_ms = 0` clears resume position but does not mark the item watched.
Positive positions below the watched threshold set `resume_position_ms` and
`last_played_at`. The server derives `progress_percent` from position and
duration when duration is known.

The first watched threshold policy is:

- duration must be known;
- for items at least 60 seconds long, `position_ms / duration_ms >= 0.90`
  marks the item watched;
- for items at least 20 minutes long, remaining time at or below 120 seconds
  also marks the item watched.

When an item becomes watched, `watched = true`, `watched_at` is set, and
`resume_position_ms` is cleared. If duration is unknown, progress reports never
auto-mark watched.

## Explicit Watched Semantics

`SetWatchedStateRequest.watched = true` marks the item watched and clears the
resume position. If `position_ms` or `duration_ms` is supplied, the server may
use them to update duration/progress-derived fields, but watched state wins.

`watched = false` clears `watched_at`. If a positive `position_ms` below the
watched threshold is supplied, the server may restore that resume position;
otherwise resume remains empty.

## Continue Watching Semantics

Continue Watching returns only items with a non-empty resume position for the
resolved principal. Watched items are excluded. Results are sorted by
`last_played_at desc`, then by item id for deterministic pagination.

Each row includes the public `MediaItemDto`, `UserPlaybackStateDto`, and public
image refs needed by clients to render a rail without item-detail N+1 calls.

## First-Slice Deferrals

The following remain out of the first route set:

- full multi-user account management;
- arbitrary-user admin routes;
- favorites;
- hidden state;
- user rating;
- offline sync conflict resolution;
- playback-session internals;
- audio, subtitle, and chapter selection persistence.

Favorites, hidden state, and user rating still belong to **User Playback
State** or broader **User Library State**; they are deferred fields, not global
Media Item metadata.

## Boundary Rules

Public DTOs and requests must not expose:

- local filesystem paths;
- raw source locators;
- bearer tokens or resolved token values;
- internal principal ids;
- playback-session internals;
- database ids that are not already public item/source ids.

Android device-local resume remains a fallback/local cache. It must not drive
cross-device Continue Watching or be presented as server-authoritative state.
