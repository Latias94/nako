# Playback Media Maturity First Slices - Handoff

Status: Completed
Last updated: 2026-05-28

## Current State

Workstream completed for the first closable media maturity slices after the
playback runtime and rendition-planning refactors landed.

## Completed

- Public Client capability DTOs, OpenAPI, generated SDKs, Rust client query
  helpers, server adapters, and renderer registration now carry the richer
  capability profile fields.
- Playback target profiles consume client bitrate, resolution, audio-channel,
  HDR, subtitle, and HLS planning preferences.
- Transcode requirements can carry HLS single/adaptive and MPEG-TS/fMP4
  planning intent without changing current executable HLS output.
- Planner tests cover bitrate, resolution, HDR, audio-channel, subtitle, and
  HLS planning requirements.

## Follow-Ons

- Executable adaptive HLS ladders.
- Executable fMP4/CMAF segment output and serving.
- DLNA-style device profile import/adaptation if external renderer work needs
  it.
- Subtitle burn-in/sidecar execution beyond the existing selected-subtitle
  planning reason.

## Guardrails

- Current executable HLS output remains unchanged.
- Preserve Public/Admin redaction semantics.
- Prefer focused tests in `nako-playback`, `nako-client-protocol`, `nako-api`,
  and `nako-server` before broader gates.
