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

CAST-050 is complete. Public Client now has
`POST /renderers/{renderer_session_id}/commands/play`. `CastingAppService`
orchestrates Renderer Session checks and Playback App Service policy/session
creation. The play path creates only direct-play Playback Sessions for now,
queues a renderer `play` command, and attaches the session to the renderer. If
remote control/cast/playback policy denies the action, tests prove no Playback
Session, Transcode Session, ticket, or Renderer Command is created.

## Active Task

- Task ID: CAST-060
- Owner: codex
- Files: `crates/nako-api/src`, `crates/nako-server/src/http/admin.rs`,
  `docs/workstreams/casting-renderer-runtime`
- Validation: `cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast`;
  `cargo nextest run -p nako-server renderer --no-fail-fast`
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
- CAST-050 kept remux/HLS renderer transport out of the play command path until
  cast-safe URLs and target-specific adapter contracts are explicit.

## Blockers

- None.

## Next Recommended Action

Start CAST-060 by adding redaction-safe Admin diagnostics for renderer runtime
state/readiness and splitting external adapter follow-ons for Chromecast, DLNA,
AirPlay, plus non-direct Nako renderer transport.
