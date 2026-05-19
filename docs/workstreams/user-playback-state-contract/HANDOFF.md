# User Playback State Contract Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This workstream was split from
`docs/workstreams/android-public-client-api-coverage/` APIC-060.

UPS-010 is complete. The first public **User Playback State** contract is frozen
in `CONTRACT.md`, and ADR-0028 defines how **Single-Admin Mode** resolves to an
internal stable `local-admin` principal without making the domain permanently
single-user.

No implementation has started. The current Android behavior remains
device-local resume through `DevicePlaybackPositionStore`; it must not be
presented as server-authoritative **User Playback State** or cross-device
Continue Watching.

## Next Task

Run UPS-020:

- implement the explicit principal parameter through core/server storage
  boundaries;
- add SQLite persistence for item/source-scoped playback state;
- implement lookup, progress, and watched/unwatched app-service behavior;
- cover idempotent writes, source/item validation, and watched threshold policy.

Recommended validation:

```powershell
cargo nextest run -p taru-db -p taru-server user_playback_state --no-fail-fast
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

## Parallel Work

Parallel workers are safe after UPS-010. Keep UPS-020 storage/service work
separate from UPS-030 public API/SDK work until repository and service behavior
are proven.
