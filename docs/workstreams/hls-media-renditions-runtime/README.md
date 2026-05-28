# HLS Media Renditions Runtime

Status: Active
Last updated: 2026-05-28

Durable fearless refactor lane for making Nako's HLS runtime capable of
describing media renditions beyond video variants.

The adaptive HLS runtime now has source-aware video ladders and can handle
video-only sources. The next maturity gap is that HLS artifacts still model a
playlist plus media segments, but not the separate media rendition vocabulary
needed for selected subtitles, alternate audio, and future richer master
playlists.

This lane starts with a first executable rendition slice. The preferred slice is
selected subtitle delivery as HLS WebVTT artifacts when the existing playback
request selects a subtitle stream. If implementation evidence shows selected
subtitles need a larger extraction pipeline first, the lane will close a
smaller alternate-audio/subtitle manifest foundation and split extraction into a
follow-on.

Out of scope: LL-HLS, CMAF encryption, DRM, full multi-audio selection UI,
subtitle OCR, provider subtitle search, and a second transcode engine adapter.
