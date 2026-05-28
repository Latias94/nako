# HLS Media Renditions Runtime TODO

Status: Active
Last updated: 2026-05-28

## Task Ledger

### HMR-010 - Open workstream and freeze first rendition scope

Status: Done
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs and index entry.
- Freeze the first-stage objective: add a typed HLS media rendition boundary and
  prefer selected subtitle WebVTT execution if it remains bounded.
- Record LL-HLS, DRM, full alternate audio UX, subtitle OCR, and second-engine
  adapter work as out of scope.

Validation:

```text
python3 -m json.tool docs/workstreams/hls-media-renditions-runtime/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-media-renditions-runtime docs/workstreams/README.md
```

### HMR-020 - Map selected audio/subtitle facts to an HLS media rendition plan

Status: Pending
Owner: codex
Depends on: HMR-010

Scope:

- Inspect current selected stream facts in playback and transcode runtime.
- Introduce a small typed HLS media rendition plan for selected subtitles and
  future alternate audio.
- Bind media rendition decisions into request identity when they affect HLS
  artifact shape.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
```

### HMR-030 - Execute the first bounded HLS rendition slice

Status: Pending
Owner: codex
Depends on: HMR-020

Scope:

- Prefer implementing selected subtitle WebVTT artifact planning and FFmpeg
  command generation.
- If subtitle extraction is too broad, land the manifest/identity foundation
  and split extraction into a follow-on with evidence.
- Keep adaptive video and no-audio stream-map behavior unchanged.

Validation:

```text
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

### HMR-040 - Server integration, reuse, redaction, and closeout

Status: Pending
Owner: codex
Depends on: HMR-030

Scope:

- Integrate the media rendition plan into HLS staging, artifact serving,
  playlist rewrite, and session reuse.
- Preserve Public/Admin redaction semantics.
- Update evidence, close the workstream, and commit verified changes.

Validation:

```text
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
