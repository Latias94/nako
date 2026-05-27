# Source-Aware Transcode Runtime

Status: Completed
Last updated: 2026-05-28

## Why This Lane Exists

Nako now has playback planning, HLS/remux execution, FFmpeg probe inventory,
stage-aware hardware planning, CPU fallback readiness, and startup degradation.
Those lanes made the surface safe and explainable, but the executable HLS path
is still narrow: it assumes a fixed H.264/AAC output and has limited knowledge
of the source stream's codec profile, bit depth, HDR color metadata, subtitle
shape, audio shape, frame rate, and required filter graph.

Jellyfin-class playback requires source-aware decisions. Nako should not grow
that by adding one more branch to `ffmpeg.rs`; it needs a deeper path from
source media facts to playback requirement, pipeline plan, FFmpeg command, and
runtime job supervision.

## Relevant Authority

- ADRs:
  - `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0046-ffmpeg-probe-inventory.md`
  - `docs/adr/0047-cpu-transcode-readiness.md`
  - `docs/adr/0048-playback-transcode-startup-degradation.md`
  - `docs/adr/0049-source-aware-transcode-runtime.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/workstreams/playback-transcode-policy-deepening/HANDOFF.md`
  - `docs/workstreams/ffmpeg-hardware-pipeline-planner/HANDOFF.md`
  - `docs/workstreams/ffmpeg-probe-inventory/HANDOFF.md`
- Reference material:
  - `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Encoder/EncoderValidator.cs`
  - `repo-ref/jellyfin/MediaBrowser.Controller/MediaEncoding/EncodingHelper.cs`
  - `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Transcoding/TranscodeManager.cs`
  - `repo-ref/jellyfin/MediaBrowser.Model/Configuration/EncodingOptions.cs`
  - `repo-ref/ffmpeg/doc/ffmpeg.texi`
  - `repo-ref/ffmpeg/doc/filters.texi`
  - `repo-ref/ffmpeg/doc/muxers.texi`
  - `repo-ref/rsmpeg`

## Problem

The current implementation has several mature seams, but it still lacks the
facts and runtime behavior needed for robust self-hosted media playback:

- `MediaProbeResult` does not expose codec profile, level, pixel format, bit
  depth, frame rate, rotation, disposition, HDR color metadata, subtitle codec
  details, or attachment/font facts.
- `PlaybackPlanner` explains direct/remux/transcode at a coarse level, but it
  does not yet produce a structured source-aware `TranscodeRequirement`.
- `TranscodePipelinePlanner` can choose decode/filter/encode stages, but the
  request does not yet carry source codec facts that determine whether a
  hardware decoder, tone-map filter, subtitle burn-in filter, or fallback is
  valid.
- `FfmpegCommandBuilder::hls` still constructs a single HLS VOD command in one
  place and only supports the narrow subtitle strategy of omitting subtitles.
- HLS execution produces artifacts before serving the playlist instead of
  supervising a progressive live transcode job with progress, cancellation,
  throttling, and segment cleanup.
- Admin diagnostics report static readiness better than before, but they cannot
  yet explain per-source transcode decisions and runtime progress.

## Target State

When this lane closes, Nako should have:

- richer source media technical facts owned by `nako-core` and populated by
  `nako-media-probe`;
- playback planning that emits structured transcode requirements with selected
  streams, output constraints, subtitle strategy, HDR/tone-map intent, and
  compatibility reasons;
- source-aware pipeline planning in `nako-transcode` that uses media facts plus
  FFmpeg inventory to select decode/filter/encode stages and fallback reasons;
- FFmpeg command construction split into small builders for device/input,
  filter graph, encoder, audio, subtitle, and HLS muxer concerns;
- runtime job supervision that can parse progress, expose session metrics,
  cancel safely, and prepare for progressive HLS serving;
- redaction-safe Admin diagnostics for both capability gaps and per-session
  runtime status.

## In Scope

- `nako-core` media probe and transcode session records.
- `nako-media-probe` ffprobe JSON parsing and fixtures.
- `nako-playback` playback compatibility and transcode requirement records.
- `nako-transcode` pipeline request/plan, FFmpeg command builders, and HLS
  profile identity.
- `nako-server` playback app service orchestration, progress persistence, and
  Admin diagnostics mapping.
- Database migrations needed for durable media facts or transcode progress.
- Tests and docs for each vertical slice.

## Out Of Scope

- Full adaptive bitrate ladder productization.
- Optimized Version background artifact workflow.
- Remote transcode workers.
- rsmpeg execution adapter replacement.
- Web, desktop, or mobile player UI.
- DLNA, SyncPlay, live TV, offline sync, and network punching.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| FFmpeg CLI remains the first executable transcode adapter. | High | Existing runners and ADR 0038/0045. | Add a separate adapter abstraction task before rsmpeg work. |
| Source-aware facts should start in `MediaProbeResult` rather than a new crate. | High | Existing `nako-core` probe records are already shared by playback and server. | Split a dedicated media-technical-facts module if the record grows too large. |
| HLS H.264/AAC can remain the first executable output while the planner deepens. | High | Existing clients and tests target HLS H.264/AAC. | Add output codec negotiation before command builder split. |
| Schema migrations are acceptable when facts or progress need persistence. | Medium | Existing workstreams update storage when runtime state becomes durable. | Keep new facts in JSON first if schema churn blocks progress. |
| Jellyfin should shape requirements but not module design. | High | Repository reference-code rule and prior ADRs. | Re-review generated diffs for copied implementation details. |
| rsmpeg is useful for future adapter ergonomics, not the first runtime target. | Medium | Current server runs FFmpeg CLI and already has redaction-safe command plans. | Open a separate rsmpeg adapter feasibility lane if CLI limits block progress. |

## Architecture Direction

This lane should deepen existing modules rather than add a new top-level media
engine crate too early.

The desired flow is:

```text
MediaProbeResult
  -> PlaybackPlanner
  -> TranscodeRequirement
  -> TranscodePipelinePlanner
  -> FfmpegCommandPlan
  -> TranscodeJobRuntime
  -> Admin diagnostics / Public playback transport
```

Ownership:

- `nako-core` owns stable source facts and durable session/progress records.
- `nako-media-probe` owns ffprobe execution and mapping into source facts.
- `nako-playback` owns compatibility decisions and transcode requirements.
- `nako-transcode` owns source-aware pipeline planning and FFmpeg command
  generation.
- `nako-server` owns runtime orchestration, persistence, cancellation, HLS
  serving, and diagnostics adaptation.
- `nako-api` exposes only stable wire DTOs and redaction-safe evidence.

The implementation should avoid a single Jellyfin-style encoding helper. Each
FFmpeg concern should be small enough to test with command-plan golden tests:
device initialization, input decode, filter graph, video encoder, audio encoder,
subtitle handling, and HLS muxer output.

## Closeout Condition

This lane can close when:

- source-aware probe facts are represented and tested;
- playback transcode requirements are explicit and covered by planner tests;
- pipeline planning consumes source facts for at least codec/profile/bit-depth
  and one HDR or subtitle decision;
- FFmpeg HLS command planning is split enough to support device/filter/muxer
  growth without a monolithic helper;
- runtime progress or progressive HLS supervision has a verified first slice;
- Admin diagnostics expose the new evidence without leaking raw paths or
  command strings;
- focused and closeout gates pass, or remaining work is split into follow-ons.

## Closeout Summary

Completed on 2026-05-28. The lane delivered the first source-aware vertical
slice from probe facts through playback requirements, transcode pipeline
planning, staged FFmpeg HLS command construction, runtime progress metrics,
Admin/Public evidence mapping, and progressive HLS segment serving.

Adaptive ladders, fMP4 output, remote transcode workers, and an rsmpeg adapter
remain explicit follow-on lanes rather than unfinished work in this lane.
