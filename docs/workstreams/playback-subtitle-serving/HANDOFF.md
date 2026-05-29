# Playback Subtitle Serving Handoff

## Status

Complete.

## Scope

Serve sidecar subtitle text through a host-owned playback route using
`media_probe` sidecar stream facts and storage backends. Keep all locator and
path details internal.

## Completed

- Shared sidecar filename, content type, stream fact, and storage URI
  derivation in `app::subtitle_sidecar`.
- Reused those helpers from addon subtitle import planning/apply paths.
- Added `GET /sources/{source_id}/subtitles/{stream_index}` for sidecar text
  serving with source play access, playback policy checks, size limits, and
  redacted storage errors.
- Added browser playback ticket mode/URL kind `subtitle`, scoped to source plus
  subtitle stream index.
- Updated Public Client route inventory, OpenAPI, TypeScript SDK, and Kotlin
  SDK for subtitle track URLs.

## Do Not Do

- Do not expose sidecar paths, source locators, backup URIs, or addon remote
  URLs.
- Do not add HLS subtitle renditions in this lane.
- Do not implement embedded subtitle extraction.
- Do not add cloud-drive transfer.
- Do not change frontend business views.

## Follow-Ons

- HLS subtitle rendition planning.
- Frontend player subtitle track wiring.
- Embedded subtitle extraction and sidecar reconciliation.
