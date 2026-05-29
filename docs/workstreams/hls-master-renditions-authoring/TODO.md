# HLS Master Renditions Authoring TODO

Status: Active
Last updated: 2026-05-29

## Task Ledger

### HMA-010 - Open workstream and freeze master playlist scope

Status: Done
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs and index entry.
- Freeze the first executable slice: selected subtitle WebVTT sidecar discovery
  through standard HLS master playlist media tags.
- Record full alternate audio, image subtitle OCR/burn-in, LL-HLS, DRM, and
  second-engine adapter work as out of scope.

Validation:

```text
python3 -m json.tool docs/workstreams/hls-master-renditions-authoring/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-master-renditions-authoring docs/workstreams/README.md
```

### HMA-020 - Add typed HLS master playlist authoring boundary

Status: Pending
Owner: codex
Depends on: HMA-010

Scope:

- Introduce a small authoring boundary that consumes `HlsArtifactManifest`.
- Express subtitle media groups and stream variant attachments without ad hoc
  string patching.
- Preserve existing FFmpeg media playlist output as raw artifacts.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
```

### HMA-030 - Make selected subtitle sidecars discoverable

Status: Pending
Owner: codex
Depends on: HMA-020

Scope:

- Emit `EXT-X-MEDIA:TYPE=SUBTITLES` for selected WebVTT sidecar playlists.
- Attach subtitle groups to single-variant and adaptive HLS video entries.
- Ensure rewritten playlist URLs and playback tickets still work.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### HMA-040 - Verify reuse/redaction and close workstream

Status: Pending
Owner: codex
Depends on: HMA-030

Scope:

- Verify session reuse reconstructs authored master playlist behavior.
- Preserve adaptive source-aware ladder and no-audio adaptive stream-map
  behavior.
- Record evidence, close the workstream, and commit verified changes.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
