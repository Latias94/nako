# Phase 4.6: Remux Playback Route

## Goal

Expose remuxed playback through HTTP while keeping handlers thin. The route
should call the Phase 4.5 remux application service instead of touching FFmpeg
plans, runners, staging paths, or duplicate-request state directly.

## Proposed Shape

- Add an HTTP route for source remux playback.
- Translate client playback capabilities from query parameters or headers.
- Return a clear pending/conflict/error response when an equivalent remux is
  already in flight.
- Stream a completed staged remux output with byte-range support when possible.
- Keep direct play behavior unchanged.
- Document the route in `docs/api/HTTP_API.md`.

## Non-Goals

- No HLS playlist or segment route.
- No persisted transcode session table.
- No remote-source staging.
- No hardware acceleration policy.

## Validation

Expected coverage:

- route returns remux output for a source with remux playback decision;
- route reuses completed staged output;
- in-flight duplicate remux maps to `409 conflict`;
- handler stays limited to request parsing, service call, and response
  translation;
- direct play route behavior remains unchanged.
