# Media Server Architecture Progress Map - Design

Status: Closed
Last updated: 2026-05-29

## Why This Lane Exists

Nako has many ADRs and detailed workstreams, but the project lacks one current
architecture map that answers:

- what Nako is trying to become;
- which systems already exist;
- where recent HLS/transcode work sits;
- which decisions are durable enough to guide future Jellyfin/Plex-class
  development.

## Relevant Authority

- ADRs:
  - `docs/adr/0001-modular-monolith-rust-workspace.md`
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0049-source-aware-transcode-runtime.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/ROADMAP.md`
  - `docs/GOALS.md`
  - `docs/workstreams/README.md`
- Related workstreams:
  - `docs/workstreams/hls-audio-sidecar-artifacts/`
  - `docs/workstreams/hls-media-renditions-runtime/`
  - `docs/workstreams/playback-capability-profile-and-rendition-planning/`

## Problem

The detailed workstream trail is too granular for planning the next system
slice. Contributors can see specific lanes, but not the current media-server
shape or the next architectural pressure points.

## Target State

- `docs/ARCHITECTURE.md` gives a current system map and progress matrix.
- `docs/ROADMAP.md` reflects the actual playback/transcode state after recent
  HLS work.
- ADR 0052 records the FFmpeg CLI-first HLS runtime and media engine boundary.
- `docs/README.md`, `docs/adr/README.md`, and `docs/workstreams/README.md`
  point to the new authority.

## In Scope

- Architecture documentation only.
- Roadmap status corrections for playback/transcode.
- ADR for media engine/HLS runtime boundary.
- Workstream evidence for this documentation update.

## Out Of Scope

- Runtime code changes.
- New HLS seek behavior.
- DASH, LL-HLS, tone mapping, ASS burn-in, or remote worker implementation.
- Provider or addon feature breadth.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Nako should remain FFmpeg CLI-first for core media processing. | High | Existing transcode runtime and FFmpeg planning lanes. | ADR 0052 would need to become proposed or superseded by an engine evaluation lane. |
| The main documentation gap is navigability, not missing low-level design. | High | Existing ADR and workstream volume. | More ADRs would be needed instead of a top-level architecture map. |
| Playback/transcode roadmap status is stale. | High | Recent HLS workstreams closed after the roadmap's local MVP summary. | Future planning would underestimate current capability. |

## Architecture Direction

Use a layered documentation model:

- `CONTEXT.md` remains the glossary.
- `docs/ARCHITECTURE.md` owns the current system map.
- ADRs own durable decisions.
- Workstreams own execution and evidence.
- `docs/ROADMAP.md` owns phase status and future breadth.

## Closeout Condition

This lane can close when the architecture map, ADR, roadmap update, and index
links are in place and docs gates pass.
