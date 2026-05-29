# HLS Selected Main Audio Cleanup

Status: Completed
Last updated: 2026-05-29

## Why This Lane Exists

HLS audio sidecar artifacts made alternate audio publication truthful: Nako now
generates audio-only HLS playlists and segments before publishing
`EXT-X-MEDIA:TYPE=AUDIO`. That first slice intentionally kept selected audio in
the main video mux for compatibility.

Keeping both paths forever makes the HLS runtime harder to reason about. The
same selected audio stream can be present in the primary video output and in an
audio sidecar group, which complicates FFmpeg map planning, adaptive variant
shape, session identity, artifact expectations, and future LL-HLS/DASH
packaging.

## Relevant Authority

- ADRs:
  - `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0048-playback-transcode-startup-degradation.md`
- Architecture maps:
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Related workstreams:
  - `docs/workstreams/hls-alternate-audio-renditions/`
  - `docs/workstreams/hls-audio-sidecar-artifacts/`
  - `docs/workstreams/hls-progressive-runtime-boundary/`
  - `docs/workstreams/playback-runtime-resource-scheduler/`

## Problem

Nako has two representations of selected HLS audio in multi-audio sidecar-capable
outputs:

- the selected source audio stream is muxed into the primary video HLS output;
- the same stream is also generated as a default audio sidecar rendition and
  advertised through `TYPE=AUDIO`.

That duplication was useful while audio sidecars were new, but it now hides the
true output contract. Future HLS/DASH work should not inherit a path where
audio-group clients receive sidecar audio while the primary video variant also
carries a selected audio track.

## Target State

When this lane closes:

- Multi-audio HLS outputs that publish generated audio groups use sidecar audio
  as the audio source for the advertised variants.
- Primary video HLS outputs avoid muxing the selected audio stream when a
  generated audio group is present and attached through `AUDIO=`.
- Single-audio, no-audio, and no-sidecar HLS outputs preserve their existing
  behavior.
- Request variant identity, artifact reconstruction, and session reuse stay
  stable and explicit after the main mux audio shape changes.
- Public master playlists keep advertising only servable audio sidecar
  artifacts.
- HLS and playback tests prove no regression in selected-audio mapping,
  adaptive ladders, browser tickets, renderer tickets, and running segment
  serving.

## Shipped Result

This lane shipped the selected-main-audio cleanup.

- `HlsArtifactManifest::main_output_has_audio()` now distinguishes source audio
  presence from whether the primary HLS output should carry audio.
- Single-variant and adaptive HLS command planning omit selected audio from the
  primary video output when generated audio sidecars are present.
- Adaptive `-var_stream_map` uses video-only variants for sidecar-capable
  outputs, while playlist authoring attaches the generated `TYPE=AUDIO` group.
- Single-audio and no-sidecar outputs keep muxed audio behavior.
- HLS request variant identity now includes `hls-main-output:v1;main_audio=false`
  for generated audio sidecar outputs so new sessions do not reuse the older
  duplicated main mux shape.

## In Scope

- HLS FFmpeg command planning for main video output audio maps.
- Server-side HLS audio rendition planning and selected/default audio behavior.
- HLS artifact identity/reconstruction changes required by the output shape.
- Focused tests for multi-audio sidecar-capable outputs, single-audio outputs,
  adaptive outputs, and no-audio outputs.
- Architecture/workstream docs and evidence.

## Out Of Scope

- Language preference policy for default audio selection.
- Codec-copy or source-codec-preserving audio sidecars.
- LL-HLS, DASH/CMAF, DRM/key delivery, or key rotation.
- Player-specific capability negotiation for clients that cannot consume HLS
  audio groups.
- Remote transcode workers or distributed packaging.
- Replacing FFmpeg CLI with libav/rsmpeg bindings.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Generated HLS audio groups are mature enough to own multi-audio playback. | Medium | `hls-audio-sidecar-artifacts` closeout and HLS gates. | Keep compatibility main audio for another slice and close with a narrower characterization task. |
| Single-audio HLS should keep muxed audio because no sidecar group is generated. | High | Existing audio sidecar lane emits sidecars only for multi-audio sources. | Add a separate single-audio sidecar design before removing muxed audio. |
| Removing duplicated main-mux audio belongs in `nako-transcode` command planning first. | High | Main video stream maps are FFmpeg command-shape behavior. | If the duplication is server-side request identity only, shrink the task to server planning. |
| Public client routes and ticket shapes should not change. | High | ADR 0052 and existing browser/renderer ticket workstreams. | Add an API/client contract task before changing route surfaces. |

## Architecture Direction

The cleanup should deepen the HLS output-shape boundary instead of adding a new
compatibility flag. `nako-server` should decide when a request has generated
audio sidecar renditions. `nako-transcode` should receive enough structured
request data to plan the main HLS mux without selected audio duplication.

Target shape:

```text
MediaProbeResult audio facts
  -> HlsAudioRendition plan
  -> HLS request variant identity
  -> FFmpeg main video output maps
  -> FFmpeg audio sidecar outputs
  -> HlsArtifactManifest
  -> TYPE=AUDIO master playlist authoring
```

The invariant is: if a master playlist attaches a generated `AUDIO=` group to a
variant, that group is the audio source for that variant. Compatibility fallback
belongs to output shapes that do not generate an audio group.

## Closeout Condition

This lane closed after:

- sidecar-capable multi-audio HLS command planning avoids selected audio
  duplication in the primary video output;
- single-audio and no-sidecar outputs preserve existing muxed-audio behavior;
- generated `TYPE=AUDIO` groups remain manifest-backed and servable;
- HLS/playback/transcode gates pass with fresh evidence;
- language policy, codec-aware sidecars, LL-HLS/DASH/DRM, and player-specific
  fallback are either split or explicitly deferred.
