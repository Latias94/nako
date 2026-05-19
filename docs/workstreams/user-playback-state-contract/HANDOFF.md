# User Playback State Contract Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This workstream was split from
`docs/workstreams/android-public-client-api-coverage/` APIC-060.

UPS-010 and UPS-020 are complete. The first public **User Playback State**
contract is frozen in `CONTRACT.md`, and ADR-0028 defines how
**Single-Admin Mode** resolves to an internal stable `local-admin` principal
without making the domain permanently single-user.

Server storage and app-service behavior now exist behind core repository
traits, SQLite migration 0029, `UserPlaybackAppService`, and auth middleware
principal resolution. Public HTTP/API/SDK routes have not started. The current
Android behavior remains device-local resume through
`DevicePlaybackPositionStore`; it must not be presented as
server-authoritative **User Playback State** or cross-device Continue Watching.

## Next Task

Run UPS-030:

- add `taru-client-protocol` DTOs matching `CONTRACT.md`;
- expose `/users/me/playback-state/...` HTTP routes through `taru-server`;
- map route requests to `UserPlaybackAppService` using the resolved
  `UserPrincipalId` request extension;
- update OpenAPI, Rust SDK, TypeScript SDK, and API docs.

Recommended validation:

```powershell
cargo nextest run -p taru-api -p taru-client --no-fail-fast
npm run check --prefix sdk/typescript
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
- UPS-030 must not expose internal principal ids in public DTOs. Routes should
  operate on `/users/me/...` only.

## Parallel Work

Parallel workers are safe for API/SDK work after UPS-020. Keep Android UPS-040
blocked until UPS-030 route names and DTOs are implemented.
