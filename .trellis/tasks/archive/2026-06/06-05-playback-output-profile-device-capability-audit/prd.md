# Playback output profile and device capability audit

## Goal

Decide the next stable playback output and device capability contract before
implementing more decoding/transcode features such as HEVC/AV1 output,
hardware tone mapping, image subtitle burn-in, TV profiles, mobile/native
profiles, or richer renderer behavior.

## Requirements

- Audit current `nako-playback`, `nako-transcode`, server playback flow, and
  Public/Admin DTO boundaries.
- Decide which facts belong to Client Applications, output profiles, Admin
  diagnostics, and operator policy.
- Identify the first safe executable follow-on after the audit.
- Keep this task architecture-only unless explicitly converted later.
- Preserve the separation between Playback Source Selection, Playback Runtime,
  Transcode Pipeline planning, and FFmpeg command execution.

## Acceptance Criteria

- [x] The audit defines the minimum output/device capability facts needed for
      the next playback/transcode wave.
- [x] HEVC/AV1, HDR tone mapping, subtitle burn-in, and device profile
      follow-ons are ranked with prerequisites.
- [x] Public Client API versus Admin diagnostics responsibilities are
      documented.
- [x] Unsafe parallel combinations are listed.
- [x] The audit recommends one first executable playback/transcode task.

## Definition of Done

- Research output is written under this task or linked from the parent audit.
- Any architecture-map update is explicit and limited.
- No Rust/TypeScript implementation changes unless the task is later expanded.
- `git diff --check` passes.

## Out of Scope

- No HEVC/AV1 execution.
- No hardware tone-map execution.
- No subtitle burn-in implementation.
- No Public Client or Admin DTO changes in this audit task.

## Technical Notes

- Parent audit: `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/`
- Key research:
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/playback-decoding-transcode.md`
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/synthesis.md`
  - `.trellis/tasks/06-05-playback-output-profile-device-capability-audit/research/public-client-admin-capability-contract.md`
  - `.trellis/tasks/06-05-playback-output-profile-device-capability-audit/research/playback-transcode-profile-boundary.md`
- Important docs/ADRs:
  - `docs/architecture/PLAYBACK.md`
  - ADR 0038, 0044, 0045, 0049, 0052, 0053

## Audit Outcome

The current playback/transcode boundaries are healthy enough for narrow
follow-ons, but the next wave must not start with HEVC/AV1 execution, hardware
tone mapping, or image subtitle burn-in. The first missing contract is an
output/device profile layer that separates:

- Public Client device/player capability facts;
- `nako-playback` pure planner profile and requirement facts;
- `nako-transcode` runtime capability, pipeline, profile identity, artifact,
  and FFmpeg command facts;
- `nako-server` operator policy, resource admission, runtime orchestration, and
  Admin diagnostics;
- Admin-only redaction-safe support evidence.

Minimum device/output capability facts for the next wave:

- `profile_id`, `profile_version`, `device_family`, and optional
  `player_engine`;
- direct-play profile rows with container, video codec, audio codec, subtitle
  format, and condition limits;
- remux output profile rows;
- transcode output profile rows including HLS segment/variant/output codec
  facts;
- subtitle delivery profile rows for sidecar, embedded, text-track, and burn-in
  requirements;
- audio output facts for channel limits, downmix, normalization, and future
  passthrough/DRC behavior;
- color pipeline facts for HDR format, bit depth, tone-map target, and
  deferred unsupported formats;
- a legacy/default mapping from the current flat fields so existing behavior
  stays stable.

Recommended first executable task:

1. `public-client-playback-capability-contract-parity-gate`
   - No playback behavior change.
   - Align the current capability fields across `nako-client-protocol`,
     OpenAPI, Rust client, `nako-client-core`, SDK query surfaces, and
     `docs/api/HTTP_API.md`.
   - Add a parity gate so future profile work does not drift across generated
     and hand-written clients.

First profile task after that gate:

2. `playback-output-profile-v2-skeleton-contract-only`
   - Add additive optional profile/device-family skeleton fields.
   - Map current flat capability fields into a `legacy_default` row.
   - Prove absent v2 fields do not change planner decisions or identity.
   - Do not enable HEVC/AV1, hardware tone mapping, or image subtitle burn-in.

Ranked execution follow-ons after the contract layer:

1. Browser/mobile/TV/renderer fixture profiles and planner matrix tests.
2. HEVC/AV1 HLS output policy design and one narrow first executable path.
3. Hardware tone-map execution first slice.
4. Image subtitle burn-in capability and execution slice.
5. Admin effective profile support evidence.

Unsafe parallel combinations:

- HEVC/AV1 output execution with hardware tone-map execution.
- Multiple tasks editing `ClientPlaybackCapabilitiesDto`,
  `BrowserPlaybackCapabilitiesDto`, playback capability query mapping, or
  generated Public Client contracts.
- Multiple tasks editing `PlaybackTargetProfile::identity`,
  `TranscodeProfile` identity, HLS request variant identity, or artifact
  manifest reconstruction.
- Admin support evidence expansion with Public Client profile DTO expansion
  without one contract owner.
- HLS lifecycle/admission changes with VFS/remote staging or circuit-breaker
  changes in the same playback flow files.
