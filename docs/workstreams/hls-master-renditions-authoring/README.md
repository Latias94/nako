# HLS Master Renditions Authoring

Status: Closed
Last updated: 2026-05-29

Durable fearless refactor lane for making Nako author standard HLS master
playlist media rendition tags after selected subtitle WebVTT sidecar artifacts
became executable.

The prior `hls-media-renditions-runtime` lane made selected subtitle sidecar
playlists and `.vtt` segments deterministic, servable, and session-reusable.
This lane makes those artifacts discoverable by HLS clients through master
playlist authoring instead of relying on callers to know sidecar artifact names.

The first executable slice is selected subtitle discovery:

- emit `EXT-X-MEDIA:TYPE=SUBTITLES` for selected WebVTT sidecar playlists;
- bind video variants to the subtitle media group with `SUBTITLES=`;
- keep adaptive fMP4, single-variant MPEG-TS/fMP4, no-audio adaptive maps, and
  session reuse behavior stable.

Out of scope: full alternate audio UX, image-subtitle OCR, subtitle burn-in,
LL-HLS, CMAF encryption, DRM, and replacing the FFmpeg CLI adapter.

## Outcome

Closed after adding server-owned HLS entry playlist authoring for selected
subtitle WebVTT sidecars. Single-variant HLS now receives a generated master
entry playlist that points at the existing media playlist and subtitle group;
adaptive fMP4 master playlists are enriched with the same subtitle media group.
Playlist URI rewrite and browser/renderer ticket decoration now cover
`EXT-X-MEDIA:URI` attributes.
