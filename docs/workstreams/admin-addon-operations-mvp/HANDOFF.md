# Admin Addon Operations MVP — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

Upstream workstreams are complete:

- `release-packaging-and-distribution` productized packaged operation.
- `addon-architecture-deepening` cleaned Addon runtime, protocol, Admin DTO,
  and persistence boundaries.

Admin Addon API currently supports registration/list/detail under
`/admin/v1/addons`, explicit enable/disable lifecycle mutation, terminal
unregister, redaction-safe health checks, plus token issue/list/rotate/revoke
and grant replace/list. The remaining product gap is surface/resource
diagnostics.

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

AAO-030 is complete. `POST /admin/v1/addons/{addon_id}/unregister` transitions
the registration to terminal `unregistered`, revokes active Addon Tokens,
clears accepted grants, and preserves registration, token, Side Effect, and
Addon Artwork Candidate audit history. Direct enable/token issue/token
rotate/grant replace against the terminal registration is rejected.
Re-registration of the same manifest creates a new disabled registration ID
through the normal registration route. `DELETE /admin/v1/addons/{addon_id}` is
not mounted.

AAO-040 is complete. `POST /admin/v1/addons/{addon_id}/health-check` calls the
Addon Sidecar `{base_url}/health` endpoint through `taru-addon-client` using
only protocol headers and a bounded timeout. It returns redaction-safe
reachability, latency, protocol/manifest facts, and safe error codes. Health
checks do not send administrator bearer tokens, Addon Tokens, resolved Secret
Reference values, or resource-call payloads to Addon Sidecars.

## Active Task

AAO-050 — Hosted surface read models.

## Next Steps

1. Execute AAO-050 by adding Admin read models for manifest-declared Addon
   Entry Points, Hosted Pages, configuration schema metadata, Secret Reference
   fields, Addon Tasks, and Addon Event Subscriptions.
2. Keep hosted page URLs external Addon Sidecar URLs and never append admin
   bearer tokens, Addon Tokens, one-time launch secrets, or resolved Secret
   Reference values.
3. Keep DTOs in `taru-api::extension` and orchestration in
   `taru-server::app::addons`.

## Constraints

- This is not an Addon Manager.
- Do not add Native Plugin or Jellyfin Plugin Compatibility.
- Do not pass administrator bearer tokens to Addon Sidecars.
- Keep Addon Token authority separate from Admin API authority.
- Keep Admin responses redaction-safe.
