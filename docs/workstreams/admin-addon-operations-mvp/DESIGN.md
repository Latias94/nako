# Admin Addon Operations MVP

Status: Active
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

1. **Unregister semantics.** The default architecture recommendation is a
   terminal lifecycle state that preserves audit records and revokes tokens,
   rather than physical deletion that cascades side effects and candidates. If
   the implementation chooses physical deletion, it must document why losing
   Addon audit state is acceptable.
2. **Health check contract.** Health checks should use a small Addon
   Protocol-owned contract and bounded timeout. Taru must not pass admin
   bearer tokens to an Addon Sidecar.
3. **Diagnostics redaction.** Diagnostics may prove reachability, protocol
   version, declared resource availability, HTTP status class, latency, and
   safe error code. They must not echo raw Addon Tokens, admin bearer tokens,
   payloads, Source Locators, storage URIs, local paths, or provider secrets.

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
