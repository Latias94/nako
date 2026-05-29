# HLS Seek Restart Lifecycle - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

HSRL-020 is complete. HLS request variant identity now has an optional
`HlsPlaybackGeneration`; default `0 ms` generation preserves current request
identity, and non-zero starts isolate request identity plus staging layout.

## Active Task

- HSRL-030: restart admission policy.

## Decisions Since Last Update

- Default playback start is `0 ms` and must preserve current request keys.
- Non-zero starts become part of `HlsRequestVariantPlan` identity.
- Runtime cancellation and FFmpeg seek flags are follow-on tasks.
- No public HTTP seek API was added in HSRL-020.

## Blockers

- None.

## Next Recommended Action

Implement HSRL-030 by making same-generation reuse and superseding-generation
restart/cancellation explicit in `HlsAppService` admission.
