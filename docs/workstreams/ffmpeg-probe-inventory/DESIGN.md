# FFmpeg Probe Inventory

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

The FFmpeg Hardware Pipeline Planner gave Nako a deeper execution seam, but the
probe side still has a shallow source of truth: encoder discovery. Mature media
servers need the runtime to know whether FFmpeg can decode, hwaccel, filter,
encode, and apply bitstream filters as separate facts.

This lane turns FFmpeg discovery into a structured inventory that can feed
pipeline planning, Admin diagnostics, and later HDR/subtitle/adaptive HLS work.

## Relevant Authority

- ADRs:
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0046-ffmpeg-probe-inventory.md`
- Existing workstreams:
  - `docs/workstreams/ffmpeg-hardware-pipeline-planner/`
- Reference pressure:
  - `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Encoder/EncoderValidator.cs`
  - `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Encoder/MediaEncoder.cs`
  - `repo-ref/jellyfin/MediaBrowser.Controller/MediaEncoding/EncodingHelper.cs`

## Problem

Current Nako hardware reports can represent stage facts, but the detector still
mostly fills those facts from encoder names and static assumptions. This means:

- a listed hardware encoder can make a pipeline look healthier than it is;
- decoder and hwaccel absence is not visible in Admin diagnostics;
- filter requirements such as `scale_vaapi`, `hwupload`, `scale_qsv`, or future
  tone mapping cannot be proven by the current report;
- bitstream filters cannot be represented as runtime evidence.

## Target State

`nako-transcode` owns an `FfmpegProbeInventory` that contains redaction-safe
sets of:

- encoders;
- decoders;
- hwaccels;
- filters;
- bitstream filters.

`FfmpegHardwareAccelerationDetector` builds `HardwareAccelerationReport` from
that inventory. Stage capabilities should be marked listed/missing per feature
instead of being static guesses when real probe output is present.

## In Scope

- Parse FFmpeg list outputs into a structured probe inventory.
- Run bounded FFmpeg discovery commands at startup for encoders, decoders,
  hwaccels, filters, and bitstream filters.
- Map inventory facts into `HardwareStageCapability`.
- Preserve existing device initialization and smoke-probe behavior.
- Update Admin diagnostics and tests where stage capability evidence changes.
- Keep Public Client API hardware-redacted.

## Out Of Scope

- Full FFmpeg version validation.
- Filter option probing.
- Hardware device path configuration.
- HDR tone mapping command generation.
- Subtitle burn-in command generation.
- Adaptive HLS ladder generation.
- Remote transcode workers.
- Copying Jellyfin implementation details.

## Architecture Direction

The detector is an Adapter over FFmpeg process execution. It should produce a
small structured inventory, then pure mapping code should build
`HardwareAccelerationReport`.

The rest of the playback stack should not know how FFmpeg list output is parsed.
Pipeline planning should keep consuming `HardwareAccelerationReport`.

## Closeout Condition

This lane can close when:

- `FfmpegProbeInventory` exists and is covered by characterization tests;
- startup detector runs the required FFmpeg list commands and degrades safely on
  failure;
- stage capabilities for the current accelerators are populated from inventory
  facts;
- Admin diagnostics expose the richer stage evidence without public API leakage;
- focused `nako-transcode`, `nako-api`, and `nako-server playback` gates pass.

## Shipped Outcome

- `nako-transcode` now has `FfmpegProbeInventory` for parsed encoders,
  decoders, hwaccels, filters, and bitstream filters.
- `FfmpegHardwareAccelerationDetector` runs the five FFmpeg discovery commands
  and degrades to probe-error reports when any required probe command fails.
- `HardwareAccelerationReport` stage capabilities are derived from inventory
  facts and distinguish required capabilities from optional evidence.
- HLS hardware decode arguments are emitted before `-i`.
- Admin diagnostics expose stage `required` evidence while Public Client APIs
  remain hardware-redacted.
