# Casting Renderer Runtime - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The casting lane is active. ADR 0040 defines casting as Renderer Sessions plus
Renderer Adapters. `playback-policy-and-renderer-targets` is closed, so this
lane can consume policy-aware target planning, safe target DTOs, and Admin
policy diagnostics.

## Active Task

- Task ID: CAST-020
- Owner: codex
- Files: `crates/nako-server/src/http/tests/playback.rs`,
  `crates/nako-server/src/app/tests/playback.rs`, `crates/nako-client-protocol/src`
- Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-client-protocol public --no-fail-fast`
- Status: READY
- Review: pending
- Evidence: pending

## Decisions Since Last Update

- Nako-to-Nako cast is the first implementation target.
- Chromecast, DLNA, and AirPlay are future Renderer Adapters.
- Renderer Session is separate from Playback Session and Transcode Session.
- Cast-safe URLs/tickets are introduced only when a target cannot use bearer
  auth or an authenticated Nako client channel.

## Blockers

- None.

## Next Recommended Action

Start CAST-020 by adding characterization tests for the missing Renderer
Session/control surface and the existing Playback Session behavior the casting
lane must preserve.
