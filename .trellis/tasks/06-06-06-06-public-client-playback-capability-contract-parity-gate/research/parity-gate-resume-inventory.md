# Public Client Playback Capability Parity Gate Resume Inventory

Date: 2026-06-06

## Summary

The completed 2026-06-05 parity task already added the primary contract gates
for the current flat v1 Public Client playback capability field set. This
resume task should verify those gates and repair only proven drift.

## Existing Gate Inventory

- `nako-client-protocol`:
  `public_playback_capability_dtos_keep_current_flat_field_contract` checks
  browser capability and client capability DTO serde field sets and rejects
  host/runtime terms.
- `nako-api` OpenAPI:
  `public_openapi_playback_capability_fields_match_current_flat_contract`
  checks query parameters, browser capability schema, and
  `ClientPlaybackCapabilitiesDto`.
- `nako-api` SDK:
  `public_sdk_playback_capability_queries_match_current_flat_contract` checks
  TypeScript and Kotlin playback/remux/HLS query surfaces and Kotlin runtime
  query rendering.
- `nako-api` SDK redaction:
  `generated_public_sdk_playback_capability_surfaces_exclude_host_runtime_facts`
  rejects FFmpeg, GPU/device, hardware, operator, resource pressure, token,
  principal, locator, local path, and output path terms in generated capability
  sections.
- `nako-client-core`:
  playback request builder tests render the full decision/remux/HLS query field
  set and keep invalid output containers out of remux query strings.
- `nako-client`:
  `streaming_request_builders_use_stable_paths_methods_headers_and_queries`
  renders remux and HLS streaming builder queries with the current flat fields.
- `nako-client-uniffi`:
  `uniffi_surface_preserves_full_playback_capability_query_fields` mirrors the
  full query set for bindings.
- `nako-server` playback:
  `playback_capability_queries_map_all_current_flat_fields` maps decision,
  remux, and HLS query structs into `ClientPlaybackCapabilities`.
- `nako-server` browser ticket:
  `browser_playback_ticket_capabilities_map_all_current_flat_fields` maps the
  body capability DTO into planner capabilities.
- `nako-server` renderer:
  `renderer_media_capabilities_map_all_current_flat_fields` maps renderer media
  capability DTOs into planner capabilities.

## Current Baseline Fields

Query/request-preference fields:

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

Allowed remux-only addition:

- `output_container`

Renderer/session DTO collection fields:

- `containers`
- `video_codecs`
- `audio_codecs`

## Implementation Decision

Do not add duplicate parity tests unless a focused gate fails or reveals an
uncovered surface. The correct implementation path for this task is:

1. Run the existing focused gates.
2. Fix only proven drift.
3. Record validation evidence and leave profile v2 for
   `playback-output-profile-v2-skeleton-contract-only`.

## Verification Result

The focused protocol, API/OpenAPI/SDK, client-core, Rust SDK, UniFFI, and server
mapping gates all passed on 2026-06-06. No missing surface was discovered, so
this task required no Rust source, generated SDK, HTTP API documentation, or
`.trellis/spec/` edits.
