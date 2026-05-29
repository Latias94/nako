# HLS Media Renditions Runtime

Status: Closed
Last updated: 2026-05-29

Closed fearless refactor lane for making Nako's HLS runtime capable of
describing and serving the first media rendition artifacts beyond video
variants.

The adaptive HLS runtime now has source-aware video ladders and can handle
video-only sources. The next maturity gap is that HLS artifacts still model a
playlist plus media segments, but not the separate media rendition vocabulary
needed for selected subtitles, alternate audio, and future richer master
playlists.

This lane shipped the first executable rendition slice: selected subtitle
delivery as HLS WebVTT sidecar playlist and segment artifacts when the playback
request selects a subtitle stream. The runtime now carries media rendition
decisions through typed request-variant identity so persisted HLS sessions can
reconstruct and serve the same subtitle artifacts.

Out of scope: LL-HLS, CMAF encryption, DRM, full multi-audio selection UI,
subtitle OCR, provider subtitle search, rich master playlist alternate-audio UX,
and a second transcode engine adapter.
