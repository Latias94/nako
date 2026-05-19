# User Playback State Contract Handoff

Status: Draft
Last updated: 2026-05-19

## Current State

This workstream was split from
`docs/workstreams/android-public-client-api-coverage/` APIC-060.

No implementation has started. The current Android behavior remains
device-local resume through `DevicePlaybackPositionStore`; it must not be
presented as server-authoritative **User Playback State** or cross-device
Continue Watching.

## Next Task

Run UPS-010:

- freeze the first public route contract;
- decide Single-Admin Mode user principal semantics;
- decide which user-state fields are in the first slice;
- decide whether an ADR is required before schema/API work.

Recommended validation:

```powershell
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

## Parallel Work

Parallel workers are safe after UPS-010 completes. Before that, contract work
is a single-owner planning task because route names, principal semantics, and
state fields are tightly coupled.
