# Addon Author Guide

This guide describes the first Taru HTTP addon contract.

## Manifest

An addon is an HTTP sidecar described by a JSON manifest. Taru stores a
validated manifest snapshot during registration.

Required manifest fields:

- `id`: stable addon ID, for example `taru.reference.metadata`.
- `name`: display name.
- `version`: addon implementation version.
- `protocol_version`: must match Taru's current addon protocol version.
- `base_url`: HTTP or HTTPS base URL for resource calls.
- `resources`: declared resource endpoints.
- `auth`: `none`, `bearer`, or `shared_secret`.
- `scopes`: all scopes the addon may request.

The current protocol version is `2026-05-15`.

Optional manifest declarations describe future-facing Addon Protocol concepts
without granting runtime authority by themselves:

- `entry_points`: user-visible Addon Entry Points such as item actions,
  settings, diagnostics, or task launchers. Each entry point has an `id`,
  `kind`, `label`, absolute `path`, optional `hosted_page_id`, and
  `required_scopes`.
- `hosted_pages`: Addon Hosted Pages served by the sidecar for advanced
  settings or diagnostics. They are external pages; Taru does not treat them as
  trusted admin UI and does not pass admin credentials to them.
- `configuration_schema`: an Addon Configuration Schema object identified by
  `schema_id`. Taru stores and presents configuration from this schema.
- `secret_reference_fields`: Secret Reference fields for sensitive settings.
  These declare references only; plaintext secrets do not belong in the
  manifest.
- `event_subscriptions`: Addon Event Subscriptions with `id`, `event_kind`,
  absolute delivery `path`, optional JSON `filters`, and `required_scopes`.
  Event delivery runtime breadth is deferred, but the declaration contract is
  explicit.
- `tasks`: Addon Task declarations with `id`, `name`, absolute execution
  `path`, optional `description`, `required_scopes`, `timeout_ms`, and
  `max_attempts`. Taru owns task lifecycle when this runtime is implemented.

## Resources

Each resource declaration defines:

- `kind`: resource kind such as `metadata`.
- `path`: absolute path below `base_url`.
- `input_schema` and `output_schema`: stable schema names.
- `required_scopes`: scopes Taru must grant before calling the resource.
- `timeout_ms`: per-call timeout.
- `max_attempts`: bounded retry budget.

Taru rejects duplicate resources, relative paths, unsupported protocol
versions, invalid timeout/retry bounds, and undeclared required scopes.
It also rejects duplicate declaration IDs within each declaration class,
relative declaration paths, entry points that reference unknown hosted pages,
non-object configuration schemas, invalid task timeout/retry bounds, and
declaration scopes that are not listed in the manifest-level `scopes`.

## Envelopes

### Health Check

Addon Sidecars should expose `POST /health` below their manifest `base_url`.
Taru uses this endpoint for Admin Addon Health Check operations. The request is
bounded and carries protocol headers only; it does not include administrator
bearer tokens, Addon Tokens, resolved Secret Reference values, or resource
payloads.

Request body:

```json
{
  "protocol_version": "2026-05-15",
  "manifest_id": "taru.reference.metadata",
  "request_id": "health-1",
  "expected_addon_version": "0.1.0",
  "expected_resource_count": 1
}
```

Response body:

```json
{
  "protocol_version": "2026-05-15",
  "manifest_id": "taru.reference.metadata",
  "status": "ok",
  "checked_at": "2026-05-21T12:00:00.000Z",
  "manifest": {
    "addon_version": "0.1.0",
    "resource_count": 1
  },
  "diagnostics": {
    "safe_note": "ready"
  }
}
```

Taru validates the protocol version, manifest ID, addon version, and resource
count before reporting the sidecar as reachable. Addon diagnostics should be
safe summary facts only; do not include credentials, raw request/response
bodies, local paths, or raw network errors.

### Resource Calls

Taru calls resources with:

```json
{
  "protocol_version": "2026-05-15",
  "addon_id": "taru.reference.metadata",
  "resource": "metadata",
  "request_id": "request-1",
  "payload": {
    "title": "The Matrix"
  }
}
```

The addon must respond with the same `protocol_version`, `addon_id`,
`resource`, and `request_id`:

```json
{
  "protocol_version": "2026-05-15",
  "addon_id": "taru.reference.metadata",
  "resource": "metadata",
  "request_id": "request-1",
  "payload": {
    "title": "The Matrix",
    "summary": "Reference addon metadata suggestion"
  },
  "artifacts": [
    {
      "kind": "metadata_suggestion",
      "payload": {
        "title": "The Matrix"
      }
    }
  ]
}
```

Taru validates response identity fields before trusting the response.

Admin resource-call diagnostics use this same resource envelope to exercise a
declared resource with bounded retry/timeout behavior. Diagnostic Admin
responses are intentionally classification-only: Taru reports safe facts such
as success, missing resource, missing grant, authorization gap, unreachable
transport, protocol mismatch, retryable or non-retryable HTTP failure, unsafe
response, latency, attempts, and HTTP status. It does not echo the diagnostic
payload, addon response payload, raw response body, admin bearer tokens, Addon
Tokens, Source Locators, storage paths, provider secrets, or raw network
errors. If a sidecar manifest requires bearer or shared-secret auth and Taru
does not have a safe resolved resource-call secret for the diagnostic, the
operation is reported as an authorization gap rather than sending admin
credentials to the sidecar.

## Registration

`POST /admin/v1/addons` registers a manifest. Addons are disabled by default.
To enable an addon, explicitly set `status` to `enabled` and grant every scope
required by every declared resource. Registration, listing, and detail lookup
are Admin API operations; Taru no longer mounts the old root `/addons`
management routes.

Runtime secrets are not stored in the manifest. If an addon uses bearer or
shared-secret auth, Taru resolves the secret at call time and sends it as an
HTTP header.

## Install Guide

Taru can generate an **Addon Install Guide** for a registered Addon Sidecar via
`GET /admin/v1/addons/{addon_id}/install-guide`.

The guide is for operators, not for Taru-managed lifecycle automation. It
contains Docker Compose and systemd snippets as inert text, Secret Reference
placeholders, direct sidecar health-check commands, Taru Admin health-check
commands, and registration/surface verification commands.

Addon authors should treat the generated snippets as a compatibility aid:

- publish a stable container image or binary command that can replace the
  guide placeholders;
- keep `base_url` reachable from Taru but do not require Taru to mount Docker
  sockets, call systemd, or supervise the process;
- declare Secret Reference fields in the manifest instead of documenting
  plaintext environment variables;
- expose `POST /health` so the guide's direct and Taru-mediated health checks
  can verify protocol, manifest ID, version, and resource count;
- keep hosted pages external and untrusted; do not expect Taru to pass admin
  bearer tokens or Addon Tokens into an install guide or hosted page.

The install guide intentionally does not include resolved Secret Reference
values, raw Addon Tokens, admin bearer tokens, Source Locators, storage paths,
or process-control instructions such as start/stop/restart. If Taru later grows
an Addon Manager, it should be a separate product surface and protocol
boundary.

## Protected Write Payload Contracts

Addon Side Effect Protected Writes use explicit payload structs from
`taru-addon-protocol`:

- `AddonMetadataPatch` for `metadata_write`.
- `AddonArtworkWritePayload` for the first `artwork_write` Addon Artwork
  Candidate proposal slice.
- `AddonLibraryFileWritePayload` for the first `library_file_write` NFO Export
  slice.

These structs describe the wire payload only. They do not expose Taru
persistence records, raw Source Locators, storage paths, cache URIs, token
material, or selected-artwork state.

`taru-addon-protocol` is intentionally a dependency-light wire-contract crate.
Rust hosts that want Taru's bounded HTTP caller, mockable transport trait, and
`ReqwestAddonTransport` can depend on the separate permissive
`taru-addon-client` crate.

## Reference Addon

The workspace includes `taru-reference-addon`, a minimal metadata addon used by
the M5.5 end-to-end test. It exposes:

- `reference_manifest(base_url)`
- `build_router()`
- `demo_metadata_patch(title)`
- `demo_nfo_export_payload()`

The reference addon is intentionally small. It proves the wire contract and is
not a full metadata provider. Its manifest includes a sample Addon Entry Point,
Addon Hosted Page, and Addon Configuration Schema so Addon authors can see the
declaration contract without needing the deferred Addon Manager, Event
Subscription runtime, or Addon Task scheduler.
