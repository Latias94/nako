# Public Client Playback Capability Contract Parity Verification

## Goal

Verify that the Public Client playback capability parity gate implemented from
the 2026-06-05 audit is still present, executable, and sufficient before the
next playback profile v2 contract-only slice starts. If a focused gate fails,
fix the specific drift in the affected surface instead of adding duplicate
coverage.

## Background

The completed 2026-06-05 parity task established the flat v1 Public Client
playback capability baseline across protocol DTOs, OpenAPI/SDK generation,
Rust client builders, server query/body mapping, renderer mapping, UniFFI, and
HTTP docs. This child task belongs to the 2026-06-06 fearless development wave
and intentionally resumes that contract as a verification slice, not as a
profile v2 feature.

## Requirements

- Confirm the current flat v1 Public Client playback capability fields remain
  aligned:
  `direct_play`, `container`, `video_codec`, `audio_codec`,
  `max_video_bitrate`, `max_width`, `max_height`, `max_audio_channels`,
  `supports_hdr`, `supports_subtitles`, `hls_variant_policy`, and
  `hls_segment_container`.
- Confirm remux stream query and browser remux planning still carry
  `output_container` only where allowed.
- Confirm renderer/session capability DTOs still use plural collection fields
  (`containers`, `video_codecs`, `audio_codecs`) while query fields remain
  singular request preferences.
- Confirm generated Public Client SDK surfaces and OpenAPI remain free of
  Admin-only, FFmpeg, GPU/device path, hardware probe, resource pressure,
  bearer token, principal, source locator, local path, raw locator, and output
  path facts.
- Confirm server playback query, browser ticket body, and renderer media
  capability mapping still accept/map every current flat field.
- Keep playback behavior unchanged. This task must not alter planner decisions,
  transcode runtime behavior, HLS artifact identity, auth, tickets, renderer
  control behavior, schema, storage, or durable jobs.

## Acceptance Criteria

- [x] Task-local research records the existing parity gate inventory and any
      discovered missing surface.
- [x] Focused protocol, API/OpenAPI/SDK, client-core/client, and server mapping
      gates pass, or failures are fixed with scoped code changes and rerun.
- [x] `cargo check` covers the touched or verified public client stack.
- [x] Formatting and whitespace gates pass when code or docs are changed.
- [x] No new Public Client playback capability fields are added in this task.
- [x] The next allowed follow-on remains
      `playback-output-profile-v2-skeleton-contract-only`.

## Definition Of Done

- Verification evidence is recorded in this task PRD or research notes.
- If code changes are needed, they are contract-scoped and covered by focused
  tests.
- Generated SDK/OpenAPI artifacts are regenerated from generators if touched.
- Specs are updated only if this task discovers durable knowledge not already
  captured in `.trellis/spec/`.

## Out Of Scope

- No profile v2 fields such as `profile_id`, `profile_version`,
  `device_family`, `player_engine`, audio output matrices, subtitle delivery
  matrices, color pipeline matrices, or HLS output codec matrices.
- No HEVC/AV1 output execution, hardware tone-map execution, image subtitle
  burn-in, Admin effective-profile support evidence, or playback planner
  behavior change.
- No Addon, remote access, source fingerprint hash, storage/VFS, DB schema,
  auth, network endpoint discovery, or durable-job changes.

## Validation Plan

Run focused gates first:

```powershell
cargo nextest run -p nako-client-protocol public_playback_capability_dtos_keep_current_flat_field_contract --no-fail-fast
cargo nextest run -p nako-api public_openapi_playback_capability_fields_match_current_flat_contract public_sdk_playback_capability_queries_match_current_flat_contract generated_public_sdk_playback_capability_surfaces_exclude_host_runtime_facts --no-fail-fast
cargo nextest run -p nako-client-core playback --no-fail-fast
cargo nextest run -p nako-client streaming_request_builders_use_stable_paths_methods_headers_and_queries --no-fail-fast
cargo nextest run -p nako-server playback_capability_queries_map_all_current_flat_fields browser_playback_ticket_capabilities_map_all_current_flat_fields renderer_media_capabilities_map_all_current_flat_fields --no-fail-fast
cargo check -p nako-client-protocol -p nako-client-core -p nako-client -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
```

Broaden only if focused failures or touched files require it.

## Current Evidence

- Prior field inventory:
  `.trellis/tasks/06-05-public-client-playback-capability-contract-parity-gate/research/public-client-playback-capability-field-inventory.md`.
- Relevant specs:
  `.trellis/spec/nako-client-protocol/backend/index.md`,
  `.trellis/spec/nako-api/backend/quality-guidelines.md`,
  `.trellis/spec/nako-client-core/backend/index.md`,
  `.trellis/spec/nako-client/backend/index.md`,
  `.trellis/spec/nako-server/backend/http-api-patterns.md`.

## Validation Evidence

- Passed:
  `python .\.trellis\scripts\task.py validate .trellis\tasks\06-06-06-06-public-client-playback-capability-contract-parity-gate`.
- Passed:
  `cargo nextest run -p nako-client-protocol public_playback_capability_dtos_keep_current_flat_field_contract --no-fail-fast`.
- Passed:
  `cargo nextest run -p nako-api playback_capability --no-fail-fast`.
- Passed:
  `cargo nextest run -p nako-client-core playback --no-fail-fast`.
- Passed:
  `cargo nextest run -p nako-client streaming_request_builders_use_stable_paths_methods_headers_and_queries --no-fail-fast`.
- Passed:
  `cargo nextest run -p nako-server playback_capability_queries_map_all_current_flat_fields browser_playback_ticket_capabilities_map_all_current_flat_fields renderer_media_capabilities_map_all_current_flat_fields --no-fail-fast`.
- Passed:
  `cargo nextest run -p nako-client-uniffi uniffi_surface_preserves_full_playback_capability_query_fields --no-fail-fast`.
- Passed:
  `cargo check -p nako-client-protocol -p nako-client-core -p nako-client -p nako-client-uniffi -p nako-api -p nako-server --tests`.
- Passed:
  `cargo fmt --all -- --check`.
- Passed:
  `git diff --check`.

## Result

All focused gates passed. No Public Client playback capability drift was found,
no code or generated contract changes were required, and no new spec rule was
needed because the relevant executable contract is already captured in
`.trellis/spec/nako-client-protocol/backend/index.md` and
`.trellis/spec/nako-api/backend/quality-guidelines.md`.
