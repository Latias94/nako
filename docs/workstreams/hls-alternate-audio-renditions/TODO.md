# HLS Alternate Audio Renditions TODO

Status: Active
Last updated: 2026-05-29

## Task Ledger

### HAA-010 - Open workstream and freeze audio correctness scope

Status: Done
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs and index entry.
- Record the selected-audio correctness bug that blocks truthful alternate
  audio rendition authoring.
- Freeze true `TYPE=AUDIO` sidecar publication as a follow-on unless artifacts
  can be generated and served in this lane.

Validation:

```text
python3 -m json.tool docs/workstreams/hls-alternate-audio-renditions/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-alternate-audio-renditions docs/workstreams/README.md
```

### HAA-020 - Make selected HLS audio stream mapping executable

Status: Pending
Owner: codex
Depends on: HAA-010

Scope:

- Replace fixed HLS `0:a:0?` stream mapping with a mapping derived from
  `TranscodeTrackSelection.audio_stream`.
- Preserve optional first-audio behavior when no stream is explicitly selected.
- Apply the same selected-audio mapping to adaptive HLS renditions.
- Preserve no-audio adaptive behavior.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

### HAA-030 - Decide and document the alternate audio artifact boundary

Status: Pending
Owner: codex
Depends on: HAA-020

Scope:

- Decide whether this lane continues into audio-only sidecar HLS artifact
  generation or closes after selected-audio correctness.
- If continuing, define `HlsAudioRendition` identity, artifacts, FFmpeg command
  planning, and `TYPE=AUDIO` master playlist authoring tasks.
- If closing, record follow-on scope and required gates.

Validation:

```text
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
