# Playback Subtitle Serving Design

## Intent

Nako already records imported subtitle sidecars as redaction-safe
`media_probe` stream facts. This lane turns those facts into a playback-time
serving path: an authorized client can request subtitle text by source and
stream index, while Nako keeps local paths, backup URIs, addon URLs, and raw
locators out of public contracts.

## In Scope

- Serve sidecar subtitle text through a host-owned playback route.
- Authorize subtitle reads with the same source play access and playback policy
  model used by playback streams.
- Support opaque browser playback tickets scoped to a specific subtitle stream.
- Derive the sidecar leaf name from the source file name and subtitle stream
  fact instead of persisting or exposing a local path.
- Redact storage errors so sidecar paths are not returned to clients.
- Keep subtitle content responses as plain media bytes, not JSON envelopes.

## Out Of Scope

- HLS subtitle renditions and playlist `EXT-X-MEDIA` integration.
- Embedded subtitle extraction.
- Cloud-drive transfer or cloud-side copy.
- Frontend player integration.
- Serving addon remote download URLs directly.

## Boundary

The `media_probe` stream fact is the public/playback-visible read model. The
sidecar storage URI remains an internal playback resolution detail derived from
the source locator and a safe sidecar leaf name. Addon import write and playback
read share the sidecar leaf/URI rules, but playback does not depend on addon
admin request types.

## Validation

- `cargo nextest run -p nako-server subtitle --no-fail-fast`
- `cargo nextest run -p nako-server browser_playback_ticket --no-fail-fast`
- `cargo nextest run -p nako-client-protocol browser_playback --no-fail-fast`
- `cargo nextest run -p nako-api openapi --no-fail-fast`
- `cargo nextest run -p nako-api typescript_sdk --no-fail-fast`
- `cargo nextest run -p nako-api kotlin_sdk --no-fail-fast`
- `cargo check -p nako-api -p nako-client -p nako-client-protocol -p nako-server --tests`
- `cargo fmt --all -- --check`
- `git diff --check`
