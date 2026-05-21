# Admin Addon Operations MVP — Handoff

Status: Completed
Last updated: 2026-05-21

## Current State

Upstream workstreams are complete:

- `release-packaging-and-distribution` productized packaged operation.
- `addon-architecture-deepening` cleaned Addon runtime, protocol, Admin DTO,
  and persistence boundaries.

Admin Addon API currently supports registration/list/detail under
`/admin/v1/addons`, explicit enable/disable lifecycle mutation, terminal
unregister, redaction-safe health checks, hosted surface read models, bounded
resource-call diagnostics, plus token issue/list/rotate/revoke and grant
replace/list. The lane is closed.

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

AAO-050 is complete. `GET /admin/v1/addons/{addon_id}/surfaces` returns Admin
read models for manifest-declared Entry Points, Hosted Pages, Configuration
Schema metadata, Secret Reference field declarations, Addon Tasks, and Addon
Event Subscriptions. Hosted Page URLs are derived from manifest base URL and
declared paths only; Taru does not append administrator bearer tokens, Addon
Tokens, launch secrets, or resolved Secret Reference values.

AAO-060 is complete. `POST
/admin/v1/addons/{addon_id}/diagnostics/resource-call` runs a bounded
diagnostic call against a declared Addon Resource and returns classification
facts only. It distinguishes success, missing resource, missing grant,
authorization gap, unreachable transport, protocol mismatch, retryable HTTP
failure, non-retryable HTTP failure, and unsafe response cases. It does not
echo raw diagnostic payloads, Addon response payloads, raw response bodies,
token material, Source Locators, storage paths, provider secrets, or raw
network errors.

## Active Task

None. AAO-070 closed the workstream on 2026-05-21.

## Closeout

AAO-070 is complete. The design target state is met: an operator can manage
manually registered Addon Sidecars through registration, enable/disable
lifecycle mutation, terminal unregister, health checks, hosted surface read
models, resource-call diagnostics, token management, and grant management under
`/admin/v1/addons`.

No hidden tail was kept inside this lane. Addon Manager discovery, install,
update, package signing, process supervision, logs, rollback, removal, full
Addon Task runtime, and Addon Event Subscription delivery remain explicit
future non-goals rather than vague buckets.

Closeout validation passed:

- `cargo fmt --all -- --check`;
- `cargo check -p taru-addon-protocol -p taru-addon-client -p taru-api -p
  taru-core -p taru-db -p taru-server --tests`;
- `cargo nextest run -p taru-addon-protocol -p taru-addon-client
  --no-fail-fast`;
- `cargo nextest run -p taru-db addon --no-fail-fast`;
- `cargo nextest run -p taru-server addons --no-fail-fast`;
- `cargo check --workspace --tests`;
- `cargo nextest run --workspace --no-fail-fast`;
- `git diff --check`.

PostgreSQL opt-in contracts were skipped because `TARU_TEST_POSTGRES_URL` was
not set.

## Constraints

- This is not an Addon Manager.
- Do not add Native Plugin or Jellyfin Plugin Compatibility.
- Do not pass administrator bearer tokens to Addon Sidecars.
- Keep Addon Token authority separate from Admin API authority.
- Keep Admin responses redaction-safe.
