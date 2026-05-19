# User Playback State Contract Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

This workstream was split from
`docs/workstreams/android-public-client-api-coverage/` APIC-060.

UPS-010, UPS-020, UPS-030, and UPS-040 are complete. The first public **User Playback State**
contract is frozen in `CONTRACT.md`, and ADR-0028 defines how
**Single-Admin Mode** resolves to an internal stable `local-admin` principal
without making the domain permanently single-user.

Server storage and app-service behavior now exist behind core repository
traits, SQLite migration 0029, `UserPlaybackAppService`, and auth middleware
principal resolution. Public HTTP/API/SDK routes now exist under
`/users/me/playback-state/...` with protocol DTOs, OpenAPI, Rust SDK,
TypeScript SDK, and HTTP API documentation. Android now has a dedicated
`TaruUserPlaybackClient`, renders Continue Watching only from server-backed
state, prefers authoritative resume over device-local fallback, and reports
progress/watched transitions from player exit state. `DevicePlaybackPositionStore`
remains local cache/fallback only.

## Closeout

UPS-050 is complete. The Android smoke lane now proves `profile-with-media`
renders Continue Watching from server-backed **User Playback State**, and the
fixture seed is deterministic even when prior smoke runs left watched state in
the demo fixture database.

Fresh evidence:

- `pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States profile-with-media`
- `apps/android/build/smoke-regression/20260519-164812/report.md`
- `git diff --check`

Recommended validation:

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States profile-with-media
git diff --check
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

Follow-ons should be split into new lanes if needed: multi-user account UI,
offline sync, recommendation logic, favorites, hidden state, and rating.
