# Playback Runtime Boundary Deepening

Status: Completed
Last updated: 2026-05-28

## Why This Lane Exists

The source-aware transcode runtime lane deepened Nako's playback pipeline, but
it also made `nako-server` playback orchestration more crowded. The current
`PlaybackAppService` module owns direct playback, remux, HLS session start,
HLS artifact serving, playback session validation, support evidence,
diagnostics, cancellation, and several artifact lifecycle helpers.

That shape is workable for the first source-aware slice, but it is a poor base
for adaptive HLS, fMP4, rsmpeg adapter feasibility, or remote transcode workers.
Those features should attach to focused boundaries rather than extending a
large orchestration module.

## Relevant Authority

- ADRs:
  - `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0049-source-aware-transcode-runtime.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/workstreams/source-aware-transcode-runtime/CLOSEOUT.md`
  - `docs/workstreams/playback-transcode-ops-hardening/HANDOFF.md`
- Code:
  - `crates/nako-server/src/app/playback/mod.rs`
  - `crates/nako-server/src/app/playback/hls.rs`
  - `crates/nako-server/src/app/playback/remux.rs`
  - `crates/nako-transcode/src/`

## Problem

The current playback runtime has several maintainability pressure points:

- `PlaybackAppService` is still the central place for route-oriented playback
  orchestration and artifact-serving mechanics.
- HLS playback-session playlist rewriting, segment readiness, throttled waits,
  and stale segment cleanup live next to unrelated session and support evidence
  code.
- Support evidence and runtime diagnostics are conceptually Admin/support
  read-model concerns, but they are still methods on the broad playback app
  service.
- The broad `PlaybackRuntimeStore` trait makes every playback sub-boundary look
  like it can use every repository operation.
- Tests prove behavior, but some tests stay tied to the broad module shape
  instead of the intended focused boundaries.

## Target State

When this lane closes:

- HLS artifact serving has a focused module/service that owns playlist rewrite,
  playable session state checks, segment readiness, throttled wait, segment
  cleanup, and direct segment response planning.
- `PlaybackAppService` delegates HLS artifact serving instead of carrying the
  low-level filesystem and lifecycle policy itself.
- Admin support evidence and runtime diagnostics have clearer read-model
  boundaries, or this lane records why they should remain on the app service.
- Store access is narrowed where it removes real coupling, without inventing a
  new abstraction for one call site.
- Existing route behavior and redaction contracts are unchanged and covered by
  focused gates.

## In Scope

- `crates/nako-server/src/app/playback`
- Focused playback HTTP tests that prove HLS/session/support behavior.
- Workstream docs and closeout evidence.

## Out Of Scope

- Public/Admin DTO shape changes.
- Database migrations.
- Adaptive HLS ladders, fMP4/CMAF, rsmpeg adapter replacement, or distributed
  transcode workers.
- New media capabilities beyond preserving existing behavior.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| HLS artifact serving can be split without route/API changes. | High | Existing HLS routes already call app methods with explicit inputs. | Keep the first slice smaller and only move helper functions. |
| Support evidence can be made more local without changing Admin DTOs. | Medium | DTO mapping already lives in `nako-api`; server app only collects context. | Leave support evidence as a documented follow-on if it needs broader Admin runtime reshaping. |
| Store trait narrowing should be evidence-driven. | Medium | `PlaybackRuntimeStore` is broad, but splitting traits too early can add noise. | Only split where a submodule gets a stable independent seam. |
| No compatibility constraints block behavior-preserving moves. | High | Nako has no users yet and the user explicitly allowed breaking internals. | Public/API behavior still stays stable because tests rely on it. |

## Architecture Direction

The immediate direction is to keep `PlaybackAppService` as the composition
entry point, but move cohesive sub-boundaries behind smaller modules:

```text
PlaybackAppService
  -> HlsAppService              execution/start
  -> HlsArtifactService         playlist/segment artifact serving
  -> RemuxAppService            remux execution/start
  -> PlaybackSupportReadModel   support evidence and diagnostics
```

The first slice should avoid a broad new trait. It can pass `PlaybackConfig`,
session records, and paths directly into a focused HLS artifact module. If that
module grows a real persistence seam, the store trait can be split later with
tests proving the reduced dependency direction.

## Closeout Condition

This lane can close when:

- HLS artifact serving no longer lives as helper functions on
  `playback/mod.rs`;
- behavior-preserving route and app tests pass;
- any support evidence or store trait follow-on is completed or explicitly
  split;
- docs reflect the shipped boundary;
- fresh closeout gates pass.

## Closeout Summary

Completed on 2026-05-28. The lane extracted HLS artifact serving and playback
support/runtime diagnostics collection into focused modules, preserved route and
redaction behavior, and recorded an explicit no-split decision for store traits
in this slice.
