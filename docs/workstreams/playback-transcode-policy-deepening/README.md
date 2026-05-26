# Playback Transcode Policy Deepening

Status: Active
Last updated: 2026-05-27

This workstream owns the architecture-first refactor that makes Nako's playback
and server-side transcode policy mature enough for Jellyfin-class local media
features while preserving Nako's own domain model.

The lane is driven by product capabilities, not by adding a media library for
its own sake. Nako should use mature media engines for heavy media work, but
own the playback control plane: access, planning, capability negotiation,
runtime policy, transcode artifacts, diagnostics, cancellation, cleanup, and
safe client contracts.

## Goals

- Introduce a Playback Planner Module that decides direct play, remux, HLS
  transcode, and future optimized/remote playback from explicit facts.
- Introduce a Transcode Policy Module that models decode/filter/encode/subtitle
  stages, hardware fallback, bitrate, and output constraints without a shallow
  `hardware_acceleration` boolean.
- Make FFmpeg CLI the first Transcode Engine Adapter behind a typed interface.
- Add a Playback Runtime Inventory for redaction-safe FFmpeg/hardware capability
  evidence.
- Align Public Client playback decisions and Admin diagnostics with the same
  planner/policy facts while keeping their DTOs separate.
- Keep the architecture ready for desktop-native players, browser playback
  tickets, adaptive HLS, optimized versions, remote transcode workers, and
  scoped user transcode permissions.

## Non-Goals

- Build a frontend player UI.
- Implement a Tauri desktop player or native mobile player.
- Replace FFmpeg with an embedded media stack.
- Copy Jellyfin code, schemas, comments, tests, or assets from `repo-ref/`.
- Implement recommendations, SyncPlay, DLNA, live TV, or offline sync in this
  lane.
- Add a new crate before reuse pressure proves that a crate boundary is deeper
  than server app Modules.

## Reference Research

Reference repositories are used for behavior and architecture pressure only:

- `repo-ref/jellyfin/MediaBrowser.Model/Dlna/DeviceProfile.cs`: mature device
  capability shape for direct play, codec/container constraints, subtitles, and
  transcoding profiles.
- `repo-ref/jellyfin/MediaBrowser.Model/Dlna/StreamInfo.cs`: playback-info
  shape for selected method, streams, output codecs, HLS/progressive settings,
  and transcode reasons.
- `repo-ref/jellyfin/MediaBrowser.Model/Session/TranscodeReason.cs`: explicit
  user-facing reasons for transcoding rather than opaque engine failures.
- `repo-ref/jellyfin/MediaBrowser.Model/Configuration/EncodingOptions.cs`:
  operator settings pressure for hardware acceleration, tonemapping, segment
  deletion, throttling, subtitles, and per-codec hardware decode.
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Transcoding/TranscodeManager.cs`
  and `MediaBrowser.Controller/MediaEncoding/ITranscodeManager.cs`: job
  lifecycle pressure around play-session pings, progress, cancellation,
  throttling, and output cleanup.
- `repo-ref/oximedia`: media pipeline and hardware acceleration layering
  reference, not a dependency target.
- `repo-ref/libmedia`: web/client capability reference, not a server
  dependency.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

