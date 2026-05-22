# 0028: Resolve User Playback State Through a Stable Principal

## Status

Accepted.

## Context

Nako needs server-authoritative **User Playback State** for resume, watched
state, last played ordering, and Continue Watching. The current inbound access
boundary is bearer-token based and deliberately does not implement full user
accounts yet.

That leaves an architectural gap: **User Playback State** is user-scoped, but
the first shipping server mode is **Single-Admin Mode**. If storage or public
routes treat the admin token, the process, or the whole server as the user, the
domain model becomes permanently single-user and later account/profile work
requires a broad data migration.

## Decision

Nako will persist **User Playback State** by an explicit server-internal user
principal, a Media Item id, and an optional Media Source id when source-specific
resume matters.

The inbound authentication layer resolves each accepted request into an
authenticated principal before playback-state services run:

- in **Single-Admin Mode**, every accepted admin bearer token resolves to a
  reserved stable local principal named `local-admin`;
- when auth is explicitly disabled for local development or tests, requests
  also resolve to the same `local-admin` principal unless a test injects a more
  specific principal;
- bearer token values are never stored as user ids, exposed as principals, or
  persisted in playback-state rows;
- the public client contract uses `/users/me/...` routes and never exposes an
  arbitrary user id or the internal principal id in first-slice DTOs.

The first public route set is:

- `GET /users/me/playback-state/items/{item_id}`;
- `GET /users/me/playback-state/continue-watching`;
- `PUT /users/me/playback-state/items/{item_id}/progress`;
- `PUT /users/me/playback-state/items/{item_id}/watched`.

The first DTO surface covers resume position, duration, progress percent,
watched state, watched timestamp, last played timestamp, source id, update
timestamp, and row version. Favorites, hidden state, and user rating remain
**User Playback State** domain concepts, but are intentionally deferred from
the first route set so progress/resume can ship without turning a small slice
into a full user-preferences system.

The server owns watched-threshold policy. For the first implementation, a
progress report auto-marks an item watched when duration is known and either:

- `position_ms / duration_ms >= 0.90` for items at least 60 seconds long; or
- duration is at least 20 minutes and remaining time is at most 120 seconds.

Explicit watched/unwatched commands override automatic progress evaluation.
When an item becomes watched, the authoritative resume position is cleared.

## Consequences

- Storage and service APIs must take a principal explicitly; they must not infer
  user scope from process globals, raw bearer tokens, or request-local strings.
- Future account/profile work can map different credentials to different
  principals while preserving the same playback-state repository contract.
- Migrating Single-Admin Mode to accounts is a bounded principal remapping
  problem instead of a rewrite of global item state.
- Public DTOs stay safe for generated SDKs because they do not expose local
  paths, raw source locators, tokens, or playback-session internals.
- Continue Watching can become a public client feature only when it is backed
  by server-owned rows for the resolved principal.

## Alternatives Considered

- Store playback state globally per Media Item: rejected because watched and
  resume state are explicitly **User Playback State**.
- Use the admin bearer token as the user id: rejected because tokens are
  secrets, can rotate, and must not appear in persisted user data.
- Delay all playback state until full user accounts exist: rejected because
  Android needs authoritative resume and Continue Watching before full account
  management.
- Put user ids directly in public routes now: rejected because the first public
  client slice only needs the current authenticated principal and should not
  imply account administration semantics.

## Related Workstreams

- `docs/workstreams/user-playback-state-contract/`
- `docs/workstreams/access-boundary-auth/`
- `docs/workstreams/android-device-local-playback-position/`
- `docs/workstreams/android-public-client-api-coverage/`
