# Playback Transcode Ops Hardening

Status: Active
Last updated: 2026-05-22

## Purpose

This workstream turns the existing **Playback Runtime**, transcode runtime, and
Admin playback diagnostics into an operator-supportable product surface.

The baseline is already stronger than an MVP: Taru has playback source
selection, direct play/remux/HLS orchestration, FFmpeg hardware capability
probing, resource budgets, persisted sessions, cancellation, staging, and a
safe Admin runtime diagnostics route. The remaining product risk is that an
operator still needs clearer readiness, fallback, validation, failure, and
support evidence before playback/transcode issues can be diagnosed without
reading local paths, FFmpeg command lines, logs, or secrets.

## Current Decision

PTOH-010 opens the lane as the next mainline child of
`post-rpd-product-hardening`.

The first executable task is PTOH-020: define and prove a stable playback
runtime readiness contract on top of the existing Admin playback runtime
diagnostics and `taru-transcode` hardware evidence.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [transcode-runtime](../transcode-runtime/README.md)
- [playback-streaming](../playback-streaming/README.md)
- [admin-playback-runtime-diagnostics](../admin-playback-runtime-diagnostics/README.md)
- [admin-playback-session-read-model](../admin-playback-session-read-model/README.md)
- [playback-source-selection-deepening](../playback-source-selection-deepening/README.md)

## Boundary

This lane is runtime and diagnostics focused. It must not introduce downloader
protocols, NFO/sidecar writes, metadata authority changes, Addon runtime
distribution, adaptive bitrate ladders, direct remote FFmpeg credentials, or a
distributed transcode queue.
