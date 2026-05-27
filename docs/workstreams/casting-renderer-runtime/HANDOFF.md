# Casting Renderer Runtime - Handoff

Status: Planned
Last updated: 2026-05-27

## Current State

The casting lane is opened as a planned follow-on. ADR 0040 defines casting as
Renderer Sessions plus Renderer Adapters. This lane should not start
implementation until `playback-policy-and-renderer-targets` closes or provides
an explicit handoff with policy/target records ready to consume.

## Active Task

- Task ID: CAST-020
- Owner: codex
- Files: `crates/nako-server/src/http/tests/playback.rs`,
  `crates/nako-server/src/app/tests/playback.rs`, `crates/nako-client-protocol/src`
- Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-client-protocol public --no-fail-fast`
- Status: BLOCKED
- Review: pending
- Evidence: pending

## Decisions Since Last Update

- Nako-to-Nako cast is the first implementation target.
- Chromecast, DLNA, and AirPlay are future Renderer Adapters.
- Renderer Session is separate from Playback Session and Transcode Session.
- Cast-safe URLs/tickets are introduced only when a target cannot use bearer
  auth or an authenticated Nako client channel.

## Blockers

- Waiting for `playback-policy-and-renderer-targets` to deliver policy-aware
  target planning.

## Next Recommended Action

After PRT-070, start CAST-020 by adding characterization tests for the missing
Renderer Session/control surface and the existing Playback Session behavior the
casting lane must preserve.
