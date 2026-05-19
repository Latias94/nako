# User Playback State Contract Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This workstream was split from
`docs/workstreams/android-public-client-api-coverage/` APIC-060.

UPS-010, UPS-020, and UPS-030 are complete. The first public **User Playback State**
contract is frozen in `CONTRACT.md`, and ADR-0028 defines how
**Single-Admin Mode** resolves to an internal stable `local-admin` principal
without making the domain permanently single-user.

Server storage and app-service behavior now exist behind core repository
traits, SQLite migration 0029, `UserPlaybackAppService`, and auth middleware
principal resolution. Public HTTP/API/SDK routes now exist under
`/users/me/playback-state/...` with protocol DTOs, OpenAPI, Rust SDK,
TypeScript SDK, and HTTP API documentation. The current Android behavior
remains device-local resume through
`DevicePlaybackPositionStore`; it must not be presented as
server-authoritative **User Playback State** or cross-device Continue Watching.

## Next Task

Run UPS-040:

- add Android Public Client API methods for
  `GET /users/me/playback-state/items/{item_id}`,
  `GET /users/me/playback-state/continue-watching`,
  `PUT /users/me/playback-state/items/{item_id}/progress`, and
  `PUT /users/me/playback-state/items/{item_id}/watched`;
- integrate authoritative resume and Continue Watching UI against server state;
- keep `DevicePlaybackPositionStore` as fallback/local cache only;
- ensure failed or unavailable server playback state does not produce
  cross-device Continue Watching claims.

Recommended validation:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
```

## Constraints To Preserve

- **User Playback State** is not **Canonical Metadata**.
- Device-local Android resume is fallback/local cache only.
- No Public Client API route should expose source locators, local paths, token
  material, or playback session internals.
- Single-Admin Mode must not become a permanent single-user domain model.
- Continue Watching should appear only when backed by authoritative server
  state.
- UPS-020 should not add favorites, hidden state, or user rating. They are
  deferred from the first route set.
- Public DTOs must not expose internal principal ids. Routes operate on
  `/users/me/...` only.

## Parallel Work

Android UPS-040 is now unblocked. Keep smoke evidence and closeout in UPS-050
until Android uses the public route set.
