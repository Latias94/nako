# Playback / Transcode / Streaming Architecture Report

Scope: short read-only review for the 10-hour media-server campaign. This
report prioritizes product-visible playback reliability and self-hosted
operability over broad refactoring.

## Evidence Read

- `docs/architecture/PLAYBACK.md`: Direct Play/Remux/HLS matrix, HLS runtime
  lifecycle, runtime resource scheduler, VFS/remote resilience, release
  packaging, web player integration, risk register.
- `docs/architecture/STORAGE_VFS.md`: remote probe/FFmpeg input staging, cache
  and staging cleanup, playback artifact I/O pressure follow-ons.
- `docs/architecture/LANES.md`: `playback-transcode` is idle and next actions
  include resource admission queueing, LL-HLS/CMAF, player UX, hardware
  tone-map execution, HEVC/AV1 output policy, subtitle burn-in, and hardware
  smoke evidence.
- Small code/test-name evidence only:
  - `crates/nako-server/src/app/playback/resource.rs`
  - `crates/nako-server/src/app/playback/hls_flow.rs`
  - `crates/nako-server/src/app/playback/hls_artifact.rs`
  - `crates/nako-server/src/app/playback/support.rs`
  - `crates/nako-server/src/http/playback.rs`
  - `crates/nako-playback/src/capability.rs`
  - `crates/nako-transcode/src/hardware.rs`
  - `crates/nako-transcode/src/profile.rs`
  - Tests: `admin_v1_playback_runtime_reports_active_resource_pressure`,
    `hls_segment_waits_once_for_running_segment_when_throttle_enabled`,
    `direct_stream_route_records_playback_session_without_transcode_artifact`,
    `hls_playlist_route_accepts_preferred_audio_language_defaults`,
    `transcode_profile_validation_rejects_hevc_and_av1_as_deferred_outputs`,
    `self_host_smoke_sqlite_operator_flow_redacts_sensitive_boundaries`.

## Ranked Opportunities

### 1. Playback Transcode Runtime Session Module

Type: refactor + reliability.

User-visible value: fewer HLS/remux startup races, clearer cancellation/reuse,
and better playback session correlation when a user seeks, retries, refreshes,
or switches clients.

Evidence:

- `PLAYBACK.md` says Remux orchestration already has
  `app/playback/remux_flow.rs`, renderer orchestration has
  `app/playback/renderer_flow.rs`, and HLS lifecycle invariants are frozen.
- `PLAYBACK.md` still lists durable queueing, remote workers, LL-HLS/CMAF, and
  player UX as follow-ons.
- `hls_flow.rs`, `hls.rs`, `hls_artifact.rs`, and `support.rs` are separate
  runtime session concerns: startup, cancellation/supersede, artifact serving,
  diagnostics.

10-hour slice:

- Extract a focused server-side Playback Transcode Runtime session module that
  owns HLS session start/reuse/supersede/cancel correlation and leaves HTTP,
  FFmpeg command planning, and pure playback planning untouched.
- Keep public DTOs and schema unchanged.
- Add regression coverage around active-session reuse, supersede cancellation,
  failed startup correlation, and playback-session-to-transcode linkage.

Parallelism:

- Can run in parallel with player UX/client capability work if no DTO changes.
- Must serialize with remote stage/artifact pressure if both edit
  `app/playback/hls_flow.rs` or `app/playback/resource.rs`.

Verification:

- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server remux --no-fail-fast`
- `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`

Stop conditions:

- Stop if the slice needs schema changes, public API changes, or durable job
  semantics.
- Stop if the module starts rebuilding FFmpeg argv or playback decisions.
- Stop if it weakens existing Direct Play -> Remux -> Transcode preference.

### 2. Remote Stage / HLS Artifact I/O Pressure

Type: reliability + product operations.

User-visible value: fewer failed starts and fewer stalled streams on NAS,
WebDAV, rclone-like mounts, and small self-hosted disks; Admin can explain
whether playback is blocked by stream, stage, transcode, or artifact pressure.

Evidence:

- `PLAYBACK.md` marks VFS/remote playback resilience as partial and calls out
  resource admission queueing plus per-artifact read/write pressure as
  follow-ons.
- `STORAGE_VFS.md` lists remote probe staging and remote FFmpeg input staging
  as shipped foundations, with per-backend staging budgets and diagnostics as
  follow-ons.
- `resource.rs` already models `RemoteStream`, `RemoteStage`,
  `CpuTranscode`, `GpuTranscode`, and `HlsArtifactIo`.
- `hls_artifact.rs` has manifest-backed segment serving and a one-shot segment
  wait test, but not a per-artifact read/write waitlist.
- Admin diagnostics already have
  `admin_v1_playback_runtime_reports_active_resource_pressure`.

10-hour slice:

- Add a first read/write pressure slice for HLS artifact serving: classify
  segment-read pressure separately from transcode-start pressure, expose
  redaction-safe diagnostics, and preserve manifest allow-list behavior.
- Prefer process-local bounded admission first; defer durable queues.

Parallelism:

- Coordinates tightly with storage/VFS. Can run beside hardware/release smoke.
- Should serialize with Playback Transcode Runtime session Module if both touch
  HLS startup/admission.

Verification:

- `cargo nextest run -p nako-server hls_segment --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_playback_runtime_reports_active_resource_pressure --no-fail-fast`
- `cargo check -p nako-server --tests`

Stop conditions:

- Stop if raw Source Locators, local paths, backend URLs, etags, fingerprints,
  or FFmpeg argv would enter diagnostics.
- Stop if Direct Play remote stream behavior silently changes from fast-fail to
  unbounded wait.
- Stop if storage schema or durable queue tables become necessary.

### 3. Player UX / Client Capability / Device Profile First Slice

Type: feature + planner hardening.

User-visible value: browser, TV, mobile, and renderer clients get more correct
Direct Play/Remux/HLS decisions, clearer reasons, and fewer wrong transcodes.

Evidence:

- `PLAYBACK.md` lists Device Capability Profiles as a parallel lane and Web
  player integration follow-ons include capability reporting, richer retry UX,
  and desktop/native decisions.
- `nako-playback/src/capability.rs` is the pure capability planner.
- `http/playback.rs` already accepts browser capability query/DTO fields such
  as container, codecs, HDR, subtitles, HLS variant policy, segment container,
  language preferences, and `start_position_ms`.
- Existing tests cover preferred audio/subtitle language defaults.

10-hour slice:

- Add named built-in client capability presets for Browser Default, Conservative
  TV, Mobile Low Bandwidth, and Renderer Conservative inside the existing
  planner/API mapping path.
- Improve decision reasons and response shape only if current DTOs can carry
  them; otherwise keep this backend-internal and defer DTO changes.

Parallelism:

- Can run in parallel with hardware/release smoke.
- Must serialize with public client DTO/generated SDK work and web player UI
  if the slice expands beyond backend mapping.

Verification:

- `cargo nextest run -p nako-playback capability --no-fail-fast`
- `cargo nextest run -p nako-server playback_decision --no-fail-fast`
- `cargo check -p nako-api -p nako-playback -p nako-server --tests`

Stop conditions:

- Stop if generated client contracts are required but not planned.
- Stop if profile presets become user preferences requiring persistence.
- Stop if the planner stops being deterministic or starts reading runtime
  storage/HTTP facts directly.

### 4. Hardware / Release Smoke Evidence

Type: operations + reliability.

User-visible value: operators know before playback whether FFmpeg, ffprobe,
hardware encoders, drivers, and fallback behavior are usable in their
self-hosted environment.

Evidence:

- `PLAYBACK.md` says release packaging is partial and release gates should
  validate FFmpeg/ffprobe presence, hardware diagnostics, Docker device docs,
  and CPU fallback smoke.
- `nako-transcode/src/hardware.rs` models `Hardware Capability Report`,
  hardware smoke probe state, and accelerator diagnostics.
- `nako-transcode/src/profile.rs` recognizes HEVC/AV1 HLS output policy but
  rejects them as deferred executable outputs.
- `self_host_smoke_sqlite_operator_flow_redacts_sensitive_boundaries` provides
  an existing smoke-test style.

10-hour slice:

- Add a non-invasive release smoke target/script or nextest gate that validates
  FFmpeg/ffprobe discovery, CPU HLS readiness, redaction of local paths, and
  hardware report serialization using existing diagnostics.
- Keep real GPU execution optional and reported as not-run/skipped when the
  runner lacks devices.

Parallelism:

- Highly parallel with player capability and runtime module work.
- Coordinates with operations-release for docs/checklist naming.

Verification:

- `cargo nextest run -p nako-transcode hardware --no-fail-fast`
- `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`
- `cargo check -p nako-transcode -p nako-server --tests`

Stop conditions:

- Stop if the smoke requires host-specific GPU devices in normal CI.
- Stop if it starts promising HEVC/AV1 executable HLS output before the encoder
  execution slice exists.
- Stop if diagnostics leak FFmpeg path, local cache roots, or media locators.

### 5. Seek / Player Polish Over LL-HLS/CMAF

Type: feature.

User-visible value: seeking, refresh, and retry behavior improves for existing
HLS playback without taking on a new streaming protocol surface.

Evidence:

- `PLAYBACK.md` says HLS seek/restart has a shipped first slice, while
  generation identity, restart admission, FFmpeg seek flags, and public
  `start_position_ms` behavior remain follow-ons.
- `LANES.md` lists LL-HLS/CMAF and player UX as possible next actions.
- `http/playback.rs` already has `start_position_ms` in HLS query handling.

Recommendation:

- Prefer seek/player polish for the 10-hour campaign.
- Defer LL-HLS/CMAF because it changes segment timing, playlist update
  semantics, client compatibility, cache behavior, and probably artifact
  lifecycle assumptions. It is a larger protocol lane, not the best first
  10-hour product slice.

10-hour slice:

- Harden seek restart behavior for existing HLS: explicit tests for
  `start_position_ms`, session reuse versus supersede, stale segment denial,
  and user-visible error messaging.
- Do not add LL-HLS partial segments or CMAF packaging.

Parallelism:

- Can follow the Playback Transcode Runtime session Module.
- Should not run concurrently with HLS artifact I/O pressure if both edit
  segment readiness and session-generation logic.

Verification:

- `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
- `cargo nextest run -p nako-transcode hls --no-fail-fast`
- `cargo check -p nako-transcode -p nako-server --tests`

Stop conditions:

- Stop if correct behavior requires keyframe/GOP policy changes not already
  modeled.
- Stop if FFmpeg seek flag changes lack exact argv tests.
- Stop if LL-HLS/CMAF partial segment behavior starts entering scope.

### 6. Subtitle Burn-In Follow-On For Anime/TV

Type: feature + compatibility reliability.

User-visible value: ASS/SSA and future image subtitles stop being mysterious
playback failures on clients that cannot render sidecars.

Evidence:

- `PLAYBACK.md` says HLS subtitle sidecar and burn-in planning shipped a first
  slice; image subtitle execution, external subtitle burn-in, hardware-filter
  burn-in, and richer client subtitle profiles remain follow-ons.
- `nako-playback/src/capability.rs` owns sidecar versus burn-in intent.
- `nako-transcode` owns FFmpeg HLS command planning and exact argv tests.

10-hour slice:

- Add one narrow executable burn-in case or one negative capability slice, not
  the whole subtitle matrix. Best first slice: harden diagnostics and tests for
  known image/external subtitle unsupported paths, then choose PGS/image
  execution later.

Parallelism:

- Can run beside hardware/release smoke.
- Serializes with client capability/device profile work if richer subtitle
  profile facts enter DTOs.

Verification:

- `cargo nextest run -p nako-playback subtitle --no-fail-fast`
- `cargo nextest run -p nako-transcode hls --no-fail-fast`
- `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`

Stop conditions:

- Stop if the slice requires external subtitle file discovery or Library File
  Write behavior.
- Stop if it needs hardware-filter burn-in.
- Stop if it broadens into client subtitle profile DTO changes without a
  contract task.

## Recommended 10-Hour Campaign Shape

Primary lane: Playback Transcode Runtime session Module.

Parallel support lanes:

- Remote stage/artifact I/O pressure, only if scoped away from session
  refactor files or sequenced after the session module lands.
- Hardware/release smoke, safe to run independently.
- Player UX/client capability presets, safe if DTO shape remains unchanged.

Defer:

- LL-HLS/CMAF.
- HEVC/AV1 executable HLS output.
- Durable remote transcode workers.
- Full Optimized Version asset lifecycle.

Minimum gate before implementation closeout:

- `cargo fmt --all`
- `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`
- Focused `cargo nextest run` per changed lane.
