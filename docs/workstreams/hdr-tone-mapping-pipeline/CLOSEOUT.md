# HDR Tone Mapping Pipeline - Closeout

Date: 2026-05-31
Status: Closed

## Result

`HTP-010` through `HTP-040` are complete. Nako now models playback-owned color
pipeline requirements and carries HDR-to-SDR tone-mapping intent through the
transcode-owned HLS runtime/profile/execution policy into deterministic
software FFmpeg command planning.

The shipped slice keeps `nako-transcode` independent from `nako-playback` and
keeps server playback code at the mapping/composition boundary. Server routes
do not assemble raw FFmpeg requests or filter chains.

## Final Gates

```text
cargo nextest run -p nako-playback hdr --no-fail-fast
cargo nextest run -p nako-transcode hdr --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json
git diff --check
```

All implementation gates passed on 2026-05-31 before merge. Closeout validates
the manifest and docs diff after the lane status update.

## Follow-ons

- hardware tone mapping for VAAPI, QSV, NVENC, AMF, VideoToolbox, OpenCL, and
  vendor-specific filter chains;
- Dolby Vision and HDR10+ dynamic metadata handling or preservation;
- richer display capability inputs and device profile databases;
- operator hardware smoke matrices and release diagnostics;
- UI/client controls for HDR behavior.

## Residual Risks

The first slice is intentionally software-first. It makes HDR content watchable
on SDR HLS clients, but it does not prove GPU tone-map quality, driver-specific
filter availability, dynamic HDR preservation, or real-device display behavior.
