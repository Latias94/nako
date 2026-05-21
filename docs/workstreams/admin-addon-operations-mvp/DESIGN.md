# Admin Addon Operations MVP

Status: Completed
Last updated: 2026-05-21

## Why This Lane Exists

Release packaging is already complete and Addon Architecture Deepening closed
with clean runtime, protocol, DTO, and persistence boundaries. The next product
gap is operational: an administrator can register Addons, issue Addon Tokens,
replace grants, and submit side effects, but cannot yet operate an Addon like a
product surface.

The Admin Web Console matrix names the missing pieces directly: health check,
enable/disable mutation, delete/unregister, hosted-page metadata, and
resource-call diagnostics.

## Target State

When this lane closes:

- Addon lifecycle operations are explicit Admin API commands, not full
  registration upserts used as status mutation.
- Unregister semantics are intentional and safe: Addon Tokens stop working,
  grants stop authorizing, and audit/redaction rules remain truthful.
- Admins can run an Addon Health Check that proves the Addon Sidecar is
  reachable and still matches the accepted Addon Manifest contract, without
  leaking admin credentials or raw network errors.
- Admin UI can read Addon Entry Points, Hosted Pages, configuration schema
  metadata, tasks, and event subscription declarations through DTOs shaped for
  administration rather than by scraping persistence records.
- Admins can run bounded resource-call diagnostics for declared Addon
  Resources, with redacted request/response/error summaries.
- The work remains Addon Protocol / Addon Sidecar productization, not an Addon
  Manager.

## In Scope

- Admin API routes under `/admin/v1/addons/{addon_id}`.
- Addon status mutation (`enabled` / `disabled`) through a PATCH-like command.
- Unregister/delete command semantics and repository support.
- Addon Health Check protocol/client/server DTOs.
- Hosted Addon surface read models for Admin UI.
- Resource-call diagnostics for declared Addon Resources.
- SQLite/PostgreSQL parity for any Addon lifecycle state changes.
- HTTP API, Addon Author Guide, Admin Web Console matrix, and workstream docs.

## Out Of Scope

- Addon discovery, download, installation, update, marketplace, package
  signing, process supervision, port allocation, logs, rollback, or removal of
  external sidecar processes.
- OAuth-first authorization or remote multi-tenant Addons.
- Native Plugin ABI or Jellyfin Plugin Compatibility.
- Embedded trusted Admin UI.
- Full Addon Task runtime or Addon Event Subscription delivery.
- Subtitle or arbitrary Library File Write breadth.

## Key Design Questions

1. **Unregister semantics.** AAO chooses a terminal runtime lifecycle state,
   not physical deletion. `unregistered` Addons remain visible to Admin
   history/detail reads, but cannot be enabled, cannot authenticate runtime
   Addon routes, and cannot authorize side effects. Unregister atomically
   revokes active Addon Tokens and clears the accepted grant set. Side Effect,
   token, registration, and Addon Artwork Candidate history is preserved for
   audit.
2. **Health check contract.** Health checks should use a small Addon
   Protocol-owned contract and bounded timeout. Taru must not pass admin
   bearer tokens to an Addon Sidecar.
3. **Diagnostics redaction.** Diagnostics may prove reachability, protocol
   version, declared resource availability, HTTP status class, latency, and
   safe error code. They must not echo raw Addon Tokens, admin bearer tokens,
  payloads, Source Locators, storage URIs, local paths, or provider secrets.

## Frozen MVP Contract

### Lifecycle states

AAO extends the Addon registration lifecycle to:

- `enabled`: runtime Addon Token authentication and accepted grants may
  authorize Addon runtime routes.
- `disabled`: registration is retained, but runtime Addon Token authentication
  fails before permission checks. Admins may re-enable after validation.
- `unregistered`: terminal runtime state. Registration and audit history are
  retained, active tokens are revoked, accepted grants are cleared, and runtime
  Addon Token authentication always fails. AAO does not physically delete the
  registration row or cascade historical records.

Re-registering a previously unregistered `manifest_id` is allowed only through
the normal registration route and starts from `disabled` with no accepted
grants and no reusable token. It preserves the old registration history instead
of pretending the Addon never existed.

### Route contract

AAO reserves these Admin API routes:

- `PATCH /admin/v1/addons/{addon_id}/status`
  - Body: `{ "status": "enabled" | "disabled" }`.
  - Does not accept `unregistered`; unregister is a separate command.
  - Enabling validates the stored manifest and accepted scopes/grants but does
    not imply sidecar process management.
- `POST /admin/v1/addons/{addon_id}/unregister`
  - Transitions the registration to `unregistered`, revokes active Addon
    Tokens, clears accepted grants, and returns a redaction-safe Admin
    lifecycle response.
  - AAO intentionally does not mount `DELETE /admin/v1/addons/{addon_id}` to
    avoid implying physical deletion.
- `POST /admin/v1/addons/{addon_id}/health-check`
  - Calls the Addon Sidecar health contract at the accepted manifest base URL
    with bounded timeout and protocol headers only.
  - Does not send administrator bearer tokens, Addon Tokens, or resolved
    Secret Reference values.
- `GET /admin/v1/addons/{addon_id}/surfaces`
  - Returns Entry Points, Hosted Pages, configuration schema metadata, Secret
    Reference field declarations, Addon Task declarations, and Addon Event
    Subscription declarations as Admin DTOs.
  - Hosted Page URLs are external Addon Sidecar URLs and must not contain admin
    bearer tokens or one-time secret launch parameters.
- `POST /admin/v1/addons/{addon_id}/diagnostics/resource-call`
  - Runs a bounded diagnostic call against one declared Addon Resource.
  - Uses current manifest declarations and granted scopes to classify missing
    resource, missing grant, protocol mismatch, timeout, retryable transport
    failure, non-retryable HTTP failure, invalid envelope, and unsafe response
    cases.
  - Never echoes raw diagnostic payloads, response bodies, Addon Tokens, admin
    tokens, Source Locators, storage paths, or provider secrets.

### Addon Manager boundary

AAO does not start, stop, install, update, remove, supervise, package, sign, or
log Addon Sidecar processes. It only operates registrations, credentials,
grants, health, surface metadata, and diagnostics for manually managed Addon
Sidecars.

## Architecture Direction

- Keep `taru-addon-protocol` as the wire contract crate.
- Keep outbound HTTP/probe behavior in `taru-addon-client`.
- Keep Admin DTOs in `taru-api`, not in `taru-core` persistence records.
- Keep orchestration in `taru-server` App Services and HTTP routes.
- Keep SQLite/PostgreSQL behavior backend-neutral when lifecycle persistence
  changes.

## Closeout Condition

This lane can close when:

- lifecycle mutation, unregister, health check, hosted surface metadata, and
  diagnostics are either implemented or explicitly split into named follow-ons;
- Admin Addon DTOs remain redaction-safe;
- Addon Token authority and admin bearer-token authority remain separated;
- focused Addon API and DB gates pass;
- workspace checks pass when practical;
- PostgreSQL opt-in contracts are run when `TARU_TEST_POSTGRES_URL` is
  available, or skipped with explicit evidence.

Closed on 2026-05-21 after AAO-010 through AAO-070 completed. No follow-on
workstream was required for this MVP closeout.
