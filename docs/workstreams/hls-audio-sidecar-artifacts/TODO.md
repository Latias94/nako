# HLS Audio Sidecar Artifacts TODO

Status: Active
Last updated: 2026-05-29

## Task Ledger

### HAS-010 - Open workstream and freeze audio sidecar scope

Status: Done
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs and index entry.
- Freeze the first executable slice to generated AAC/ADTS audio sidecars for
  multi-audio sources.
- Preserve selected-audio main mux behavior.

Validation:

```text
python3 -m json.tool docs/workstreams/hls-audio-sidecar-artifacts/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-audio-sidecar-artifacts docs/workstreams/README.md
```

### HAS-020 - Add typed audio rendition artifacts

Status: Pending
Owner: codex
Depends on: HAS-010

Scope:

- Add `HlsAudioRendition` to `HlsMediaRenditionPlan`.
- Include audio rendition identity round trip and validation.
- Add audio playlist/segment membership to `HlsArtifactManifest`.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
```

### HAS-030 - Generate and publish audio sidecars

Status: Pending
Owner: codex
Depends on: HAS-020

Scope:

- Add FFmpeg audio-only sidecar HLS args.
- Build audio rendition plans from multi-audio probe facts.
- Serve audio sidecar artifacts through existing segment routes.
- Author `TYPE=AUDIO` master playlist tags and `AUDIO=` stream attributes.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### HAS-040 - Verify and close

Status: Pending
Owner: codex
Depends on: HAS-030

Scope:

- Record gate evidence.
- Close workstream docs.
- Commit verified changes.

Validation:

```text
python3 -m json.tool docs/workstreams/hls-audio-sidecar-artifacts/WORKSTREAM.json
cargo fmt --all -- --check
git diff --check
```
