# Adaptive HLS Source-Aware Ladder Runtime TODO

Status: Active
Last updated: 2026-05-28

## Task Ledger

### AHSL-010 - Open workstream and freeze source-aware adaptive scope

Status: Done
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs and index entry.
- Freeze the first two adaptive-runtime deepening targets: source-aware ladder
  policy and no-audio stream-map planning.
- Record that adaptive MPEG-TS, alternate audio, subtitle renditions, LL-HLS,
  CMAF, DRM, and second-engine adapter work remain separate lanes.

Validation:

```text
python3 -m json.tool docs/workstreams/adaptive-hls-source-aware-ladder/WORKSTREAM.json
git diff --check -- docs/workstreams/adaptive-hls-source-aware-ladder docs/workstreams/README.md
```

### AHSL-020 - Add source-aware adaptive ladder policy and identity

Status: Pending
Owner: codex
Depends on: AHSL-010

Scope:

- Introduce a typed adaptive ladder plan derived from source video facts and
  client output constraints.
- Avoid upscaling and cap variant bitrates by source/client facts.
- Make adaptive request/session identity include the ladder decision or its
  versioned typed policy inputs.
- Keep behavior deterministic when probe facts are incomplete.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-transcode transcode_profile --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
```

### AHSL-030 - Make adaptive FFmpeg planning audio-presence aware

Status: Pending
Owner: codex
Depends on: AHSL-020

Scope:

- Carry selected-source audio presence into adaptive HLS command planning.
- Preserve current audio-bearing adaptive command behavior.
- Emit video-only maps and `var_stream_map` entries for no-audio sources.
- Cover both source shapes with focused command-plan tests.

Validation:

```text
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
```

### AHSL-040 - Integrate source-aware plans in server HLS runtime

Status: Pending
Owner: codex
Depends on: AHSL-030

Scope:

- Use the source-aware ladder plan for adaptive staging layout.
- Reconstruct session artifact manifests from the same identity/plan boundary.
- Preserve adaptive playlist rewrite, artifact allow-listing, cleanup
  candidates, and runtime reuse semantics.
- Keep Public/Admin redaction behavior covered.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### AHSL-050 - Verify, close, and commit

Status: Pending
Owner: codex
Depends on: AHSL-040

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
