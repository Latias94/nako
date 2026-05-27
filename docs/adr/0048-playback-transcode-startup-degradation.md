# 0048: Playback Transcode Startup Degradation

## Status

Accepted.

## Context

ADR 0047 made CPU HLS transcode readiness executable: probe-derived CPU
readiness requires the software encoders needed for the current HLS H.264/AAC
pipeline. That makes the planner correct, but it exposes a service boundary
problem: playback service construction currently expects a valid HLS pipeline
plan during startup.

That couples the whole playback/admin surface to one optional runtime capability.
For a media server, browse, admin diagnostics, direct play, remux, and renderer
control should remain available when HLS transcode is unavailable. Operators
need a running admin surface that says what is missing.

## Decision

Nako will treat HLS transcode readiness as a runtime capability, not a server
startup invariant.

Playback startup may continue when the default HLS transcode pipeline is
unavailable. The playback service must keep:

- the hardware report;
- the configured hardware policy;
- a typed `TranscodePipelineReadiness`;
- an executable `TranscodePipelinePlan` only when one exists.

Admin playback runtime diagnostics will report unavailable transcode readiness
without hiding the missing capability. HLS execution paths still plan before
running and must reject unavailable transcode with a typed unsupported error.

## Consequences

- Admin, browse, direct play, and other non-HLS surfaces can start even when
  FFmpeg cannot execute the configured HLS path.
- HLS transcode requests fail at the HLS planning boundary instead of process
  startup.
- Runtime diagnostics become the authority for operator remediation.
- Startup no longer needs to invent a fake CPU fallback just to keep the service
  alive.

## Alternatives Considered

- **Keep failing startup when HLS is unavailable:** rejected because it blocks
  admin remediation and unrelated playback modes.
- **Keep CPU always available:** rejected by ADR 0047 because it creates late
  runtime failures.
- **Disable the whole playback service when transcode is unavailable:** rejected
  because direct play and remux do not require the HLS transcode path.

## Related Workstreams

- `docs/workstreams/playback-transcode-startup-degradation/`
- `docs/workstreams/cpu-transcode-readiness/`
