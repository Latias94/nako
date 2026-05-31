# HLS Runtime Lifecycle Boundary

Status: Closed
Last updated: 2026-05-31

## Why This Lane Exists

Recent playback/transcode work made HLS output more capable: source-aware
planning, fMP4, progressive playlist readiness, seek/restart, selected audio,
audio output filters, and software-first HDR-to-SDR tone mapping are now
shipped. The next risk is not another FFmpeg flag. The risk is lifecycle
ownership.

Current HLS behavior is assembled across several modules:

- playback composition builds source context, request identity, runtime plan,
  staging layout, and resource demand;
- HLS app service reserves, reuses, supersedes, runs, cancels, and finalizes
  sessions;
- playlist and segment routes perform readiness checks and wait decisions;
- the FFmpeg runner owns process timeout/cancel and output publishing;
- startup and request paths both participate in cleanup;
- resource admission exists in server playback while transcode runtime still
  has a runner-level guard;
- `HlsArtifactIo` exists but is still not enforced.

That shape is acceptable for the shipped first slices, but it will become
harder to reason about when queueing, remote workers, LL-HLS/CMAF, disk I/O
pressure, and richer restart behavior arrive.

## Target State

When this workstream closes:

- HLS active/reuse/supersede/readiness/cancel/cleanup invariants are documented
  and covered by focused tests or explicit follow-on gaps;
- route and app code have a clearer lifecycle owner or coordinator boundary;
- resource admission and HLS request admission have a documented relationship;
- artifact readiness and cleanup policy has a single place to extend;
- artifact I/O pressure, remote workers, LL-HLS/CMAF, player UX, DTO changes,
  and storage/schema work are split unless explicitly approved.

## In Scope

- HLS runtime lifecycle invariants and test coverage mapping.
- Server playback HLS app-service boundaries.
- Existing session reuse, supersede, cancellation, readiness, segment wait, and
  cleanup behavior.
- Documentation and focused server HLS tests that prove existing invariants.
- A behavior-preserving lifecycle facade/coordinator only after `HRLB-010`
  freezes the boundary.

## Out Of Scope

- FFmpeg command planning and transcode pipeline selection.
- Transcode hardware capability inventory.
- Public/Admin DTO shape changes.
- Storage schema or durable artifact tables.
- Client/player seek UX or controls.
- LL-HLS/CMAF, DASH/CMAF, DRM/key delivery, or remote worker execution.
- Artifact I/O pressure enforcement until the lifecycle boundary is frozen.

## Architecture Direction

Start with `HRLB-010`, a docs/research and invariant freeze. It should produce
an explicit lifecycle table for:

- active same-generation requests;
- finished session reuse;
- different-generation supersede;
- playlist readiness while running;
- segment readiness and one-shot wait;
- cancellation and timeout cleanup;
- startup stale-session and terminal artifact cleanup;
- staging input release.

Only after those invariants are clear should implementation start. The first
implementation slice should be behavior-preserving: concentrate lifecycle
decisions and tests without changing FFmpeg command planning, transcode
pipeline policy, storage schema, or public contracts.

## HRLB-010 Lifecycle Freeze

This freeze is descriptive. It records current behavior and expected coverage
targets for the next behavior-preserving task. It does not approve a runtime
behavior change.

### Lifecycle Invariants

| Concern | Frozen behavior | Current evidence |
| --- | --- | --- |
| Active same-generation request | `hls_source` treats an active same request key as a duplicate conflict. Playlist entry points may wait on an already active same-generation session until the playlist is ready. | `hls_source_rejects_persisted_active_duplicate`; `hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`. |
| Finished session reuse | A finished HLS session may be reused only when the persisted request key matches, the output path matches the current layout primary playlist, and that playlist still exists. Reuse must not acquire a new process permit. | `hls_source_runs_runner_and_reuses_completed_session`; PRRS reuse evidence. |
| Different-generation supersede | A request key for the same source but a different HLS playback generation supersedes active prior-generation sessions by requesting cancellation, then starts a new session. | `hls_source_seek_generation_supersedes_active_prior_generation`; HSRL handoff. |
| Playlist readiness while running | Running HLS sessions may serve a playlist before FFmpeg exits only after the manifest primary playlist exists and contains at least one media or variant URI line. Header-only playlists are not ready. | `hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`; `hls_playlist_readiness_requires_a_media_uri_line`; HPRB handoff. |
| Segment readiness and one-shot wait | Segment routes serve only manifest-approved artifacts. For running sessions, a missing or zero-length artifact performs one configured throttle wait, then returns readiness conflict if still unavailable. Terminal missing artifacts are not found. | `hls_segment_waits_once_for_running_segment_when_throttle_enabled`; `hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`; `hls_source_rejects_persisted_active_duplicate`. |
| Cancellation and timeout cleanup | Cancellation signals the process-local token, marks active DB state as cancel requested from the route/supersede path, and lets the HLS runner kill FFmpeg and remove visible or temporary output. Timeout also kills FFmpeg, removes output, and maps to timeout failure. | Cancellation: `hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`; `hls_runner_can_publish_output_while_process_is_running`. Timeout cleanup is implemented in the HLS runner but lacks a focused HLS timeout test. |
| Startup stale-session cleanup | Startup marks unfinished active transcode sessions stale/failed before new playback work begins. This is transcode-session generic and should apply to HLS sessions. | `app_startup_marks_stale_transcode_sessions_failed`; DB transcode session stale contract. Add HLS-specific startup fixture in HRLB-020. |
| Terminal artifact cleanup | Startup artifact cleanup is optional, scans terminal finished/failed/cancelled sessions, deletes remux files or HLS output directories under the configured transcode root, honors retention, and skips paths outside the root. | `app_startup_cleans_expired_playback_artifacts_inside_transcode_root`. |
| Staging input release | FFmpeg source inputs release staging leases after successful HLS completion, after errors, and after playback-resource admission rejection. Local path hints have no lease. | `staging_lease_transitions_between_ready_and_leased`; `dropped_staging_lease_releases_manifest_record`; HLS end-to-end remote staging release remains a HRLB-020 test target. |
| Artifact I/O pressure | `HlsArtifactIo` is modeled as not-yet-enforced. HLS lifecycle work must not silently add disk read/write pressure limits. | `playback_resource_admission_explains_process_permits_and_unenforced_artifacts`; PRRS closeout. Split PAIP follow-on. |

### Cleanup Ownership Map

| Surface | Current owner | Freeze |
| --- | --- | --- |
| Request admission and in-flight same-key guard | `HlsAppService` | Owns same-key duplicate rejection, finished reuse, new-session creation, supersede cancellation requests, and release of the process-local in-flight key. |
| Playlist/segment artifact serving | `HlsArtifactService` | Owns manifest reconstruction, playlist readiness, segment readiness, one-shot running wait, content type/range planning, and per-request segment cleanup. |
| FFmpeg process cancellation/timeout/failure cleanup | `FfmpegHlsRunner` | Owns process kill, temp or visible HLS output cleanup, and conversion to finished/cancelled/error outcomes. |
| Startup recovery and artifact cleanup | `ServerStartupWorkflow` | Owns stale active-session failure and optional terminal playback artifact cleanup under the configured transcode root. |
| FFmpeg input staging lease | `FfmpegInputService` plus staging runtime | Owns staged input acquisition and release. HLS orchestration must release inputs after every success/error/admission branch. |
| Resource pressure | `PlaybackRuntimeAdmission` | Owns CPU/GPU process permits today. HLS artifact I/O remains observable/not-yet-enforced until a PAIP follow-on. |

### Test Coverage Map

| Area | Current coverage status | HRLB-020 target |
| --- | --- | --- |
| Same-generation active conflict | Covered for `hls_source`; playlist wait covered indirectly. | Add a focused route/app invariant test if lifecycle extraction changes entry points. |
| Finished reuse | Covered across same process and app restart. | Preserve as a regression test around any coordinator/facade. |
| Generation supersede | Covered for active default generation superseded by non-zero seek generation. | Add planned/starting/cancel-requested state variants only if coordinator extraction touches them. |
| Running playlist readiness | Covered by app test plus unit guard requiring a URI line. | Preserve. |
| Running segment one-shot wait | Unit covered; app test covers eventual readiness and missing running conflict. | Add zero-length segment case if artifact service is refactored. |
| Cancellation cleanup | Covered by app cancellation and HLS runner visible-output cancellation cleanup. | Preserve; assert DB transition after any coordinator refactor. |
| Timeout cleanup | HLS runner has timeout branch; remux has focused timeout test; HLS-specific timeout cleanup is not directly tested. | Add focused HLS timeout cleanup test before accepting HRLB-020. |
| Startup stale HLS cleanup | Generic transcode startup stale recovery is covered; HLS-specific fixture is missing. | Add an HLS session fixture so stale recovery and request reuse cannot drift apart. |
| Terminal HLS artifact cleanup | Covered for failed HLS output directory under root and outside-root skip. | Preserve. |
| Staging input release | Lease primitive covered; HLS remote staged input release is not directly covered. | Add HLS success/error/admission rejection staged-input release tests if remote staging path is touched. |

## Follow-On Pressure

The storage/VFS subarchitecture review identified playback artifact I/O
pressure as the most concrete storage follow-on. This workstream should decide
whether that becomes a later `HRLB` task or a separate
`storage-vfs-playback-artifact-io-pressure` workstream. Do not implement it in
`HRLB-010`.

`HRLB-010` freezes the decision as: split artifact I/O pressure into a PAIP
follow-on, preferably under the existing
`proposed:hls-artifact-io-pressure-enforcement` lane name. PAIP should not be
implemented as part of the behavior-preserving lifecycle coordinator because it
crosses playback resource admission, storage/VFS health, segment read/write
pressure, and Admin diagnostics. `HRLB-020` may keep the current
`HlsArtifactIo` not-yet-enforced evidence, but must not enforce it.

## HRLB-030 Follow-On Split Decisions

`HRLB-030` keeps this workstream as a lifecycle/test-coverage lane and splits
all non-lifecycle expansion work into explicit follow-ons. The decisions are:

| Area | Decision | Proposed lane | Rationale |
| --- | --- | --- | --- |
| HLS test stability | Open the next bounded playback-transcode workstream. | `proposed:hls-progressive-readiness-test-stability` | HRLB-020 passed the final HLS gate, but an earlier full-suite run exposed a load-sensitive progressive-readiness timeout. Gate trust should be hardened before larger HLS runtime changes. |
| Artifact I/O pressure | Split into a PAIP follow-on, not this lifecycle lane. | `proposed:hls-artifact-io-pressure-enforcement` | Enforcing `HlsArtifactIo` crosses playback resource demand, segment read/write scheduling, storage/VFS health, and Admin diagnostics. |
| Resource admission unification | Keep as a separate playback resource scheduler follow-on. | `proposed:playback-admission-queueing-and-waitlist` | Queueing, waitlists, reuse fairness, and permit ownership are broader than HLS lifecycle invariants and should not be hidden inside artifact I/O enforcement. |
| Remote workers | Keep as a later runtime/control-plane follow-on. | `proposed:remote-transcode-worker-runtime` | Remote execution needs durable ownership, artifact transport, cancellation semantics, and scheduler policy beyond server-local HLS lifecycle. |
| LL-HLS/CMAF | Keep as a later protocol/runtime follow-on. | `proposed:ll-hls-cmaf-runtime` | LL-HLS changes manifest semantics, partial segment readiness, and player compatibility; it should build on stable lifecycle and test gates. |
| Player UX | Keep in client/player product lanes. | `proposed:player-hls-session-controls-and-recovery` | Seek controls, stalled playback recovery, ABR UI, and device behavior depend on client contracts and should not be implemented inside server lifecycle cleanup. |

The immediate recommendation is: close HRLB after `HRLB-040`, then open
`hls-progressive-readiness-test-stability` before PAIP or LL-HLS/CMAF. PAIP can
be planned after the HLS gate is stable, with storage/VFS coordination from the
start.

## HRLB-040 Closeout Review

`HRLB-040` did not close this workstream. Fresh verification on 2026-05-31
showed the required full HLS gate failing twice:

```text
cargo nextest run -p nako-server hls --no-fail-fast
```

Both failures were in the progressive readiness tests:

- `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`
- `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`

Both tests passed when rerun individually, so the remaining work is a
progressive readiness test-stability boundary, not PAIP, LL-HLS/CMAF, remote
workers, player UX, DTO, schema, or VFS behavior. That follow-on is now split
to `docs/workstreams/hls-progressive-readiness-test-stability/`.

Final closeout result: `hls-progressive-readiness-test-stability` stabilized
the full HLS gate with a test-only Windows readiness timeout adjustment.
`HRLB-040` was retried after HPRTS closeout, the default full HLS gate passed,
and this workstream is closed.

The remaining follow-ons are outside HRLB: PAIP artifact I/O pressure,
resource admission queueing, remote workers, LL-HLS/CMAF, and player UX.
