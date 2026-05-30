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

## HTP-010 Research Result

Current source facts are sufficient for a first HDR planning slice. Existing
`MediaStreamTechnicalFacts` records include pixel format, bit depth, color
range, color space, transfer, primaries, chroma location, and HDR metadata for
dynamic range, mastering display, content light level, Dolby Vision, and
HDR10+. `nako-media-probe` already maps PQ and HLG transfer functions into HDR
detection, and persistence tests cover HDR technical JSON round trips.

Current client facts are intentionally shallow but enough for the first slice:
`ClientPlaybackCapabilities.supports_hdr` can distinguish an HDR-capable
client from an SDR-only path, and playback already emits
`VideoHdrUnsupported` when direct play would send HDR to a client that does not
support HDR. `PlaybackOutputConstraints.prefer_hdr` and the transcode output
constraint mirror already carry a coarse HDR preference.

The missing boundary is not more ffprobe data first. The missing boundary is a
playback-owned **Color Pipeline Requirement** that turns source color facts and
client capability into an explicit output intent:

- preserve source color when the client can present it;
- tone-map HDR to SDR when the client cannot present the source HDR path;
- reject or defer formats outside the first slice, such as Dolby Vision-only
  handling, HDR10+ dynamic metadata preservation, or hardware-specific paths.

## First Executable Slice

The first implementation task after planner activation is `HTP-020`, a
playback-only vocabulary slice. It should add the **Color Pipeline Requirement**
and typed compatibility reasons in `nako-playback` without changing FFmpeg
commands, server HLS behavior, Public Client API DTOs, or media probe records.

The first media-output slice after that is software-first HLS tone mapping for:

- a selected video stream with existing HDR facts such as HDR10/PQ or HLG;
- an SDR-only client path represented by `supports_hdr=false`;
- output HLS H.264/AAC using the current session-started FFmpeg CLI boundary;
- deterministic CPU fallback before any vendor hardware tone-map strategy.

Hardware tone mapping for VAAPI, QSV, NVENC, AMF, VideoToolbox, OpenCL, or
device-specific filter chains is a follow-on. Device profile databases,
per-display HDR modes, Dolby Vision dynamic metadata, HDR10+ preservation, UI
controls, and operator hardware smoke matrices are also follow-ons.

## Shared Scopes And Sequencing

`HTP-020` overlaps the active `ACDN-020` playback vocabulary task in:

- `crates/nako-playback/src/capability.rs`;
- `crates/nako-playback/src/values.rs`;
- `crates/nako-playback/src/lib.rs`.

Do not start HDR implementation while `ACDN-020` is active. After `ACDN-020`
lands, the HDR worker must reread those files and preserve audio requirement
semantics instead of building a parallel requirement model.

Later HDR tasks share scopes with transcode and server playback:

- `crates/nako-transcode/src/policy.rs`;
- `crates/nako-transcode/src/pipeline.rs`;
- `crates/nako-transcode/src/profile.rs`;
- `crates/nako-transcode/src/ffmpeg.rs`;
- `crates/nako-server/src/app/playback/mod.rs`;
- `crates/nako-server/src/app/playback/hls.rs`.

Public API DTOs, generated clients, and media probe schema changes are not
approved for the first slice. If implementation proves those are required,
return to planner coordination before editing code.

## Architecture Direction

Start with docs/research. The planner should not approve implementation until
`HTP-010` identifies the first executable seam, validation commands, and shared
scope with active audio work. `HTP-010` confirms a playback-first seam. If the
first code task discovers missing probe facts or needs Public Client API shape
changes, split that work explicitly rather than widening the HDR task.

The workstream should remain draft until planner review confirms that
`ACDN-020` is complete, merged, or otherwise serialized away from the playback
vocabulary files. Moving the lane to active before that would create a shared
scope conflict.

## Closeout Condition

This lane can close only after implementation evidence exists or after the
research task intentionally defers it with documented blockers. Draft status is
expected until `HTP-010` resolves the first executable slice.
