# Transcode Output Shape, HLS Manifest, And Ladder Runtime - Design

Status: Active
Last updated: 2026-05-28

## Problem

The completed fMP4 single-variant runtime slice made HLS output requirements
executable, but it intentionally left three transitional assumptions in place:

- `TranscodeProfile` still represents output as `output_container: String` plus
  `hls_output: Option<HlsOutputRequirement>`, so invalid combinations are
  expressible and rejected later by validation.
- HLS runtime still treats `TranscodeSession.output_path` as the durable
  artifact root and derives playlist, segment directory, content type, and
  cleanup behavior from filesystem layout.
- Adaptive HLS is still rejected even though planner vocabulary can request it.

Those assumptions will make adaptive ladders harder to add safely because master
playlists, variant playlists, init segments, media segments, and cleanup all
need explicit identity and serving rules.

## Intent

Delete the shallow output-shape and single-artifact assumptions before adding
adaptive ladder branching. The desired end state is a typed runtime boundary
where remux, single-variant HLS, and adaptive HLS have explicit output shape,
artifact manifest, request identity, command planning, and serving semantics.

## Refactor Brief

- **Intent:** make illegal transcode output combinations unrepresentable and
  make HLS artifacts explicit before adaptive ladders multiply artifact shape.
- **Scope:** `nako-transcode` profile/request identity, FFmpeg HLS command
  planning, transcode session runtime records; `nako-server` HLS staging,
  artifact serving, playlist rewrite, and playback runtime orchestration;
  focused docs/workstream evidence.
- **Deletion plan:** remove stringly output container identity for transcode
  profiles, remove `hls_output: Option` from profile state, and remove HLS
  serving assumptions that derive all artifacts from playlist parent paths.
- **Boundary plan:** `TranscodeOutputShape` owns output identity; HLS staging
  owns `HlsArtifactManifest`; FFmpeg command planning consumes typed HLS
  layouts; server artifact serving consumes manifest-derived artifact rules.
- **Testing plan:** profile identity/validation tests, command-plan tests for
  remux, MPEG-TS, fMP4, and adaptive HLS; server tests for manifest reuse,
  playlist rewriting, content types, cleanup, and runtime session reuse.
- **Risk plan:** keep single-variant behavior green while introducing adaptive;
  preserve redaction of host paths and raw FFmpeg commands; split adaptive
  bitrate policy sophistication if the first ladder slice gets too broad.
- **Workflow plan:** one durable workstream with three medium slices. Commit
  after each verified slice when the staged diff is coherent.

## Target Flow

```text
PlaybackTargetProfile
  -> TranscodeProfile { output: TranscodeOutputShape }
  -> TranscodeRequestIdentity
  -> HlsArtifactManifest
  -> HlsRequest / FfmpegCommandBuilder::hls
  -> TranscodeSession / HlsArtifactService
```

## Output Shape Model

`TranscodeProfile` should not need validation rules like "remux must not set
HLS output" because the type should make that state impossible.

Target shape:

```text
TranscodeOutputShape
  Remux { container }
  Hls { requirement }
```

The persisted request key must continue to include all output-shape-relevant
fields, including HLS variant policy and segment container.

## HLS Artifact Manifest

The manifest should identify:

- output directory;
- primary playlist path;
- optional master playlist path for adaptive output;
- variant playlist paths;
- media segment filename patterns;
- init segment names for fMP4 variants;
- allowed artifact names and content types;
- cleanup candidates.

The server may continue to persist `TranscodeSession.output_path` as the primary
artifact path, but runtime code should consume the manifest where multiple
artifacts are involved.

## Adaptive Ladder Slice

The first executable adaptive slice should be intentionally small:

- fixed or policy-derived ladder entries sufficient to prove multi-variant
  command planning and artifact serving;
- master playlist plus variant playlists;
- per-variant segment directories or filename prefixes;
- fMP4 first if it produces a clean manifest model; MPEG-TS adaptive can remain
  a follow-on if it broadens the command surface too much.

## Non-Goals

- Do not implement a full Jellyfin device-profile model.
- Do not implement LL-HLS, DRM, CMAF encryption, subtitle renditions, or audio-
  only alternate renditions in this lane.
- Do not replace the FFmpeg CLI adapter with rsmpeg.
- Do not copy Jellyfin or FFmpeg source, schemas, tests, comments, or assets.

## Closeout Condition

This lane can close when:

- transcode output shape invalid states are removed;
- HLS runtime serving and cleanup use explicit manifest/artifact rules;
- adaptive HLS has a verified executable first slice;
- MPEG-TS and fMP4 single-variant paths remain covered;
- focused Rust gates pass and workstream evidence is recorded;
- residual ladder breadth is split into named follow-ons.
