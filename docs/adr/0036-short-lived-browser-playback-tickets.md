# 0036: Use Short-Lived Playback Tickets for Browser Media Transport

## Status

Accepted.

## Context

Nako's first inbound access boundary is bearer-token based. That works for JSON
Public Client API requests and SDK `fetch` calls, but it does not work for a
normal browser media element:

- `<video src>` cannot attach an `Authorization` header;
- native HLS behavior cannot reliably attach bearer headers across platforms;
- direct `fetch` playback is not a robust replacement for browser media range
requests;
- putting the long-lived bearer token into a media URL would expose the wrong
secret through history, logs, referrers, devtools, and copied links.

Media Web therefore needs a browser playback transport that preserves Library
Access and playback policy without exposing bearer tokens, raw Source Locators,
local filesystem paths, or privileged permanent stream URLs.

## Decision

Nako will use **short-lived browser playback tickets** as the first browser
media transport.

An authenticated Public Client request may ask the server to issue a playback
ticket for a specific Media Source and playback mode. The ticket is an opaque
secret for browser media requests only. It is not the user's bearer token, not a
Source Locator, and not a permanent URL.

The first contract shape is:

- bearer-authenticated JSON route to issue a browser playback ticket for a
  Media Source;
- request fields include the requested playback mode (`direct`, `remux`, or
  `hls`) and the client playback capabilities needed to choose the URL shape;
- response fields include source id, item id when available, selected mode,
  expiry, content type or playlist type, range support, and one or more
  browser-safe playback URLs;
- browser-safe playback URLs carry only an opaque ticket or ticket-derived
  child token;
- direct stream, remux, HLS playlist, and HLS segment requests must validate
  the ticket before serving bytes.

Ticket validation must enforce:

- ticket expiry and optional absolute maximum lifetime;
- source id and playback mode scope;
- current Library Access and playback policy at issuance and at use;
- range request handling without bypassing validation;
- HLS playlist and segment protection;
- redaction of ticket values in logs, API errors, UI, tests, and diagnostics.

Tickets may use a sliding idle expiry or explicit refresh mechanism if long
playback sessions need renewal. That refresh must happen through an
authenticated Public Client route and must not turn the ticket into a permanent
capability URL.

The Media Web player may render a real media element only after it receives a
browser-safe ticket response. It must never place the long-lived bearer token in
`src`, HLS playlists, segment URLs, or visible UI state.

## Consequences

- Browser direct/remux playback can use native media elements while preserving
  a server-owned auth decision.
- HLS can use playlist and segment URLs that are scoped and expiring rather
  than public permanent links.
- Public Client API gains a playback ticket issuance contract.
- Ticket persistence, signing, hashing, expiry, and refresh policy become
  server-owned implementation details.
- Library Access revocation can be respected during playback if validation
  rechecks access at use or uses a sufficiently narrow ticket lifetime with
  explicit revocation handling.
- Reverse proxies and logs must treat ticket query values as secrets.
- Desktop native playback can still use bearer-authenticated SDK transport or
  native client-core transport later; it does not need to share browser ticket
  mechanics unless useful.

## Alternatives Considered

- **Use the bearer token in media URLs:** rejected because the bearer token is a
  long-lived inbound credential and would leak through browser and proxy
  surfaces.
- **Cookie/session auth for playback URLs:** useful long term, but it depends
  on credential/session UX, CSRF and same-site policy, reverse proxy behavior,
  logout semantics, and account switching. It should be designed in the
  credential/session lane, not as the first playback transport.
- **JavaScript HLS/MSE with Authorization headers:** useful for advanced HLS or
  MSE playback, but it does not solve native direct `<video src>` and carries
  browser compatibility and buffering complexity. It can be layered later.
- **Proxy all playback bytes through authenticated `fetch`:** rejected for the
  first browser player because it fights native media range behavior and makes
  large local media playback fragile.
- **Make stream URLs public behind reverse proxy auth:** rejected because Nako
  first-party clients need a server-owned access boundary and must not assume a
  reverse proxy is the only guard.

## Related Workstreams

- `docs/workstreams/media-web-client-foundation/`
- `docs/workstreams/browser-playback-auth-transport/`
- `docs/workstreams/public-client-api/`
- `docs/workstreams/playback-transcode-ops-hardening/`
