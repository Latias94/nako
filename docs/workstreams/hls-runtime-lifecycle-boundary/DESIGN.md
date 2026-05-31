# HLS Runtime Lifecycle Boundary

Status: Active
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

## Follow-On Pressure

The storage/VFS subarchitecture review identified playback artifact I/O
pressure as the most concrete storage follow-on. This workstream should decide
whether that becomes a later `HRLB` task or a separate
`storage-vfs-playback-artifact-io-pressure` workstream. Do not implement it in
`HRLB-010`.
