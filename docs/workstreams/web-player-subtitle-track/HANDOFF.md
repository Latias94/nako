# Web Player Subtitle Track Handoff

## Status

Complete.

## Completed

- Public media data source can build a playback plan from browser playback
  tickets and source probe sidecar subtitle facts.
- Media player route asks for the playback plan and passes live media/subtitle
  URLs to `VideoPlayer`.
- `VideoPlayer` renders native `<video>`, `<source>`, and `<track>` elements
  when live ticket URLs exist.

## Follow-Ons

- Full WMLP browser playback entry with playback sessions and heartbeat.
- Route-owned live detail/source selection once WMLP-030 lands.
- HLS subtitle renditions for clients that need playlist-level subtitles.
