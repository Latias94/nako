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

CAST-030 is complete. Renderer Session and Renderer Command are now core
records with durable SQLite/PostgreSQL repository adapters. Renderer Session is
the controllable target identity; Playback Session remains the media playback
attempt; Transcode Session remains an internal artifact.

## Active Task

- Task ID: CAST-040
- Owner: codex
- Files: `crates/nako-server/src/app`, `crates/nako-server/src/http`,
  `crates/nako-client-protocol/src`
- Validation: `cargo nextest run -p nako-server renderer --no-fail-fast`;
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
- CAST-020 fixed current gaps in tests before adding the Renderer Session
  domain.
- CAST-030 chose durable renderer persistence now because command polling and
  future adapter processes need a stable queue boundary.

## Blockers

- None.

## Next Recommended Action

Start CAST-040 by adding Nako-to-Nako renderer registration, heartbeat,
controllable target listing, and command polling/delivery through Public Client
API routes.
