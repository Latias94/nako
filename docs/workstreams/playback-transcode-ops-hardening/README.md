# Playback Transcode Ops Hardening

Status: Complete
Last updated: 2026-05-22

## Purpose

This workstream turns the existing **Playback Runtime**, transcode runtime, and
Admin playback diagnostics into an operator-supportable product surface.

The baseline is already stronger than an MVP: Nako has playback source
selection, direct play/remux/HLS orchestration, FFmpeg hardware capability
probing, resource budgets, persisted sessions, cancellation, staging, and a
safe Admin runtime diagnostics route. The remaining product risk is that an
operator still needs clearer readiness, fallback, validation, failure, and
support evidence before playback/transcode issues can be diagnosed without
reading local paths, FFmpeg command lines, logs, or secrets.

## Current Decision

This lane is complete. PTOH-020 through PTOH-050 shipped a runtime/readiness,
validation/fallback, failure-taxonomy, and bounded Admin support evidence
surface for Playback Runtime supportability. PTOH-060 closes the lane and
returns routing to `post-rpd-product-hardening`.

Downloadable support bundles, Admin UI workflows, adaptive ladders, optimized
versions, downloader/watch-folder acquisition, network traversal, AI, and Addon
runtime behavior remain split follow-ons.

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
