# feat: Playback profile preset capability preview

## Goal

Expose built-in playback profile preset capability templates as a read-only
Admin diagnostics surface. Operators and future clients should be able to see
what Nako recommends for known profile families, while actual playback
planning continues to use explicit flat capability facts from the request.

## What I Already Know

* `device_family` and `profile_version` are additive Public Client playback
  capability fields and are included in `PlaybackTargetProfile::identity()`.
* `PlaybackProfileFamily` already recognizes safe families such as
  `browser_chromium`, `browser_firefox`, `browser_safari`, `android_media3`,
  `desktop_native`, TV families, Chromecast, and DLNA renderer.
* Admin session diagnostics already expose normalized profile family/version
  facts and suppress unknown raw `device_family` text.
* The playback architecture map names Device Capability Profiles as Lane A,
  but the current flat fields remain the authority for Direct Play, Remux,
  Transcode, and Denied decisions.

## Requirements

* Add a pure `nako-playback` preset catalog for known playback profile
  families. Each preset must expose a deterministic capability template made
  from the same flat fields clients can already report: direct play, container
  list, video codec list, audio codec list, HDR/subtitle support, HLS variant
  policy, and HLS segment container.
* Include safe preset identity facts in each template: recognized profile
  family, normalized `device_family`, and `profile_version`.
* Exclude `unknown` from preset templates. Unknown raw family strings must not
  become echoed presets.
* Keep presets descriptive and opt-in for callers. Profile family names must
  not change playback decisions, transcode policy, FFmpeg command planning,
  storage staging, or runtime admission in this task.
* Expose the preset catalog through Admin playback runtime diagnostics as a
  read-only preview so Admin Web and operators can inspect server-known
  capability recommendations.
* Keep Public Client route/DTO/OpenAPI/SDK surfaces unchanged in this task.
* Keep Admin output redaction-safe: no source locators, local paths, playback
  tickets, bearer tokens, user-agent strings, FFmpeg commands, GPU device
  names, backend URLs, or raw client strings.

## Acceptance Criteria

* [ ] `nako-playback` exposes a typed preset catalog for known
      `PlaybackProfileFamily` values, with tests proving coverage, identity
      facts, non-empty capability fields, and no preset for `Unknown`.
* [ ] Admin playback runtime diagnostics include a `profile_presets` preview
      with only safe preset facts and flat capability values.
* [ ] Existing playback planner behavior is unchanged when a profile family is
      present or absent; presets are not consulted by decision selection.
* [ ] Admin TypeScript contracts under `apps/admin-web` and `web` are
      regenerated from `nako-api` when the Admin DTO shape changes.
* [ ] Focused Rust and TypeScript checks pass for touched crates/apps.

## Definition of Done

* Focused `nako-playback`, `nako-api`, and `nako-server` checks/tests pass.
* Generated Admin contract artifacts are synchronized.
* Formatting and whitespace gates pass.
* Trellis task validates and records verification evidence.
* Commit uses a Conventional Commit message.

## Technical Approach

Implement a pure preset catalog in `crates/nako-playback/src/capability.rs`
near the existing profile family helpers. The catalog should produce owned
capability templates so callers can pass them through API DTO mapping without
sharing mutable state.

Map those presets into new Admin DTOs in `crates/nako-api/src/admin/playback.rs`
and add the field to `AdminPlaybackRuntimeDiagnosticsResponse`. The server
runtime diagnostics handler should fill the field from the pure catalog only;
it should not query storage, inspect user agents, or look at active sessions.

Regenerate Admin TypeScript contracts from the API generator and update mock or
fixture data only where the new required Admin response field is consumed.

## Decision (ADR-lite)

**Context**: Nako has client-reported profile identity, but there is not yet a
server-owned way to preview what known client families should report. Jumping
straight to a device profile database would make profile names look
authoritative before the planner has a full compatibility matrix.

**Decision**: Add a small built-in preset catalog and expose it only through
Admin read-only diagnostics. Keep request flat capability fields as the sole
planner authority.

**Consequences**: Operators can inspect and compare profile recommendations
immediately, while future work can add public discovery or richer device
profile evolution without breaking the current planning contract.

## Implementation Summary

* Added `PlaybackProfilePreset` and `playback_profile_presets()` to
  `nako-playback`, covering known browser, mobile, desktop, TV, cast, and DLNA
  profile families while excluding `Unknown`.
* Added Admin `AdminPlaybackProfilePresetDiagnostic` plus explicit HLS preset
  enums and exposed the catalog through
  `AdminPlaybackRuntimeDiagnosticsResponse.profile_presets`.
* Mapped the server Admin playback runtime route from the pure preset catalog,
  with no storage/runtime/user-agent inspection and no planner behavior change.
* Regenerated Admin TypeScript contracts for `apps/admin-web` and `web`, and
  updated mock/test fixtures for the new required field.
* Updated the API code-spec with the Admin Playback Profile Preset Diagnostics
  contract.

## Verification Evidence

* `cargo check -p nako-playback -p nako-api -p nako-server --tests` passed.
* `cargo check -p nako-api --examples` passed.
* `cargo nextest run -p nako-playback playback_profile_preset playback_profile_family playback_target_profile_identity --no-fail-fast` passed.
* `cargo nextest run -p nako-api admin_playback_runtime_diagnostics admin_contract --no-fail-fast` passed.
* `cargo nextest run -p nako-server admin_v1_playback_runtime_reports_safe_diagnostics --no-fail-fast` passed.
* `npm run check --prefix apps/admin-web` passed.
* `npm run check --prefix web` passed.
* `cargo fmt --all -- --check` passed.
* `git diff --check` passed.
* `python ./.trellis/scripts/task.py validate 06-17-playback-profile-preset-capability-preview` passed.

## Out of Scope

* Public Client discovery route or SDK changes.
* Server-owned device profile database.
* Automatic browser/user-agent detection.
* Applying presets automatically to playback decisions.
* Database migrations.
* Admin Web UI beyond generated contract/mock compatibility.
* Hardware, FFmpeg, GPU, or runtime resource diagnostics expansion.

## Technical Notes

* Relevant docs/specs:
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
  * `crates/nako-api/src/admin_contract.rs`
  * `crates/nako-server/src/http/admin.rs`
  * generated Admin TypeScript contracts.
