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

## Envelopes

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

## Registration

`POST /addons` registers a manifest. Addons are disabled by default. To enable
an addon, explicitly set `status` to `enabled` and grant every scope required
by every declared resource.

Runtime secrets are not stored in the manifest. If an addon uses bearer or
shared-secret auth, Taru resolves the secret at call time and sends it as an
HTTP header.

## Reference Addon

The workspace includes `taru-reference-addon`, a minimal metadata addon used by
the M5.5 end-to-end test. It exposes:

- `reference_manifest(base_url)`
- `build_router()`

The reference addon is intentionally small. It proves the wire contract and is
not a full metadata provider.
