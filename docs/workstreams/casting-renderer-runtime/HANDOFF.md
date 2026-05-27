# Casting Renderer Runtime - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The casting lane is active. ADR 0040 defines casting as Renderer Sessions plus
Renderer Adapters. `playback-policy-and-renderer-targets` is closed, so this
lane can consume policy-aware target planning, safe target DTOs, and Admin
policy diagnostics.

CAST-020 is complete. Current behavior is now characterized: Nako has durable
Playback Sessions, playback heartbeat, browser tickets, and safe target
vocabulary, but no Public Client renderer registration, renderer session,
remote-control command, or cast route surface yet.

## Active Task

- Task ID: CAST-030
- Owner: codex
- Files: `crates/nako-core/src`, `crates/nako-db/src`
- Validation: `cargo nextest run -p nako-core renderer --no-fail-fast`;
  `cargo nextest run -p nako-db renderer --no-fail-fast`
- Status: READY
- Review: pending
- Evidence: pending

## Decisions Since Last Update

- Nako-to-Nako cast is the first implementation target.
- Chromecast, DLNA, and AirPlay are future Renderer Adapters.
- Renderer Session is separate from Playback Session and Transcode Session.
- Cast-safe URLs/tickets are introduced only when a target cannot use bearer
  auth or an authenticated Nako client channel.
- CAST-020 fixed current gaps in tests before adding the Renderer Session
  domain.

## Blockers

- None.

## Next Recommended Action

Start CAST-030 by adding Renderer Session and Renderer Command domain records
at the core boundary, then decide whether the first repository is durable or
process-local with an explicit extraction path.
