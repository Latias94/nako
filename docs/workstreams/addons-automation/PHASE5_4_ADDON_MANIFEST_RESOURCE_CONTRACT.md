# Phase 5.4: Addon Manifest and Resource Contract

Status: completed.

## Goal

Define Nako's first HTTP addon manifest, resource envelope, registration, and
bounded caller contract before adding a reference addon.

## Completed Shape

- Added `nako-addon-protocol` manifest validation for protocol version,
  identity fields, HTTP base URL, resource declarations, scope declarations,
  timeout bounds, retry attempt bounds, and duplicate resources.
- Added resource request and response envelopes with protocol version, addon ID,
  resource kind, request ID, JSON payload, and proposed artifacts.
- Added addon scopes for catalog, metadata, image, subtitle, stream,
  recommendation, automation, and webhook-style access.
- Added explicit resource scope checks. A resource call is denied unless the
  registration grants every scope required by the resource declaration.
- Added a mockable `AddonTransport` plus `ReqwestAddonTransport`.
- Added `call_addon_resource` with bounded timeout, bounded retry attempts,
  bearer/shared-secret runtime authentication headers, non-retryable 4xx
  handling, retryable 408/429/5xx/network handling, and response envelope
  validation.
- Architecture deepening later moved `AddonTransport`,
  `ReqwestAddonTransport`, and `call_addon_resource` to the separate
  permissive `nako-addon-client` crate so `nako-addon-protocol` could remain a
  dependency-light wire-contract crate.
- Added `AddonStatus`, addon registration records, `AddonRepository`, SQLite
  migration `0012_addons.sql`, and SQLite persistence.
- Added HTTP routes for addon registration and inspection.

## Manifest Contract

An addon manifest declares:

- `id`, `name`, `version`, `protocol_version`, and `base_url`;
- optional `description`;
- declared resources with absolute paths, input/output schema names, required
  scopes, timeout, and max attempts;
- authentication mode: `none`, `bearer`, or `shared_secret`;
- the total scope set the addon may request.

The current Addon Protocol Version is `0.1.0-alpha.1`. Nako rejects manifests
whose `protocol_version` is not in the supported Addon Protocol Version set.

## Resource Calls

Resource calls use the `network.addon` budget class conceptually. The first
caller shipped as library code in `nako-addon-protocol`; architecture
deepening later moved that caller to `nako-addon-client`. HTTP handlers still
only register and inspect addons. Handlers do not call addon HTTP endpoints
inline.

Runtime secrets are passed to the caller and emitted only as request headers.
They are not stored in manifests, jobs, outbox events, or registration records.

Retry policy is deliberately bounded:

- `max_attempts` defaults to `1`;
- the accepted range is `1..=10`;
- 408, 429, 5xx, and transport errors are retryable;
- other 4xx responses fail without retry.

## HTTP Surface

Initial routes shipped in M5:

- `POST /addons`
- `GET /addons`
- `GET /addons?status=enabled|disabled`
- `GET /addons/{addon_id}`

`POST /addons` validates the manifest and granted scopes before persistence.
Registrations are disabled by default unless the caller explicitly requests
`enabled` and grants the required scopes.

Later architecture deepening moved this management surface to
`/admin/v1/addons` and removed the root `/addons` compatibility routes before
Nako had external users. This file remains a historical M5 phase note.

## Non-Goals

- No reference addon implementation yet.
- No addon-triggered background job scheduler yet.
- No SDK or generated JSON schema artifact yet.
- No in-process plugin ABI, JavaScript runtime, or Stremio compatibility.

## Validation

Coverage:

- `nako-addon-protocol` tests verify manifest validation, resource envelope
  round-tripping, scope denial, auth token enforcement, bounded retry behavior,
  non-retryable status handling, HTTP request headers, and response envelope
  mapping.
- `nako-db` tests verify addon registration persistence and status filtering.
- `nako-server` HTTP tests verify registration defaults to disabled, status
  filtering, detail lookup, invalid manifest rejection, and scope denial.
- Workspace gates pass when run after this phase: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo nextest run --workspace`, and
  `git diff --check`.
