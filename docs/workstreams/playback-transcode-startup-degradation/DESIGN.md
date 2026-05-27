# Playback Transcode Startup Degradation

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

CPU readiness is now correctly probe-derived. The next boundary problem is that
playback startup still expects a default executable HLS plan. That is too
coarse: HLS transcode can be unavailable while admin diagnostics, library
browse, direct play, remux, and renderer control should continue.

## Relevant Authority

- `docs/adr/0047-cpu-transcode-readiness.md`
- `docs/adr/0048-playback-transcode-startup-degradation.md`
- `docs/workstreams/cpu-transcode-readiness/`

## Target State

`HlsAppService` keeps readiness and an optional executable plan separately.
Startup succeeds with an unavailable HLS pipeline, admin diagnostics report the
typed readiness reason, and HLS execution still rejects unavailable planning
before spawning FFmpeg.

## In Scope

- Split HLS startup state into readiness plus optional executable plan.
- Update playback runtime diagnostics to carry readiness without requiring a
  successful plan.
- Report selected HLS slots as zero when no HLS plan is executable.
- Make selected fallback readiness unavailable when the pipeline itself is
  unavailable.
- Add server tests for startup/admin diagnostics when CPU software encoders are
  missing.

## Out Of Scope

- Frontend rendering.
- Source-codec decoder matrix.
- HLS muxer/protocol probing.
- Remote worker transcode scheduling.
- Changing direct play or remux policy semantics.

## Closeout Condition

This lane can close when:

- playback startup no longer fails solely because configured HLS transcode is
  unavailable;
- admin playback runtime diagnostics expose unavailable HLS readiness;
- HLS request planning still rejects unavailable transcode before execution;
- focused playback/admin gates pass.

## Closeout Summary

Completed on 2026-05-27.

`HlsAppService` now stores transcode readiness separately from an optional
executable HLS plan. Startup continues when the configured HLS pipeline is
unavailable, while HLS execution policy planning still rejects before FFmpeg is
spawned. Admin runtime diagnostics use the readiness record directly and report
zero selected HLS slots when no executable plan exists.

The old behavior where `fallback=fail` could abort server startup has been
removed. The failure now belongs at the HLS planning boundary and the admin
runtime surface.
