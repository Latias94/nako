# Public Client And Admin Playback Capability Contract Research

Date: 2026-06-05

## Summary

The current Public Client playback capability contract is broader than the
early four-field shape, but it is still a flat capability set. It can describe
basic containers, video/audio codecs, bitrate and resolution limits, audio
channel limits, HDR/subtitle booleans, HLS variant policy, and HLS segment
container. It cannot yet describe a stable device family, profile version,
container/codec condition rows, codec profile and level, bit depth, frame rate,
HDR format matrix, subtitle delivery matrix, audio output behavior, or HEVC/AV1
output preferences.

The Public Client and Admin boundary is directionally correct. Public Client
APIs accept safe client/player capability facts and return safe playback
decisions, tickets, URLs, and session state. Admin APIs expose runtime,
hardware, resource pressure, support evidence, sessions, and renderer
diagnostics. The missing Admin surface is a redaction-safe effective profile
and decision-matrix summary for support evidence; hardware probes, FFmpeg
stage details, host paths, device paths, and commands must remain Admin-only
and must not flow back into Public Client DTOs.

The safest follow-on is not HEVC/AV1 execution or hardware tone mapping. The
next executable work should first close capability contract drift across
protocol, OpenAPI, generated clients, client-core builders, SDKs, and
documentation, then add an additive output/device profile skeleton.

## Evidence

- `crates/nako-client-protocol/src/catalog.rs:496`: browser ticket requests
  accept optional `BrowserPlaybackCapabilitiesDto`.
- `crates/nako-client-protocol/src/catalog.rs:506`: browser ticket
  capabilities are flat optional fields and include `output_container`.
- `crates/nako-client-protocol/src/catalog.rs:705`: public
  `ClientPlaybackCapabilitiesDto` requires the four base fields and adds
  optional bitrate, resolution, audio channel, HDR, subtitle, and HLS fields.
- `crates/nako-client-protocol/src/catalog.rs:769`: renderer registration and
  heartbeat carry optional `media_capabilities`.
- `crates/nako-playback/src/lib.rs:270`: internal
  `ClientPlaybackCapabilities` remains a flat capability record.
- `crates/nako-playback/src/capability.rs:119`: `PlaybackTargetProfile` is
  profile-shaped inside the planner.
- `crates/nako-playback/src/capability.rs:136`: `from_capabilities` maps the
  flat record to one direct-play profile, one remux profile, and one HLS
  H264/AAC transcode profile.
- `crates/nako-server/src/http/playback.rs:947`: browser ticket body
  capabilities map into internal `ClientPlaybackCapabilities`.
- `crates/nako-server/src/http/playback.rs:1366`: public playback/remux/HLS
  query fields cover the current flat capability set.
- `crates/nako-server/src/http/playback.rs:1496`: query fields map into
  internal `ClientPlaybackCapabilities`.
- `crates/nako-server/src/http/renderer.rs:396`: renderer DTO capabilities map
  into internal `ClientPlaybackCapabilities`.
- `crates/nako-api/src/public_client.rs:1102`: persisted session capability JSON
  maps back to `ClientPlaybackCapabilitiesDto`.
- `crates/nako-api/src/openapi.rs:1432` and `:1599`: OpenAPI schemas expose
  browser and client playback capability DTOs.
- `crates/nako-api/src/admin/playback.rs:65`: Admin runtime diagnostics expose
  readiness, policy, FFmpeg, hardware, resource, staging, artifact, and throttle
  facts.
- `crates/nako-api/src/admin/playback.rs:367`: Admin support evidence exposes
  source/session/runtime summaries and redaction flags.
- `crates/nako-api/src/admin/playback.rs:426`: source support evidence keeps a
  redacted source summary instead of raw locators.
- `crates/nako-server/src/http/admin.rs:2738`: Admin support evidence adapts
  runtime diagnostics into a redaction-safe response.
- `crates/nako-server/src/http/admin.rs:3233`: support runtime evidence only
  includes summarized unavailable hardware facts.
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`: Public
  Client DTOs and Admin runtime settings/diagnostics are separate boundaries.
- `docs/adr/0044-playback-capability-profile-planner.md`: Public Client DTOs
  map into Nako's profile model; they do not become the domain model.
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`: hardware and FFmpeg
  stage details belong to transcode/Admin diagnostics, not Public Client API.
- `docs/adr/0053-application-control-plane-boundary.md`: diagnostics must be
  useful and redacted.

## Current Public Client Contract

### Query Shape

Public playback, remux, and HLS query entry points currently support:

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

Remux query adds `output_container`. HLS query adds request controls such as
`start_position_ms`, audio/subtitle stream selection, preferred languages, and
ticket/session parameters.

### Body And Response Shape

`BrowserPlaybackTicketRequest.capabilities` uses optional fields so browser
clients can override only a subset of defaults for a one-shot ticket request.
Renderer registration and heartbeat use `ClientPlaybackCapabilitiesDto`, where
the four base fields are required and the newer limit/HDR/subtitle/HLS fields
are optional.

Decision responses are stronger than capability inputs. `ClientPlaybackDecision`
and `ClientPlaybackDecisionReport` already expose typed direct/remux/transcode
evaluations, and `ClientPlaybackCompatibilityCondition` includes useful reasons
such as unsupported bitrate, resolution, HDR, audio channels, subtitle delivery,
and transcode profile.

## Current Admin Contract

Admin playback routes currently cover runtime diagnostics, support evidence,
sessions, and renderer diagnostics.

Admin exposes:

- playback readiness, policy, FFmpeg probe status, and hardware inventory
  summary;
- hardware policy, selected pipeline, fallback, stage capabilities, encoder
  discovery, device initialization, and smoke probes;
- CPU/GPU transcode slots, remux concurrency, resource pressure, remote
  stream/stage budgets, staging, artifact lifecycle, and throttle;
- session summaries with `has_client_capabilities`, not full raw client JSON;
- renderer summaries with supported commands, media capability presence, and
  direct-play support;
- support evidence with source/session/runtime summaries and explicit redaction
  flags.

Admin does not currently expose:

- resolved `PlaybackTargetProfile`;
- a profile/device family name or version;
- per-session effective profile summaries;
- a source-vs-client compatibility matrix for support evidence;
- a public contract parity status across generated clients and docs.

Those gaps should be addressed through redaction-safe Admin support summaries,
not by returning raw client capability JSON, source locators, host paths,
FFmpeg commands, stderr, tokens, secrets, or device paths.

## Contract Gaps

1. Flat fields cannot express condition matrices.
   Separate container, video codec, and audio codec lists cannot say that a
   specific container only supports a specific codec combination, that HEVC
   Main10 only works through HLS fMP4, or that a subtitle format is sidecar-only.

2. Device family and profile identity are missing.
   There is no `profile_id`, `profile_version`, `device_family`,
   `player_engine`, capability source, or compatibility-mode field.

3. HDR input is too coarse.
   `supports_hdr: bool` cannot distinguish SDR, HDR10, HLG, Dolby Vision,
   HDR10+, transfer function, bit depth, or tone-map target preference.

4. Subtitle input is too coarse.
   `supports_subtitles: bool` cannot distinguish WebVTT, SRT, ASS/SSA, PGS,
   DVD subtitles, sidecar delivery, embedded delivery, browser text tracks,
   image subtitle burn-in, or font/style support.

5. Audio output input is too coarse.
   `max_audio_channels` is useful, but it does not model codec-specific
   passthrough, downmix, dynamic range compression, normalization, dialogue
   clarity, or output-device behavior.

6. HLS output policy is still narrow.
   Current client fields only cover single/adaptive and MPEG-TS/fMP4. They do
   not express HEVC/AV1 output, LL-HLS/CMAF, DASH/CMAF, ladder constraints, or
   rendition quirks.

7. Contract coverage drift already exists.
   The server query shape, OpenAPI, and Rust client support the full current
   field set. `nako-client-core`, parts of the Kotlin SDK query surface, and
   `docs/api/HTTP_API.md` still reflect a narrower four-field shape.

## Recommended Boundaries

### Public Client API

Public Client API should continue to accept only client/player capability facts
and request preferences. It should not accept server hardware facts, FFmpeg
facts, operator policy, resource pressure, or host diagnostics.

The next additive profile shape should keep the existing fields as a v1
simplified entry point and add optional structured facts such as:

- `profile_id`
- `profile_version`
- `device_family`
- `player_engine`
- direct-play profile rows
- remux profile rows
- transcode output profile rows
- subtitle delivery profile rows
- audio output facts
- color pipeline facts
- HLS output facts

The DTOs belong to `nako-client-protocol` and should map into
`nako-playback` domain records. They should not become the domain model.

### Admin API

Admin API should expose runtime and support evidence, not client request
inputs. A future support evidence slice should add an
`effective_profile_summary` that includes:

- profile fingerprint;
- device family;
- direct/remux/transcode profile counts;
- selected output profile summary;
- reason counts;
- whether client capabilities were provided, defaulted, or invalid.

### Playback, Transcode, And Server

- `nako-playback` owns pure planning, target profiles, compatibility
  conditions, and audio/color/subtitle/output requirements.
- `nako-transcode` owns runtime hardware capability reports, pipeline planning,
  transcode profile identity, artifact identity, and FFmpeg command planning.
- `nako-server` owns HTTP mapping, access/policy resolution, orchestration,
  persistence, runtime diagnostics, and Admin support adaptation.
- `nako-api` and `nako-client-protocol` own wire DTOs.

## Recommended Follow-Ons

1. `public-client-playback-capability-contract-parity-gate`
   - No behavior change.
   - Align `nako-client-protocol`, OpenAPI, Rust client, `nako-client-core`,
     Kotlin/TypeScript SDK query surfaces, and `docs/api/HTTP_API.md`.
   - Add contract tests so current fields do not drift again.

2. `playback-output-profile-v2-skeleton-contract-only`
   - Add optional profile/device-family skeleton fields.
   - Map legacy flat fields into a `legacy_default` profile row.
   - Keep behavior unchanged when new fields are absent.

3. `admin-playback-effective-profile-support-evidence`
   - Add redaction-safe effective profile and decision-matrix summaries to
     Admin support evidence.

4. `browser-mobile-tv-profile-fixtures`
   - Add browser, native mobile, native desktop, TV, and renderer profile
     fixtures after the contract skeleton is stable.

5. `hevc-av1-output-policy-design-before-execution`
   - Decide whether HEVC/AV1 is a Public Client output profile fact, an Admin
     operator policy fact, or a runtime capability fact before command
     execution is enabled.

## Unsafe Parallel Work

- Multiple tasks editing Public Client playback DTOs, query mapping, generated
  clients, and docs at the same time.
- HEVC/AV1 output execution in parallel with hardware tone-map execution.
- Admin support evidence expansion in parallel with Public Client profile DTO
  expansion without a contract owner.
- Concurrent changes to HLS request identity, `PlaybackTargetProfile::identity`,
  or `TranscodeProfile` identity.

## Changes Made By This Research

This research changed only this task's research documentation. It did not
modify production Rust, TypeScript, DTOs, routes, OpenAPI generation, SDK
generation, or tests.
