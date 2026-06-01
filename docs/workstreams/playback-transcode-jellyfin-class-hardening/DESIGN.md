# Playback Transcode Jellyfin-Class Hardening - Design

Status: Closed
Last updated: 2026-06-01

## Problem

Nako already has a cleaner playback/transcode architecture than the typical
controller/helper-heavy media-server implementation: playback planning,
transcode pipeline planning, FFmpeg execution planning, HLS artifact manifests,
and runtime admission are modeled as typed Interfaces.

The next risk is parallel development pressure. Jellyfin/Plex-class behavior
adds device profiles, hardware capability matching, restart/reuse lifecycle,
artifact cleanup, command planning breadth, and operational diagnostics. If
multiple Codex terminals deepen those areas without an explicit seam freeze,
they will fight over the same files and may rebuild a monolithic media
encoding helper under a different name.

This workstream exists to freeze boundaries before implementation.

## Reference Posture

Jellyfin and Plex are product and architecture pressure, not source material.
Local Jellyfin code under `repo-ref/jellyfin` may be used to understand mature
behavior and failure modes, but Nako code, comments, schemas, tests, and docs
must remain original and aligned with Nako's domain model.

## Authority

- `CONTEXT.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0044-playback-capability-profile-planner.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/adr/0046-ffmpeg-probe-inventory.md`
- `docs/adr/0047-cpu-transcode-readiness.md`
- `docs/adr/0048-playback-transcode-startup-degradation.md`
- `docs/adr/0049-source-aware-transcode-runtime.md`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/adr/0053-runtime-control-plane-boundary.md`

## Target State

- Playback/transcode seam map is explicit enough that separate Codex terminals
  can work in parallel without editing the same ownership surface by accident.
- Public Interfaces between playback, transcode, server runtime, and artifact
  authority are named and guarded by stop conditions.
- The first safe parallel batch is split into independent goals:
  Playback Capability, Transcode Pipeline Capability, and FFmpeg Adapter.
- Coordinated HLS runtime/artifact work is sequenced after the first batch
  instead of being mixed into it.
- Artifact I/O pressure is kept as a later lane unless the planner explicitly
  expands scope.

## In Scope

- Interface freeze for playback/transcode hardening.
- Task ledger and validation gates for parallel Codex lanes.
- Worker prompt/worktree guidance for the first parallel batch.
- First implementation slices only after the freeze task records owned scopes,
  shared scopes, and stop conditions.

## Out Of Scope

- Copying, translating, or porting Jellyfin/Plex source.
- One giant playback/transcode rewrite.
- Built-in tunnel or remote-access provider behavior.
- Plugin/addon runtime behavior.
- LL-HLS, CMAF, DASH, DRM, remote workers, player UX, and hardware smoke
  matrices unless split into dedicated follow-on workstreams.

## Seam Map

### 1. Playback Capability Module

Owned by `crates/nako-playback`.

Responsibilities:

- Accept source facts, client facts, policy, and user preferences.
- Decide Direct Play, Remux, or HLS Transcode.
- Emit precise compatibility and transcode requirements.

It must not assemble FFmpeg command arguments, HLS artifact paths, runtime
session state, or server route behavior.

### 2. Transcode Pipeline Capability Module

Owned by `crates/nako-transcode` pipeline, hardware, probe, and policy code.

Responsibilities:

- Convert playback requirements plus hardware capability reports into a typed
  transcode pipeline plan.
- Explain stage-level fallback and degradation.
- Keep VAAPI, NVENC, QuickSync, CPU, tone-map, audio, and subtitle requirements
  explicit enough for tests and diagnostics.

It must not own playback policy or server session lifecycle.

### 3. FFmpeg Adapter Module

Owned by `crates/nako-transcode` execution and FFmpeg internals.

Responsibilities:

- Consume high-level execution requests and artifact manifests.
- Produce command plans and process execution policies.
- Keep low-level filter graph, encoder, muxer, seek, and output argument
  assembly behind internal adapters.

It must not leak raw FFmpeg builder details back into `nako-server` or
`nako-playback`.

### 4. HLS Artifact Authority Module

Owned by `crates/nako-transcode` artifact/staging policy code with coordinated
server integration.

Responsibilities:

- Own request variant identity, artifact names, manifest reconstruction, and
  serveable artifact allow-lists.
- Keep playlists, media groups, segments, init files, audio sidecars, and
  subtitle sidecars manifest-backed.

It must not mix runtime admission or process lifecycle policy into artifact
identity.

### 5. Playback Runtime Module

Owned by playback app services in `crates/nako-server`.

Responsibilities:

- Own HLS/remux session lifecycle, admission, reuse, supersede, cancel,
  timeout, failure classification, metrics, and diagnostics.
- Call transcode Interfaces instead of constructing FFmpeg details directly.

It must not become a second transcode planner.

### 6. Artifact I/O Policy Module

Owned by playback resource and artifact policy follow-on work.

Responsibilities:

- Model disk-sensitive HLS artifact pressure, cleanup, throttling, diagnostics,
  and startup recovery.

It should remain split from the first parallel batch unless the planner
explicitly reopens scope.

## Stop Conditions

Pause implementation and return to the planner if a task requires any of these:

- Changing public API DTO shape.
- Changing request identity or artifact path format.
- Adding schema migrations.
- Moving ownership across crate boundaries.
- Adding raw FFmpeg command assembly outside `nako-transcode`.
- Adding raw `tokio::spawn` for playback, transcode, artifact, or cleanup work
  without checking ADR 0053.
- Editing shared server playback files from two active lanes at once.

## First Parallel Batch

After `PTJCH-010` and `PTJCH-020`, these lanes are intended to run in parallel:

- Playback Capability: broaden compatibility conditions and test vocabulary
  inside `nako-playback`.
- Transcode Pipeline Capability: deepen stage-aware capability matching inside
  `nako-transcode`.
- FFmpeg Adapter: split internal command planning helpers without changing
  external behavior.

HLS Artifact Authority and Playback Runtime work should start after the first
batch clarifies Interfaces. Artifact I/O pressure should remain a later split.

## Closeout Criteria

- `PTJCH-010` and `PTJCH-020` are complete.
- Either the first implementation batch is complete or its tasks are split into
  separate active workstreams with clear ownership.
- Architecture links and handoff state are updated.
- Required gates pass or failures are recorded with exact follow-up scope.

## Closeout Summary

Closed on 2026-06-01 after the first parallel batch, HLS Artifact Authority,
and Playback Runtime slices were implemented and verified. `PTJCH-310` decided
that HLS artifact I/O pressure remains outside this coordination lane and
should use the existing `proposed:hls-artifact-io-pressure-enforcement`
follow-on. `PTJCH-390` closed the workstream with no additional Rust changes.
