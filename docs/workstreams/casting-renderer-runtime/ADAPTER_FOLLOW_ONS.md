# Casting Renderer Runtime - Adapter Follow-Ons

Status: Active
Last updated: 2026-05-27

## Purpose

CAST-060 keeps external casting protocols out of the Nako remote-client adapter
while making their future contracts explicit. The shared host boundary is:

```text
Renderer Adapter
  -> discovers or registers a Renderer Session
  -> reports redaction-safe readiness
  -> accepts typed Renderer Commands
  -> asks Playback App Service for a policy-checked Playback Session
  -> obtains target-safe media transport
```

The adapter must never expose raw Source Locators, local paths, bearer tokens,
or Transcode Session IDs as public transport identities.

## Shared Adapter Contract

Every protocol adapter needs these host-facing capabilities:

- `inspect_readiness`: report protocol availability, configuration gaps, and
  safe operator reason codes.
- `list_targets`: create or refresh Renderer Sessions with target kind,
  network scope, transport auth, media capabilities, and supported commands.
- `send_command`: translate typed Renderer Commands into protocol messages.
- `bind_playback`: request a policy-checked Playback Session and attach it to
  the Renderer Session.
- `resolve_transport`: obtain direct, remux, or HLS transport that is valid for
  this renderer and does not leak host-private paths or credentials.

## Nako Remote Client Cast-Safe Transport

Current state:

- `nako_remote_client` with bearer auth is implemented.
- Direct-play command flow is implemented.
- Non-direct remux/HLS transport is deliberately deferred.

Required follow-on:

- Issue session-bound cast-safe URLs when a Nako renderer cannot use bearer
  auth for media fetches.
- Support remux/HLS renderer playback without inventing placeholder Playback
  Sessions.
- Bind ticket expiry to Playback Session, Renderer Session, selected Source,
  and network scope.
- Keep ticket material out of Admin diagnostics and Public list responses.

## Chromecast Adapter

Adapter contract:

- Discovery: local-network discovery through a Chromecast adapter process or
  host runtime component.
- Control plane: adapter process sends receiver commands.
- Transport: cast-safe HTTPS URL; bearer headers cannot be assumed.
- Configuration: receiver application id, external base URL, CORS/HTTPS
  readiness, and reverse-proxy endpoint selection.
- Commands: play, pause, resume, seek, stop; volume only if target reports
  support.

Readiness checks:

- receiver app configured;
- external endpoint configured when the renderer cannot reach local server URL;
- cast-safe transport enabled;
- HTTPS/CORS requirements satisfied for the selected receiver path;
- local discovery available or manually configured target is valid.

## DLNA Renderer Adapter

Adapter contract:

- Discovery: SSDP/UPnP discovery owned by the adapter.
- Control plane: AVTransport/RenderingControl messages through adapter process.
- Transport: URL fetch from renderer, usually without bearer headers.
- Configuration: trusted LAN scope, optional allowlist, advertised base URL.
- Commands: play, pause, seek, stop; volume only through RenderingControl when
  supported.

Readiness checks:

- local-network discovery enabled;
- URL exposure is restricted to trusted network scope;
- cast-safe URL/ticket lifecycle is available;
- renderer capability profile is known or safely probed.

## AirPlay Adapter

Adapter contract:

- Discovery: platform or protocol-specific discovery owned by adapter.
- Control plane: adapter process translates typed commands.
- Transport: protocol-specific media stream or cast-safe URL depending on
  implementation.
- Configuration: pairing/auth state when required, advertised endpoint, codec
  support profile.
- Commands: play, pause, resume, seek, stop; volume only if negotiated.

Readiness checks:

- discovery backend available on the host OS;
- pairing/auth configuration is complete when required;
- media compatibility is known;
- network exposure and ticketing match the selected target.

## Split Order

1. Nako remote-client non-direct transport.
2. Chromecast adapter.
3. DLNA renderer adapter.
4. AirPlay adapter.

This order keeps the shared transport/ticket primitive honest before adding
protocol discovery and receiver-specific behavior.
