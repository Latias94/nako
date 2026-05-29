# HLS Master Renditions Authoring - Design

Status: Closed
Last updated: 2026-05-29

## Problem

Nako can now generate selected subtitle WebVTT sidecar playlists and segments,
but the primary HLS playlist does not advertise those media renditions through
standard HLS master playlist tags. That leaves the runtime with servable
subtitle artifacts that clients cannot discover from the normal HLS entry
point.

The same gap will block alternate audio: the artifact boundary can know about a
media rendition, but playback clients need the master playlist to declare media
groups and attach them to stream variants.

## Intent

Move HLS media rendition discoverability into a typed authoring boundary. The
server should not patch one-off strings for every future media kind; it should
derive master playlist media tags from the same `HlsArtifactManifest` and
request-variant plan used for artifact serving and session reuse.

## Refactor Brief

- **Intent:** remove the assumption that FFmpeg-produced playlists are the final
  public HLS playlist when Nako-owned media rendition metadata must be declared.
- **Scope:** `nako-transcode` HLS manifest/rendition vocabulary,
  `nako-server` HLS playlist reading/rewrite boundaries, focused playback route
  tests, and workstream evidence.
- **Deletion plan:** delete or avoid ad hoc subtitle URI injection helpers once
  a typed HLS master playlist authoring helper owns `EXT-X-MEDIA` and
  `EXT-X-STREAM-INF` enrichment.
- **Boundary plan:** keep media artifact planning in `nako-transcode`; make
  playlist authoring a server playback boundary that consumes the manifest and
  emits a public HLS master entry point; keep raw segment and sidecar playlist
  serving unchanged.
- **Testing plan:** unit-test master playlist authoring for single/adaptive,
  subtitle/no-subtitle, and no-audio adaptive paths; integration-test HLS source
  selected subtitle discovery and reuse through public playlist output.
- **Risk plan:** do not regress existing HLS media playlist semantics; if a
  single-variant media playlist cannot legally carry subtitles, introduce a
  generated master playlist entry point while preserving the current media
  playlist as a variant artifact.
- **Workflow plan:** one durable workstream with selected subtitle master
  playlist authoring as the first executable slice, then close or split
  alternate audio into a follow-on.

## Target Flow

```text
HlsArtifactManifest + HlsMediaRenditionPlan
  -> HlsMasterPlaylistAuthor
  -> public HLS master playlist body
  -> playlist URI rewrite / playback ticket decoration
  -> sidecar subtitle playlist and VTT segment routes
```

## First Slice

Selected subtitle sidecar artifacts should become visible from the public HLS
entry playlist:

- single-variant HLS should expose a generated master playlist that points at
  the existing media playlist and selected subtitle playlist;
- adaptive fMP4 HLS should enrich the master playlist with subtitle media tags
  and attach the subtitle group to each stream variant;
- subtitle sidecar playlist routes should remain relative and servable through
  existing segment artifact routing.

## Non-Goals

- Do not implement full alternate audio in this lane unless the selected
  subtitle authoring boundary makes it trivial and bounded.
- Do not implement image subtitle OCR or burn-in.
- Do not implement LL-HLS, CMAF encryption, or DRM.
- Do not replace FFmpeg CLI execution with rsmpeg or libav bindings.
- Do not copy Jellyfin, FFmpeg, or rsmpeg source, schemas, tests, comments, or
  assets.

## Closeout Condition

This lane can close when:

- selected subtitle WebVTT sidecars are advertised through standard HLS master
  playlist tags;
- single-variant and adaptive HLS entry playlists remain deterministic;
- session reuse and artifact reconstruction preserve the same authored master
  playlist behavior after restart;
- Public/Admin redaction gates remain green;
- focused Rust gates pass and evidence is recorded.
