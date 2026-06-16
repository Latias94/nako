# Playback Policy Denial Before Runtime Work

## Problem

Nako already enforces user playback policy for Direct Play, Remux, HLS, and
renderer playback routes. The remaining evidence gap is narrower: policy
denials must be proven to happen before expensive or stateful Playback Runtime
work starts.

For a self-hosted media server this is both an access-control and resource-cost
boundary. A denied user request must not create playback sessions, transcode
sessions, FFmpeg input staging records, or start FFmpeg work before returning
the policy error.

## Scope

In scope:

- Strengthen existing app-level playback tests for user-facing playback entry
  points.
- Prove Remux and HLS policy denial creates no playback session, no transcode
  session, no FFmpeg input staging record, and no FFmpeg output/start marker.
- Preserve current Direct Play, browser ticket, and renderer policy behavior.
- Refactor only if the strengthened test exposes policy checks after runtime
  work.

Out of scope:

- Changing the internal `remux_source` no-principal entry point.
- Adding new Public Client API fields or generated contract changes.
- Changing playback planner compatibility rules.
- Changing Addon playback resource suggestions, renderer DTO shape, or HLS
  artifact cache behavior.

## Requirements

- R1. Remux playback stream policy denial must fail before playback session
  creation, transcode session creation, FFmpeg input staging, or FFmpeg output
  creation.
- R2. HLS playlist playback policy denial must fail before playback session
  creation, transcode session creation, FFmpeg input staging, or HLS playlist
  output creation.
- R3. Existing browse-only access checks must still hide lower-level playback
  policy details.
- R4. Tests must use existing playback app-service fixtures and avoid route or
  DTO churn unless a real route-level bug is found.

## Acceptance

- A focused `nako-server` nextest filter covering the changed playback tests
  passes.
- `cargo fmt --all -- --check` passes.
- `git diff --check` passes.
- Trellis task validation passes.

## Notes

- The current code already has policy-denial tests for Direct, Remux, HLS, and
  renderer playback. This task should deepen those tests rather than duplicate
  route status-code coverage.
- `remux_source_currently_starts_without_principal_or_playback_policy` documents
  an internal entry point and is not a user playback policy contract.
