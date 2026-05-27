# FFmpeg Hardware Pipeline Planner

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

The Playback Capability Planner can now explain why playback should direct
play, remux, or transcode. The next weak point is lower: `nako-transcode` still
turns hardware policy into a small selected accelerator and lets FFmpeg command
planning branch on that selection. That is not enough for mature media-server
features such as hardware decode, hardware filters, tone mapping, subtitle
burn-in, stage-specific fallback, and future platform adapters.

## Relevant Authority

- ADRs:
  - `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/workstreams/playback-capability-profile-planner/FOLLOW_ONS.md`
  - `docs/workstreams/playback-transcode-policy-deepening/HANDOFF.md`
- Reference material:
  - `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Encoder/EncoderValidator.cs`
  - `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Encoder/MediaEncoder.cs`
  - `repo-ref/jellyfin/MediaBrowser.Controller/MediaEncoding/EncodingHelper.cs`
  - `repo-ref/jellyfin/MediaBrowser.Model/Configuration/EncodingOptions.cs`

## Problem

At lane start, the transcode runtime had useful pieces, but the main seam was
still too shallow:

- `HardwareAccelerationReport` is centered on encoder discovery.
- `select_hardware_acceleration` returns one selected accelerator for the whole
  HLS path.
- `TranscodeAccelerationPlan::from_hardware_selection` hard-codes stage choices
  from that single accelerator.
- `ffmpeg.rs` translates `acceleration.encode.accelerator` directly into FFmpeg
  arguments.
- Admin diagnostics expose capability summaries, but cannot explain whether
  decode, filter, hwaccel, or encode caused a failure.

Deleting that old chain should move the complexity into one deep Module rather
than spreading hardware decisions through playback app services and command
builders.

## Target State

Nako has a **Transcode Pipeline Planner** in `nako-transcode` that:

- accepts output requirements, track/subtitle/output constraints, global
  **Hardware Acceleration Policy**, and a cached **Hardware Capability Report**;
- returns a typed `TranscodePipelinePlan` with decode/filter/encode stages;
- records fallback evidence and unsupported reasons;
- drives transcode profile identity and FFmpeg command planning;
- supports the current HLS H.264/AAC path while preparing for NVENC/NVDEC, QSV,
  VAAPI, AMF, and VideoToolbox;
- keeps Public Client API free of hardware/runtime details;
- gives Admin diagnostics redaction-safe stage evidence.

## In Scope

- New pipeline planning records in `nako-transcode`.
- Stage-aware hardware capability records.
- Replacement of encoder-only selection helpers where they are shallow.
- HLS FFmpeg command planning from pipeline/execution policy.
- Admin diagnostic mapping for stage-aware evidence.
- Focused tests for fallback, fail policy, command generation, and redaction.
- Removal of now-redundant old helpers or compatibility wrappers.

## Out Of Scope

- Full HDR tone mapping implementation.
- Subtitle extraction/conversion/burn-in breadth beyond existing validation.
- Adaptive HLS ladders.
- Optimized Version artifacts.
- Remote transcode workers.
- Frontend playback UI work.
- Copying Jellyfin internals or schemas.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| HLS H.264/AAC remains the only executable output in this lane. | High | Existing `FfmpegCommandBuilder::hls` and Public Client transcode plan. | Broaden output validation and command tests before closeout. |
| `nako-transcode` is the right crate for pipeline planning. | High | ADR 0038/0044 ownership split; `nako-transcode` owns FFmpeg and hardware inventory. | Move only pure records to `nako-core` if multiple crates need them without transcode. |
| Admin diagnostics can change because Nako is not production-compatible yet. | High | User explicitly approved fearless refactor and deleting old semantics. | Add adapters only if generated SDK/client compile failures require a transition. |
| Jellyfin is reference pressure, not source material. | High | Repo AGENTS reference-code rule. | Re-review diffs for copied strings or schema shapes. |

## Architecture Direction

The deep Module is `TranscodePipelinePlanner`. Its Interface should stay small:
callers provide a `TranscodePipelineRequest` and receive a
`TranscodePipelinePlan`. The Implementation may know about hardware stage
capabilities, fallback rules, and FFmpeg adapter constraints.

`nako-server::app::playback::HlsAppService` should become an Adapter that:

- loads config and cached runtime capability report;
- asks the planner for a pipeline;
- stores the resulting execution policy/profile identity;
- passes the policy to the HLS runner.

`FfmpegCommandBuilder` should remain an Adapter over an already-planned
pipeline. It may know exact FFmpeg arguments, but should not decide whether a
hardware path is allowed.

## Closeout Condition

This lane can close when:

- the old encoder-only hardware selection chain is removed or reduced to test
  fixtures;
- HLS command planning consumes a stage-aware pipeline plan;
- Admin diagnostics expose stage-aware, redaction-safe hardware evidence;
- focused playback/transcode/API gates pass;
- docs and handoff reflect shipped behavior and follow-ons.

## Shipped Outcome

- `nako-transcode` now exports stage-aware hardware capability facts and a
  `TranscodePipelinePlanner`.
- HLS command planning consumes a pipeline-derived execution policy.
- `nako-server` adapts config/runtime capability state into a pipeline plan
  instead of carrying encoder-only selection state.
- Admin playback diagnostics expose pipeline readiness and stage capability
  evidence while the Public Client API remains hardware-redacted.
- The old production selected-accelerator helper chain was deleted.
