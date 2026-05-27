# 0045: FFmpeg Hardware Pipeline Planner

## Status

Accepted.

## Context

ADR 0044 made playback decisions profile-driven and left a deliberate seam for
turning a **Playback Transcode** requirement into an execution pipeline. The
remaining risk is that `nako-transcode` still mostly treats hardware support as
"is a H.264 encoder listed by FFmpeg", then lets the FFmpeg command builder
branch directly on a selected accelerator.

That model is too shallow for Jellyfin-class playback pressure:

- hardware decode and hardware encode are different capabilities;
- filters such as scale, tone-map, format conversion, subtitle burn-in, and
  upload/download may require their own hardware context;
- NVENC does not automatically imply NVDEC;
- QSV, VAAPI, AMF, VideoToolbox, and future adapters require different device
  initialization and command shapes;
- Admin diagnostics need to explain which stage failed without exposing raw
  FFmpeg commands, host paths, or credentials.

`repo-ref/jellyfin` shows the mature shape: encoder validation separately
tracks decoders, encoders, filters, hwaccels, bitstream filters, devices, and
codec-specific options. Nako must use that feature pressure without copying
Jellyfin code, schemas, tests, comments, or assets.

## Decision

Nako will introduce a **Transcode Pipeline Planner** inside `nako-transcode`.
It owns the pure decision that maps a playback transcode requirement plus a
cached **Hardware Capability Report** and **Hardware Acceleration Policy** into
typed decode/filter/encode stages.

The ownership split is:

- `nako-playback` produces playback decisions and output requirements.
- `nako-transcode` owns hardware capability inventory, pipeline planning, FFmpeg
  command planning, and transcode profile identity.
- `nako-server` owns configuration, runtime startup, persistence, and Admin
  diagnostics mapping.
- `nako-api` exposes redaction-safe Admin DTOs only.
- Public Client API remains free of hardware command details.

The target module shape is:

```text
TranscodePipelineRequest
  output container
  output codecs
  track/subtitle/output constraints
  Hardware Acceleration Policy
  Hardware Capability Report

TranscodePipelinePlan
  decode stage
  filter stage
  encode stage
  fallback evidence
  unsupported reason when fail policy blocks fallback
```

Hardware inventory must stop being encoder-only. It should include stage-level
capabilities for decoder, encoder, filter, hwaccel, bitstream-filter, device
initialization, and smoke-probe evidence. The first implementation can still
execute a narrow HLS H.264/AAC output, but the internal capability shape must be
able to represent NVENC/NVDEC, QSV, VAAPI, AMF, and VideoToolbox without
rewriting app-service orchestration.

FFmpeg command construction must consume a `TranscodePipelinePlan` or a
`TranscodeExecutionPolicy` derived from it. The command builder should not
select hardware policy from config or probe output.

## Consequences

- Hardware selection becomes stage-aware and explainable.
- The old "selected accelerator equals encoder path" model can be deleted.
- NVENC encode-only, VAAPI full pipeline, QSV decode+encode, AMF, and
  VideoToolbox can be added as capability adapters instead of route branches.
- Admin diagnostics can report stage readiness while continuing to redact
  command strings and paths.
- Existing HLS behavior remains narrow but sits on a deeper seam.
- Tests must cover pipeline selection, fallback, fail-policy, command planning,
  and redaction-safe diagnostics.

## Alternatives Considered

- **Keep `TranscodeAccelerationPlan::from_hardware_selection`:** rejected
  because it collapses device inventory, decode, filter, and encode into one
  accelerator choice.
- **Put hardware decisions in `nako-server`:** rejected because server config is
  an Adapter concern; pipeline semantics belong with transcode execution.
- **Let FFmpeg command builder infer hardware from available encoders:** rejected
  because command building is too low-level to own policy, fallback, and Admin
  evidence.
- **Adopt Jellyfin's encoding model directly:** rejected because Nako has its
  own domain records and repo reference rules forbid copying reference code.

## Related Workstreams

- `docs/workstreams/ffmpeg-hardware-pipeline-planner/`
- `docs/workstreams/playback-capability-profile-planner/`
- `docs/workstreams/playback-transcode-policy-deepening/`
