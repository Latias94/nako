# Public Client Browser Playback Session Identity - Design

Status: Active
Last updated: 2026-05-28

## Problem

WMLP proved that the new `web/` Media player can render browser-ticket media and
subtitle URLs without embedding bearer tokens. It also proved a contract gap:
`BrowserPlaybackTicketResponse` does not expose a stable playback session id, so
the web client cannot call `heartbeatPlaybackSession` honestly after starting
browser playback.

## Target State

When this lane closes:

- Browser playback ticket creation returns a stable web-visible playback
  session identity, or another explicit heartbeat authority accepted by the
  Public Client contract.
- The identity can be used with existing playback-session inspection and
  heartbeat routes without leaking bearer tokens, raw source locators, local
  paths, transcode internals, or renderer-only state.
- TypeScript/Rust SDKs expose the new shape.
- `web/` sends heartbeat/progress evidence through the accepted contract.
- Tests prove browser-ticket URLs remain token-safe and session identity is
  present only in JSON/control paths, not in permanent media URLs.

## Scope

In scope:

- Public Client DTO/SDK contract for browser playback session identity.
- Server/API mapping and route tests for browser ticket creation.
- Web Public Media data-source and player heartbeat integration.
- No-token and redaction assertions.

Out of scope:

- Desktop native playback.
- Renderer/cast session semantics.
- Changing HLS segment authorization policy.
- Persisting extra user playback state fields beyond heartbeat/progress.

## Architecture Direction

Prefer extending the browser ticket response with `playback_session_id` if the
server already creates a durable playback session during ticket planning. If a
ticket does not always create a playback session, define the earliest point at
which a session id becomes available and expose a control-plane JSON response
for it. Do not require the web player to parse media playlist headers to learn
control identity.
