# HDR Tone Mapping Pipeline

Status: Draft
Last updated: 2026-05-30

## Why This Lane Exists

HDR playback is a media-server maturity requirement, but it is easy to implement
incorrectly by bolting an FFmpeg filter string onto HLS. Nako needs a first
principles plan for color compatibility inputs, client HDR capability, software
and hardware tone-mapping strategies, fallback behavior, diagnostics, and HLS
runtime interaction before code starts.

## Relevant Authority

- ADRs:
  - `docs/adr/0044-playback-capability-profile-planner.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0046-ffmpeg-probe-inventory.md`
  - `docs/adr/0048-playback-transcode-startup-degradation.md`
  - `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
  - `docs/architecture/LANES.md`
- Related workstreams:
  - `docs/workstreams/playback-media-maturity-first-slices/`
  - `docs/workstreams/source-aware-transcode-runtime/`
  - `docs/workstreams/playback-planner-transcode-seam-deepening/`

## Problem

Nako has HDR as a known playback capability gap, but the first executable slice
is not yet safe. The codebase needs to know which probe facts are authoritative,
how client HDR capability is represented, where tone-mapping policy lives, how
hardware and software strategies are selected, and how failures degrade.

## Target State

After the research task, this workstream should either:

- become active with a sequenced implementation plan and first code task; or
- remain draft with explicit blockers and required ADR/research follow-ups.

The eventual implementation target is:

- **Color Pipeline Requirement** is modeled in playback planning;
- **HDR Tone Mapping** is selected only when source/client compatibility
  requires it;
- FFmpeg software and hardware tone-mapping strategies are deterministic and
  testable;
- fallback behavior preserves playable output and clear diagnostics.

## In Scope For HTP-010

- inspect existing probe, playback, transcode, and hardware planner boundaries;
- confirm current HDR/color fact availability;
- decide the smallest implementation slice and required tests;
- update this workstream and playback architecture notes.

## Out Of Scope For HTP-010

- code changes in `crates/nako-playback`, `crates/nako-transcode`, or
  `crates/nako-server`;
- FFmpeg filter implementation;
- public API DTO changes;
- web/mobile HDR controls;
- device profile databases.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| HDR implementation overlaps audio compatibility scopes. | High | Both touch playback and transcode planner seams. | Implementation must be serialized or moved behind a narrower branch plan. |
| Probe/color facts may need validation before policy work. | Medium | Existing media facts are source-aware but HDR-specific completeness is not confirmed here. | First code task may need to be media-probe facts rather than playback policy. |
| Hardware tone mapping must be optional with CPU fallback. | High | ADR 0048 already requires startup degradation and fallback thinking. | A hardware-only slice would be brittle for self-hosted deployments. |

## Architecture Direction

Start with docs/research. The planner should not approve implementation until
`HTP-010` identifies the first executable seam, validation commands, and shared
scope with active audio work. If the first code slice is playback-only, it can
follow the same shape as audio compatibility. If it requires media probe or
FFmpeg inventory changes first, split that explicitly.

## Closeout Condition

This lane can close only after implementation evidence exists or after the
research task intentionally defers it with documented blockers. Draft status is
expected until `HTP-010` resolves the first executable slice.
