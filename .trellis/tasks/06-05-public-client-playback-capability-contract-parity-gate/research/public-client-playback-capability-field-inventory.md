# Public Client Playback Capability Field Inventory

Date: 2026-06-05

## Current Compatibility Baseline

The Public Client playback capability contract remains a flat v1 field set.
The supported query/request-preference fields are:

- `direct_play`
- `container`
- `video_codec`
- `audio_codec`
- `max_video_bitrate`
- `max_width`
- `max_height`
- `max_audio_channels`
- `supports_hdr`
- `supports_subtitles`
- `hls_variant_policy`
- `hls_segment_container`

The remux stream query additionally accepts `output_container`. Browser
playback ticket capabilities use the same flat fields and also accept
`output_container` for explicit remux ticket planning. Renderer registration
and heartbeat bodies use `ClientPlaybackCapabilitiesDto`, where the container
and codec fields are pluralized as `containers`, `video_codecs`, and
`audio_codecs`.

## Contract Surfaces Covered

- `nako-client-protocol` owns the body/session/renderer DTO field set and now
  has a serde field-set gate for browser and client capability DTOs.
- `nako-api` OpenAPI generation owns Public Client schemas and query
  parameters and now checks decision, remux, HLS, browser, renderer, SDK, and
  HTTP docs parity.
- `nako-client-core` now renders the complete flat query field set for
  playback decision, remux, and HLS request builders.
- `nako-client` Rust SDK streaming builder tests now cover full remux and HLS
  capability query rendering.
- `nako-server` now has private mapping tests for playback query, remux query,
  HLS query, browser ticket body, and renderer media capability body mapping.
- Generated TypeScript and Kotlin SDK package outputs are protected by existing
  generator-output drift tests. The Kotlin request descriptor runtime now
  renders the complete flat field set.
- `docs/api/HTTP_API.md` documents the current flat fields and the audience
  boundary.

## Public Audience Boundary

The parity gate preserves the current Public Client audience boundary:
capability fields describe client/player facts and request preferences only.
The gate rejects FFmpeg, GPU/device path, hardware probe, operator policy,
resource pressure, bearer token, principal, source locator, raw locator, local
path, and output path terms from Public Client playback capability DTOs,
OpenAPI capability schemas/query parameters, and generated SDK capability
surfaces.

## Deferred Follow-On

The next allowed follow-on is:

`playback-output-profile-v2-skeleton-contract-only`

That follow-on may add optional structured profile/device fields, but it must
remain contract-only until a dedicated execution task covers HEVC/AV1,
hardware tone mapping, image subtitle burn-in, or runtime policy behavior.
