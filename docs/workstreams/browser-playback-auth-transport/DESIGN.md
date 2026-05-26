# Browser Playback Auth Transport

Status: Completed
Last updated: 2026-05-26

## Why This Lane Exists

Media Web now has browse, detail, Source/Version Picker, playback decision
preview, and User Playback State writes. It still cannot render a real browser
player safely because a normal `<video src>` request cannot attach bearer
headers.

The transport must preserve Nako's Public Client API boundary: viewer playback
uses current-principal Library Access and playback policy, not Admin API state
and not raw Source Locators.

## Target State

When this lane closes:

- Media Web can request a browser-safe playback transport for a Media Source.
- The browser player can use a real media element or accepted JavaScript player
  path without exposing bearer tokens, raw Source Locators, local paths, or
  privileged permanent stream URLs.
- Direct stream, remux, and HLS behavior are either supported or explicitly
  scoped with clear client-safe errors.
- Range requests keep working for browser media playback.
- Playback progress writes are connected to User Playback State.
- Public OpenAPI and TypeScript SDK expose the accepted contract.
- Admin diagnostics remain Admin Web-owned.

## In Scope

- Browser playback transport decision matrix and threat model.
- Public Client playback ticket or equivalent accepted transport contract.
- Server-side validation that preserves Library Access and playback policy.
- Stream/remux/HLS URL issuance or header-capable playback path.
- TypeScript SDK regeneration.
- Media Web real player integration for the accepted MVP transport.
- User Playback State progress writes from real player events.
- Browser smoke evidence on desktop and mobile viewports.

## Out Of Scope

- Public username/password login and persistent browser sessions unless the
  selected transport requires a narrow session prerequisite.
- Public self-registration or invitation redemption.
- Desktop Tauri native playback core and hardware decode.
- Native mobile playback.
- Recommendations or streaming-storefront behavior.
- Admin Web playback diagnostics beyond links to existing Admin routes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Short-lived playback tickets are the accepted browser transport. | High | ADR 0036 records the decision after comparing tickets, cookie/session auth, and JavaScript HLS/MSE with headers. | Revisit only if implementation proves native browser media playback cannot renew or protect tickets safely. |
| Ticket issuance should be Public Client API-owned. | High | Playback must respect current principal, Library Access, and ADR 0028 user state boundaries. | Add an ADR before exposing any Admin-owned or privileged transport. |
| HLS segments need the same protection as playlists. | High | Playlist-only auth would leak segment URLs or create bypass paths. | Ticket/session scope must cover playlist and segments together. |
| Real browser playback can start with direct/remux MVP. | Medium | Some local media will be browser-playable or remuxable. | HLS/transcode may need to move earlier if browser codec support is too narrow. |

## Transport Options

### Short-Lived Playback Tickets

The server issues scoped, expiring playback tickets for a source and playback
mode. The browser media element receives a URL containing only the ticket, not a
bearer token or source locator. The server validates ticket scope, expiry,
principal grants, source identity, playback mode, and optional session state on
each stream request.

Recommended default unless BPAT-010 rejects it.

### Cookie Or Session Auth

The browser uses same-site session cookies for stream requests. This can be
clean for `<video src>` but depends on credential/session work and has CSRF,
reverse proxy, same-site, and logout semantics.

Good long-term candidate, but likely too coupled to Credential And Session UX
for the first playback transport lane.

### JavaScript HLS Or MSE With Headers

A JavaScript player fetches media segments with Authorization headers. This can
work for HLS/MSE, but it does not solve native direct `<video src>` and adds
browser compatibility and memory buffering constraints.

Useful for future HLS playback, not the only first transport.

## Architecture Direction

The accepted contract shape is:

1. Public Client requests a playback decision.
2. Public Client requests a short-lived browser playback ticket for the selected
   source and mode.
3. Server returns browser-safe playback URLs or a compact ticket response.
4. Browser player uses those URLs without bearer headers.
5. Server validates every stream, remux, playlist, and segment request against
   ticket scope and current playback policy.
6. Media Web writes progress through User Playback State.

The exact route names are finalized in BPAT-020 against ADR 0036.

## Security Requirements

- Tickets must be scoped to source, mode, principal or session, and expiry.
- Tickets must carry only opaque secret material in browser-visible URLs.
- Tickets must not encode raw Source Locators, local paths, or storage handles
  in client-readable form.
- Ticket validation must enforce Library Access at issuance and at use, or
  prove why revalidation at use is unnecessary for the ticket TTL.
- Stream URLs must not be permanent.
- HLS playlists must not leak unprotected segment URLs.
- Range requests must not bypass auth validation.
- Logs and UI must redact ticket values.

## Closeout Condition

This lane can close when:

- an accepted browser playback transport exists;
- a real Media Web browser player uses it;
- progress writes are connected to real playback events;
- Public OpenAPI and SDK are current;
- relevant Rust and frontend gates pass;
- browser smoke proves desktop/mobile playback behavior or a named codec-safe
  fixture limitation;
- follow-ons for desktop native playback and full credential/session UX remain
  split.

Result: CLOSED 2026-05-26. The lane shipped short-lived browser playback
tickets, server-side ticket validation for direct/remux/HLS byte routes, Media
Web HTML5 player integration, and User Playback State writes from player
events. Fixture browser smoke covers desktop and mobile layouts; fixture media
URLs intentionally log a media-load error because they are not backed by a real
media server. Desktop native playback, subtitles, advanced codec/HDR handling,
credential/session UX, and account/admin role work remain follow-ons.
