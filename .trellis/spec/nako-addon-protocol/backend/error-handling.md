# Error Handling

Protocol errors use `AddonManifestError` so addon authors and Nako callers get a
single, serializable validation vocabulary.

## Required Patterns

- Return `UnsupportedProtocolVersion` for protocol version drift.
- Return `InvalidBaseUrl` for non-HTTP sidecar base URLs.
- Return duplicate and missing declaration errors instead of accepting ambiguous
  manifests.
- Return `InvalidEnvelope` when response facts do not match the manifest or
  request.
- Return secret-reference errors when a binding looks like a plaintext value.

## Forbidden Patterns

- Do not unwrap JSON serialization or deserialization in validation helpers.
- Do not accept missing scopes because the server might grant them later.
- Do not include token, password, magnet URI, renderer ticket, or local file
  locator values in debug output.
- Do not collapse all validation failures into a generic invalid-manifest error.

## Examples

- A response with the wrong `addon_id` fails envelope validation.
- A task response with mismatched `job_id` fails envelope validation.
- A manifest with duplicate `AddonResource::Metadata` declarations fails before
  callers attempt delivery.

## Review Checklist

- Is the error variant specific enough for callers?
- Does the error avoid leaking secret material?
- Is the same validation covered by tests?
