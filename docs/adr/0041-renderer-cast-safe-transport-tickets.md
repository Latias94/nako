# 0041: Separate Renderer Cast-Safe Transport Tickets From Browser Tickets

## Status

Accepted.

## Context

Nako now has browser playback tickets, playback policy-aware renderer targets,
Renderer Sessions, typed Renderer Commands, and direct-play Nako-to-Nako
casting. The remaining gap is renderer media transport for targets that cannot
or should not fetch media bytes with the same bearer token used for Public
Client control requests.

Browser playback tickets solve a different problem:

- a browser media element cannot attach an `Authorization` header;
- the media request belongs to the interactive browser player;
- the ticket is scoped to a source and playback mode, but not to a Renderer
  Session or remote-control command.

Renderer transport has stricter and different pressure:

- the control plane is the bearer-authenticated Public Client renderer polling
  API;
- the media plane may be consumed by a native media element, a browser media
  element inside a Nako remote client, Chromecast, DLNA, AirPlay, or another
  adapter;
- non-direct remux/HLS playback needs an expiring URL without turning
  Transcode Session IDs into public credentials;
- Admin diagnostics and renderer listing must stay redaction-safe.

Jellyfin-class casting also shows that server sessions, playback sessions,
transcode artifacts, remote-control sessions, and cast receiver URLs are
related but separate concepts. Nako should preserve that separation before
adding Chromecast, DLNA, or AirPlay.

## Decision

Nako will introduce a separate **Renderer Cast-Safe Transport Ticket**.

This ticket is:

- an opaque server-owned media transport credential;
- scoped to a Renderer Session, Playback Session, Media Source, playback mode,
  network scope, owner/controller principal facts, and expiry;
- validated on every direct/remux/HLS media request that uses it;
- redacted from Admin diagnostics, Public renderer listings, logs, errors, and
  generated SDK tests;
- not a bearer token, not a browser playback ticket, not a Source Locator, and
  not a Transcode Session ID.

The control plane remains bearer-authenticated:

```text
renderer registers/heartbeats/polls commands with bearer auth
controller sends play command with bearer auth
server creates policy-checked Playback Session
server creates target-safe media transport
renderer receives a typed command envelope
renderer fetches media through renderer-scoped ticket URLs
```

The renderer record's `transport_auth` describes the media transport needed by
the target, not the authentication required to call the Public Client control
routes. Nako remote clients may therefore use bearer auth for command polling
while receiving cast-safe ticket URLs for media bytes.

Renderer play command responses and command polling may expose a safe transport
envelope with fields such as playback mode, content type or playlist type,
range support, expiry, and URL(s). They must not expose raw `payload_json`,
source locators, local paths, bearer tokens, transcode session identifiers as
credentials, or unscoped permanent stream URLs.

HLS playlists and segments served through renderer transport must preserve the
same ticket boundary. A playlist may contain segment URLs only if each segment
request validates the renderer ticket or a derived child token with equivalent
scope.

## Consequences

- Nako remote-client remux/HLS casting can be implemented before Chromecast,
  DLNA, or AirPlay while exercising the same transport primitive those
  adapters need.
- Browser tickets remain browser-player credentials and do not grow hidden
  renderer semantics.
- Renderer command DTOs need a safe transport envelope rather than raw command
  payload exposure.
- Playback Session, Transcode Session, Renderer Session, and transport ticket
  lifetimes can be tested independently.
- Future adapter readiness can depend on the existence of cast-safe transport
  without copying transport policy into protocol-specific adapters.
- Reverse proxy and network-scope policy must be part of ticket validation
  before remote or external protocol transport is enabled.

## Alternatives Considered

- **Reuse browser playback tickets for renderers:** rejected because browser
  tickets do not bind Renderer Session, Playback Session, control command, or
  network scope.
- **Use bearer-authenticated media URLs for every renderer:** rejected because
  many renderers and protocol receivers cannot attach bearer headers to media
  byte requests.
- **Use Transcode Session IDs as URL credentials:** rejected because transcode
  sessions are internal runtime artifacts and must not become public transport
  identities.
- **Let each casting adapter mint its own media URLs:** rejected because source
  access, playback policy, expiry, and redaction must remain host-owned and
  consistent across adapters.

## Related Workstreams

- `docs/workstreams/nako-renderer-cast-safe-transport/`
- `docs/workstreams/casting-renderer-runtime/`
- `docs/workstreams/browser-playback-auth-transport/`
- `docs/workstreams/playback-policy-and-renderer-targets/`
- `docs/workstreams/network-access-boundary/`
