# Playback Profile Resolution

## Goal

Make Public Client playback capability profiles product-usable by resolving
known `device_family` + `profile_version` pairs into server-owned playback
capability baselines before planning playback. Client Applications should be
able to send a compact profile identity and only override the fields that differ
for the actual player.

## What I Already Know

- `GET /playback/profile-presets` already exposes authenticated, redaction-safe
  playback capability templates.
- `nako-playback` already owns `PlaybackProfileFamily`,
  `PlaybackProfilePreset`, and `playback_profile_presets()`.
- Public playback query/body DTOs already include `device_family` and
  `profile_version`.
- `crates/nako-server/src/http/playback.rs` currently converts playback
  capability query fields by starting from `ClientPlaybackCapabilities::default()`
  and then applying explicit flat fields. That preserves the profile identity
  but does not apply the preset's containers, codecs, HDR, subtitle, or HLS
  defaults.
- Previous playback reason work added safe reason detail projection; this task
  should not change planner reason semantics.

## Requirements

- Resolve known playback profile identity on server input boundaries:
  - `GET /sources/{source_id}/playback/decision`
  - `GET|HEAD /sources/{source_id}/stream/remux`
  - `GET /sources/{source_id}/stream/hls/playlist.m3u8`
  - `POST /sources/{source_id}/playback/browser-ticket`
- A known `device_family` with the current preset version must use that preset
  as the baseline capability set.
- Explicit request fields must override the preset baseline field by field:
  `direct_play`, `container`, `video_codec`, `audio_codec`,
  `max_video_bitrate`, `max_width`, `max_height`, `max_audio_channels`,
  `supports_hdr`, `supports_subtitles`, `hls_variant_policy`, and
  `hls_segment_container`.
- Preserve additive compatibility:
  - unknown or blank `device_family` must not reject the request;
  - unknown `device_family` should retain a normalized safe identity but use
    default capabilities;
  - unsupported `profile_version` must not reject the request and must not
    silently apply a mismatched preset.
- Preserve existing invalid-value behavior for unsupported HLS policy/container
  enum values in browser ticket bodies.
- Keep all public fields snake_case and keep existing flat field names.
- Do not add a database table, persisted preference, Admin mutation, new route,
  device detection, frontend UI, localization, or planner behavior change.
- Do not expose Source Locators, local paths, bearer tokens, FFmpeg commands,
  raw probe payloads, raw policy internals, or backend errors.

## Acceptance Criteria

- [ ] A playback decision query with
  `device_family=browser_chromium&profile_version=1` and no codec/container
  fields plans against Chromium preset capability values.
- [ ] Explicit flat fields override the preset baseline for playback decision,
  remux, HLS, and browser ticket inputs.
- [ ] Unknown profile family preserves normalized identity and defaults rather
  than applying a known preset or rejecting the request.
- [ ] Version mismatch preserves identity and defaults rather than applying the
  current preset.
- [ ] Browser ticket capability conversion uses the same profile-resolution
  semantics as query routes while preserving browser output-container handling.
- [ ] Existing Public Client OpenAPI/SDK shapes do not change.
- [ ] Tests cover redaction-sensitive behavior by asserting no raw paths,
  bearer tokens, FFmpeg terms, or policy internals appear in affected response
  bodies.

## Definition Of Done

- `cargo fmt --all`
- `cargo nextest run -p nako-playback profile --no-fail-fast`
- `cargo nextest run -p nako-server playback_capability --no-fail-fast`
- `cargo nextest run -p nako-server playback_profile --no-fail-fast`
- `cargo check -p nako-playback -p nako-api -p nako-server --tests`
- Trellis context validates.
- Task is committed and archived without staging unrelated user changes.

## Technical Approach

- Add a protocol-free helper in `nako-playback` that resolves
  `ClientPlaybackCapabilities` from request-shaped partial capability input:
  known current preset baseline first, then explicit overrides.
- Keep the helper dependency-light and usable by `nako-server` HTTP conversion
  code.
- Replace `PlaybackCapabilitiesQuery -> ClientPlaybackCapabilities` and browser
  ticket body conversion with the shared helper.
- Keep OpenAPI and generated SDK artifacts unchanged; this task changes server
  interpretation, not wire shape.
- Add focused tests in `nako-playback` for baseline/override/unknown/version
  behavior and in `nako-server` for HTTP query/body conversion.

## Decision (ADR-lite)

**Context**: Profile presets were discoverable but not operational. Clients
still had to copy every field from a preset into playback query/body requests
for Direct Play/Remux/Transcode planning to behave as advertised.

**Decision**: Resolve profile identity server-side at playback request
boundaries. Keep flat v1 fields as override fields and avoid adding a new route
or nested profile contract in this slice.

**Consequences**: Clients can send compact profile identity plus overrides.
Public API shape remains compatible. A future richer profile database or
per-user/device preference model can replace the preset lookup behind the same
resolution boundary.

## Out Of Scope

- New nested profile DTOs.
- Device autodetection or User-Agent parsing.
- Persisted user/device playback preferences.
- Admin profile editor or policy UI.
- Addon-provided playback profiles.
- Frontend rendering changes.
- Transcode Profile or Hardware Acceleration Policy changes.
- Planner compatibility reason semantics beyond consequences of receiving
  richer client capability facts.

## Technical Notes

- Terminology: use Client Applications, Public Client API, Playback Runtime,
  Playback Source Selection, and Media Technical Facts from `CONTEXT.md`.
- Architecture: `docs/architecture/PLAYBACK.md` Lane A says device capability
  profiles should provide codec/container/subtitle/HDR/audio facts for the
  planner.
- Spec: `.trellis/spec/nako-client-protocol/backend/index.md` currently keeps
  Public Client playback capabilities on the flat v1 field set until a
  dedicated profile contract task changes it; this task does not change that
  field set.
- Current server conversion lives in
  `crates/nako-server/src/http/playback.rs`.
- Current preset ownership lives in `crates/nako-playback/src/capability.rs`.
