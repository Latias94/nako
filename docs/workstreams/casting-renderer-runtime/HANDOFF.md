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

CAST-040 is complete. Public Client now has Nako-to-Nako renderer registration,
heartbeat/capability update, controllable target listing, command polling, and
command completion. The server-side boundary is `RendererAppService`, which
keeps owner checks, TTL expiry, target validation, capability normalization,
and command lifecycle rules out of HTTP handlers. Public OpenAPI and generated
TypeScript/Kotlin SDK outputs were refreshed with the renderer surface.

## Active Task

- Task ID: CAST-050
- Owner: codex
- Files: `crates/nako-server/src/app/playback`, `crates/nako-server/src/app`,
  `crates/nako-server/src/http`
- Validation: `cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast`
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
- CAST-040 kept external cast protocols out of Public Client registration.
  Only Nako remote/native renderer targets using bearer auth can register
  through `/renderers`.

## Blockers

- None.

## Next Recommended Action

Start CAST-050 by adding an authorized controller route/service method that
queues a play command for a renderer target through the existing policy-aware
Playback App Service. Denied policy/control must not create Playback Sessions,
Transcode Sessions, browser tickets, or renderer commands.
