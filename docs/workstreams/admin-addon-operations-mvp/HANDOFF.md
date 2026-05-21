# Admin Addon Operations MVP — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The lane has just been opened. Upstream workstreams are complete:

- `release-packaging-and-distribution` productized packaged operation.
- `addon-architecture-deepening` cleaned Addon runtime, protocol, Admin DTO,
  and persistence boundaries.

Admin Addon API currently supports registration/list/detail under
`/admin/v1/addons`, plus token issue/list/rotate/revoke and grant replace/list.
The remaining product gap is operator control and diagnostics.

## Active Task

AAO-010 — Contract and goal baseline.

## Next Steps

1. Complete AAO-010 by freezing the route contract and unregister lifecycle
   policy.
2. Then execute AAO-020 for explicit enable/disable mutation.

## Constraints

- This is not an Addon Manager.
- Do not add Native Plugin or Jellyfin Plugin Compatibility.
- Do not pass administrator bearer tokens to Addon Sidecars.
- Keep Addon Token authority separate from Admin API authority.
- Keep Admin responses redaction-safe.
