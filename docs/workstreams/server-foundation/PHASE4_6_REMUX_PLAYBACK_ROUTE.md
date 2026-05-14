# Phase 4.6: Remux Playback Route

## Goal

Expose remuxed playback through HTTP while keeping handlers thin. The route
should call the Phase 4.5 remux application service instead of touching FFmpeg
plans, runners, staging paths, or duplicate-request state directly.

## Implemented Shape

- Added `GET /sources/{source_id}/stream/remux`.
- Translates client playback capabilities from query parameters.
- Accepts `output_container=mp4|mkv`; the default is `mp4`.
- Calls the Phase 4.5 remux application service.
- Streams completed staged remux outputs with byte-range support.
- Returns `409 conflict` for equivalent in-flight remux requests.
- Keeps direct play behavior unchanged.
- Documents the route in `docs/api/HTTP_API.md`.

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
- direct play route behavior remains unchanged.

Validation used for this phase:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
git diff --check
```
