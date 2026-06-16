# feat: Playback profile preset diagnostics

## Goal

Turn the newly added Public Client playback profile identity
(`device_family`, `profile_version`) into a backend-observable product
contract. Nako should recognize a small built-in set of safe client profile
families and expose redaction-safe diagnostics for playback sessions, without
using profile names as hidden compatibility rules.

## What I Already Know

* `device_family` and `profile_version` are now additive Public Client playback
  capability fields across protocol DTOs, OpenAPI, generated SDKs, Rust
  clients, server query/body mapping, renderer mapping, browser tickets, and
  `PlaybackTargetProfile::identity()`.
* The existing flat v1 capability fields remain the source of truth for
  playback planning: containers, codecs, HDR, subtitles, bitrate, resolution,
  HLS policy, and direct-play preference.
* `docs/architecture/PLAYBACK.md` names "Device Capability Profiles" as a
  playback lane, with browser/mobile/TV/renderer capability records and precise
  compatibility reasons as exit criteria.
* The backend maturity plan says playback reasons and capability facts must be
  shared between planning, Admin diagnostics, Public Client responses, and
  future device profile work.
* Admin playback session list currently exposes only
  `has_client_capabilities`, not which profile family/version was supplied.

## Requirements

* Add a typed built-in playback profile family vocabulary for the first
  supported families: `browser_chromium`, `browser_firefox`,
  `browser_safari`, `android_media3`, `desktop_native`, `tv_webos`,
  `tv_tizen`, `chromecast`, `dlna_renderer`, and `unknown`.
* Recognize profile family names from `ClientPlaybackCapabilities.device_family`
  with the same normalization behavior used by playback profile identity:
  trim, lowercase ASCII, and replace unsupported characters with `_`.
* Keep profile family recognition descriptive only. It must not change Direct
  Play, Remux, Transcode, Denied, transcode policy, FFmpeg planning, or storage
  staging decisions in this task.
* Expose safe Admin playback session diagnostics for supplied profile facts:
  normalized `device_family`, `profile_version`, and recognized family enum.
* Keep raw host/runtime details out of profile diagnostics: no paths, Source
  Locators, playback tickets, bearer tokens, FFmpeg commands, GPU device names,
  local hardware identifiers, backend URLs, or client user-agent strings.
* Preserve old sessions and old clients: missing or malformed capabilities
  should produce absent/default diagnostic facts rather than failing session
  listing.

## Acceptance Criteria

* [x] `nako-playback` exposes a typed profile family recognition helper with
      tests for known, unknown, normalized, and absent `device_family` values.
* [x] `AdminPlaybackSessionListItem` exposes redaction-safe profile facts for
      sessions with serialized client capabilities.
* [x] Admin session list mapping handles missing or unreadable capability JSON
      without leaking parse errors or failing the whole response.
* [x] Existing Public Client playback decision, browser ticket, renderer, and
      SDK contracts remain unchanged apart from already-shipped profile fields.
* [x] Focused playback and server Admin playback tests pass.

## Definition of Done

* Focused `nako-playback` and `nako-server` tests pass.
* `cargo check` passes for touched crates.
* Formatting and `git diff --check` pass.
* Trellis task validates and is updated with verification evidence.
* Commit uses a Conventional Commit message.

## Technical Approach

Add a small pure helper in `nako-playback` near capability profile code:

* `PlaybackProfileFamily` enum for known safe families plus `Unknown`.
* `PlaybackProfileFamily::from_device_family(Option<&str>) -> Option<Self>`
  or equivalent.
* Reuse or share the normalization behavior already used for
  `PlaybackTargetProfile` identity so profile diagnostics and request identity
  agree.

Then map persisted `PlaybackSessionRecord.client_capabilities_json` into Admin
session list DTO fields. The Admin surface should expose only normalized
profile facts and the recognized family enum. This is deliberately a diagnostic
contract, not a planner behavior change.

## Decision (ADR-lite)

**Context**: Nako now accepts client-owned profile identity, but operators
cannot yet tell which profile family a session used. Jumping straight to a full
Jellyfin-style device profile database would over-scope the next slice and
risk hiding compatibility rules behind profile names.

**Decision**: Start with a typed, descriptive profile family recognition layer
and Admin session diagnostics. Keep all playback planning decisions tied to
explicit flat v1 capability facts.

**Consequences**: Admin diagnostics and future clients can group sessions by
profile family immediately. Later tasks can add server-known preset capability
templates or richer client feature detection without breaking the current
contract.

## Implementation Summary

* Added `PlaybackProfileFamily` and `normalize_playback_device_family` to
  `nako-playback` as pure profile identity helpers.
* Added Admin playback session profile diagnostics:
  `client_device_family`, `client_profile_version`, and
  `client_profile_family`.
* Made Admin diagnostics parse only the safe profile fields from persisted
  capability JSON so old query-shaped capability payloads still work.
* Suppressed unknown raw device family strings in Admin output while still
  exposing `client_profile_family: "unknown"` for grouping.
* Synchronized generated Admin TypeScript contracts and mock/test fixtures.

## Spec Sync

Updated `.trellis/spec/nako-api/backend/quality-guidelines.md` with the
"Admin Playback Session Profile Diagnostics" scenario so future Admin playback
session DTO changes preserve safe profile projection, generated contract
sync, malformed capability tolerance, and no hidden planner behavior.

## Verification Evidence

* `cargo check -p nako-playback -p nako-api -p nako-server --tests` passed.
* `cargo check -p nako-api --examples` passed.
* `cargo nextest run -p nako-playback playback_profile_family playback_target_profile_identity --no-fail-fast` passed.
* `cargo nextest run -p nako-api admin_playback_session_list_item admin_contract --no-fail-fast` passed.
* `npm run check --prefix apps/admin-web` passed.
* `npm run check --prefix web` passed.
* `cargo fmt --all -- --check` passed.
* `git diff --check` passed.
* `python ./.trellis/scripts/task.py validate 06-17-playback-profile-preset-diagnostics` passed.

## Out of Scope

* Server-owned device profile database or compatibility matrix.
* Automatic browser/user-agent detection.
* Changing playback decision rules based on `device_family`.
* Public Client route changes beyond the already shipped profile fields.
* Admin Web UI changes.
* Database migrations.
* Hardware/FFmpeg/GPU diagnostics expansion.

## Technical Notes

* Relevant specs/docs:
  * `CONTEXT.md`
  * `docs/architecture/PLAYBACK.md`
  * `.trellis/spec/nako-playback/backend/index.md`
  * `.trellis/spec/nako-playback/backend/quality-guidelines.md`
  * `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  * `.trellis/spec/nako-api/backend/quality-guidelines.md`
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
* Likely code areas:
  * `crates/nako-playback/src/capability.rs`
  * `crates/nako-playback/src/lib.rs`
  * `crates/nako-api/src/admin/playback.rs`
  * `crates/nako-server/src/http/admin.rs`
  * `crates/nako-api/src/admin_contract.rs`
  * generated Admin TypeScript contracts if DTO shape changes.
