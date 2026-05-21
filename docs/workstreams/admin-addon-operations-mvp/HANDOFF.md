# Admin Addon Operations MVP — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

Upstream workstreams are complete:

- `release-packaging-and-distribution` productized packaged operation.
- `addon-architecture-deepening` cleaned Addon runtime, protocol, Admin DTO,
  and persistence boundaries.

Admin Addon API currently supports registration/list/detail under
`/admin/v1/addons`, explicit enable/disable lifecycle mutation, plus token
issue/list/rotate/revoke and grant replace/list. The remaining product gap is
terminal unregister plus operator diagnostics.

AAO-010 is complete. The lane now has a frozen MVP contract:

- `PATCH /admin/v1/addons/{addon_id}/status` for `enabled` / `disabled`;
- `POST /admin/v1/addons/{addon_id}/unregister` for terminal runtime
  unregister;
- `POST /admin/v1/addons/{addon_id}/health-check`;
- `GET /admin/v1/addons/{addon_id}/surfaces`;
- `POST /admin/v1/addons/{addon_id}/diagnostics/resource-call`.

Unregister is not physical deletion. It preserves registration/token/side
effect/candidate audit history, revokes active Addon Tokens, clears accepted
grants, and makes all runtime Addon Token authentication fail. AAO does not
mount `DELETE /admin/v1/addons/{addon_id}`.

AAO-020 is complete. `PATCH /admin/v1/addons/{addon_id}/status` changes only
between `enabled` and `disabled`. Enabling revalidates the stored Addon
Manifest snapshot and granted Addon Scopes. Disabling keeps registration,
tokens, grants, and audit history, but runtime Addon Token authentication
fails before permission checks and before token `last_used_at` is refreshed.
The response uses the existing redaction-safe Admin registration detail.

## Active Task

AAO-030 — Unregister semantics.

## Next Steps

1. Execute AAO-030 by adding the terminal `unregistered` lifecycle state.
2. Implement `POST /admin/v1/addons/{addon_id}/unregister` so it transitions
   the registration, revokes active Addon Tokens, clears accepted grants, and
   preserves audit history.
3. Keep `DELETE /admin/v1/addons/{addon_id}` unmounted.

## Constraints

- This is not an Addon Manager.
- Do not add Native Plugin or Jellyfin Plugin Compatibility.
- Do not pass administrator bearer tokens to Addon Sidecars.
- Keep Addon Token authority separate from Admin API authority.
- Keep Admin responses redaction-safe.
