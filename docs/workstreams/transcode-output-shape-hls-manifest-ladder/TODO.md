# Transcode Output Shape, HLS Manifest, And Ladder Runtime TODO

Status: Active
Last updated: 2026-05-28

## Task Ledger

### TOSHL-010 - Open workstream and freeze staged scope

Status: Done
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs and index entry.
- Freeze the sequence: typed output shape, HLS artifact manifest, adaptive
  ladder runtime.
- Record that adaptive breadth outside the first executable slice must split.

Validation:

```text
python3 -m json.tool docs/workstreams/transcode-output-shape-hls-manifest-ladder/WORKSTREAM.json
git diff --check -- docs/workstreams/transcode-output-shape-hls-manifest-ladder docs/workstreams/README.md
```

### TOSHL-020 - Replace transitional transcode output profile shape

Status: Done
Owner: codex
Depends on: TOSHL-010

Scope:

- Introduce typed `TranscodeOutputShape`.
- Remove `output_container: String` and `hls_output: Option` from
  `TranscodeProfile`.
- Preserve remux and HLS request identity semantics.
- Delete validation reasons that only existed because invalid shape states were
  expressible.

Validation:

```text
cargo nextest run -p nako-transcode profile --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
```

Notes:

- Added typed `TranscodeOutputShape` and removed `TranscodeProfile.kind`,
  `TranscodeProfile.output_container`, and `TranscodeProfile.hls_output`.
- Deleted validation reasons that only existed for now-unrepresentable
  remux/HLS shape combinations.
- Kept persisted request identity keys stable for existing remux and
  single-variant HLS shapes.

### TOSHL-030 - Introduce HLS artifact manifest runtime boundary

Status: Done
Owner: codex
Depends on: TOSHL-020

Scope:

- Add explicit HLS artifact manifest/layout records for playlist, init segment,
  segment patterns, content types, and cleanup candidates.
- Make server HLS artifact serving consume manifest-derived artifact rules
  rather than deriving behavior only from `output_path.parent()`.
- Preserve existing MPEG-TS and fMP4 single-variant behavior.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-transcode hls --no-fail-fast
```

Notes:

- Added `HlsArtifactManifest` and `TranscodeArtifactSet` to the transcode
  runtime boundary.
- Made `HlsRequest` consume a manifest instead of loose output dir, playlist,
  segment pattern, and output fields.
- Made server HLS artifact serving resolve playlist, init segment, media
  segments, content type, and cleanup candidates through manifest-derived rules.
- Preserved MPEG-TS and fMP4 single-variant runtime behavior and completed
  session reuse coverage.

### TOSHL-040 - Implement adaptive HLS ladder first runtime slice

Status: Pending
Owner: codex
Depends on: TOSHL-030

Scope:

- Model a typed HLS rendition ladder in transcode request/profile identity.
- Add FFmpeg HLS command planning for master playlist plus variant playlists.
- Add server staging, artifact serving, playlist rewrite, and reuse coverage for
  the first adaptive ladder shape.
- Keep advanced bitrate policy, subtitle renditions, alternate audio, LL-HLS,
  and DRM as follow-ons unless required to prove the first slice.

Validation:

```text
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### TOSHL-050 - Verify, close, and commit

Status: Pending
Owner: codex
Depends on: TOSHL-040

Scope:

- Run fresh focused gates and closeout checks.
- Update evidence, handoff, milestones, and workstream status.
- Commit verified coherent slices autonomously with Conventional Commit
  messages.

Validation:

```text
cargo fmt --all -- --check
git diff --check
```
