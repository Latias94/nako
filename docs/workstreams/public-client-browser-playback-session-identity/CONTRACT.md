# Public Client Browser Playback Session Identity Contract

Status: Frozen for PBSI-020
Last updated: 2026-05-29

## Purpose

This document freezes the Public Client contract that lets browser playback
start with token-safe media URLs and heartbeat against the same durable playback
session. It is the implementation target for PBSI-030 and PBSI-040.

The contract follows:

- ADR-0023 for the public v1 compatibility and error envelope;
- ADR-0025 for generated Public Client SDK ownership;
- the existing playback-session inspection and heartbeat routes;
- the WMLP follow-on that forbids mining media response headers for heartbeat
  authority.

## Response Shape

`POST /sources/{source_id}/playback/browser-ticket` extends
`BrowserPlaybackTicketResponse` with a stable playback session identity:

```rust
pub struct BrowserPlaybackTicketResponse {
    pub source_id: String,
    pub item_id: Option<String>,
    pub playback_session_id: Option<String>,
    pub mode: BrowserPlaybackMode,
    pub expires_at: String,
    pub urls: Vec<BrowserPlaybackUrlDto>,
}
```

OpenAPI must model `playback_session_id` as a required nullable UUID property.
Generated TypeScript should expose it as:

```typescript
playback_session_id: string | null;
```

## Mode Semantics

`playback_session_id` values are mode-dependent:

| Mode | Value | Reason |
| --- | --- | --- |
| `direct` | Non-null | Direct browser media playback needs heartbeat before the first media request completes. |
| `remux` | Non-null | Remux playback must heartbeat the same durable session that owns the remux artifact link. |
| `hls` | Non-null | HLS playback must heartbeat the same durable session used by playlist and segment control routes. |
| `subtitle` | `null` | Subtitle tickets are ancillary fetch authority and do not create standalone playback sessions. |

Clients must use the primary media ticket response, not a subtitle ticket
response, as heartbeat authority.

## Ticket And Session Binding

For non-subtitle browser tickets, the server must allocate a durable playback
session before returning the JSON ticket response. The issued opaque browser
ticket must be bound server-side to that `playback_session_id`.

Media endpoints reached with the issued ticket must attach to that bound
playback session. They must not create a second playback session for the same
ticketed playback. For remux and HLS, the transcode artifact may be started or
linked when the media URL is first consumed, but it must link back to the
pre-created playback session exposed in JSON.

`expires_at` remains the browser-ticket expiry, not a playback-session expiry.
Expired tickets return the public error envelope with `unauthorized`. Existing
playback sessions remain inspectable by their normal session routes unless they
are deleted by a future retention policy.

## Heartbeat Authority

Browser web clients must heartbeat with:

```text
POST /playback/sessions/{playback_session_id}/heartbeat
```

The request is authenticated by the normal Public Client bearer/session
principal, not by the opaque media ticket. The server must require:

- the authenticated principal owns the playback session;
- the authenticated principal still has effective `play` access to the session
  source;
- the session is active enough to accept heartbeat updates.

Missing or inaccessible sessions return `404 not_found` to avoid exposing
session existence. Malformed IDs return `400 invalid_input`. Terminal sessions
that cannot accept heartbeat return `409 conflict`.

## Media URL Safety

`urls[]` remain token-safe media URLs:

- no bearer tokens;
- no raw Source Locator values;
- no local filesystem paths;
- no transcode output paths;
- no renderer session IDs, renderer command IDs, cast tickets, or network-scope
  fields in the browser ticket JSON.

The top-level browser-ticket media URL does not need to expose
`playback_session_id`; the opaque ticket is the media authorization handle and
the JSON field is the control-plane heartbeat handle. HLS segment URLs may
remain session-scoped implementation routes, but clients must not discover
heartbeat identity by parsing media URLs, playlists, or response headers.

## Header Compatibility

Binary media responses may continue to include
`x-nako-playback-session-id` for diagnostics and backward compatibility.
That header is not the Public Client heartbeat contract. `web/` and generated
SDK consumers must use `BrowserPlaybackTicketResponse.playback_session_id`.

## SDK Expectations

Generated SDKs should keep:

```typescript
createBrowserPlaybackTicket(
  sourceId: string,
  body: BrowserPlaybackTicketRequest,
): Promise<BrowserPlaybackTicketResponse>
```

and update `BrowserPlaybackTicketResponse` to include required nullable
`playback_session_id`.

Rust client support should deserialize the same field without requiring callers
to read binary stream headers.

## Web Expectations

`web/` may wire heartbeat only after PBSI-030 updates the server/API/SDK
contract. Until then, playback may start with ticketed URLs, but heartbeat must
remain a truthful readiness gap instead of guessing from headers or media URLs.

## Required Implementation Test Changes

PBSI-030 should replace the existing no-session assertion with:

- `direct`, `remux`, and `hls` browser ticket responses include a non-null
  `playback_session_id`;
- `subtitle` browser ticket responses include `playback_session_id: null`;
- browser ticket JSON still has no renderer/cast transport fields;
- media requests made with a browser ticket use the same session id as the JSON
  response;
- heartbeat succeeds for the owning principal and fails for non-owners or
  principals without source play access.
