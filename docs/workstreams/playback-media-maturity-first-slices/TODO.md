# Playback Media Maturity First Slices TODO

Status: Completed
Last updated: 2026-05-28

## Task Ledger

### PMMFS-010 - Open workstream and freeze first-slice scope

Status: Complete
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs and task ledger.
- Link the lane from the workstream index.
- Record validation gates and non-goals before implementation starts.

Validation:

```text
python3 -m json.tool docs/workstreams/playback-media-maturity-first-slices/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-media-maturity-first-slices docs/workstreams/README.md
```

### PMMFS-020 - Add richer Public Client capability profile input

Status: Complete
Owner: codex
Depends on: PMMFS-010

Scope:

- Extend `ClientPlaybackCapabilities` with direct-play limits and HLS planning
  preferences while preserving default behavior.
- Map browser playback request DTOs, query parameters, renderer registration,
  and Public Client DTOs into the richer domain model.
- Update OpenAPI schemas and generated SDK contract checks where required.

Validation:

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-client-protocol --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server renderer --no-fail-fast
```

### PMMFS-030 - Add adaptive HLS and fMP4 planning vocabulary

Status: Complete
Owner: codex
Depends on: PMMFS-020

Scope:

- Add HLS variant policy and segment container records to transcode
  requirements or profiles.
- Carry single/adaptive and MPEG-TS/fMP4 intent through playback decisions.
- Keep current executable runtime output unchanged unless the lane explicitly
  adds a verified runtime gate.

Validation:

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### PMMFS-040 - Mature subtitle, HDR, audio, bitrate, and resolution reasons

Status: Complete
Owner: codex
Depends on: PMMFS-020

Scope:

- Use richer capability limits when evaluating direct play.
- Emit explicit compatibility reasons for unsupported subtitle delivery, HDR,
  audio channel count, bitrate, and resolution constraints.
- Keep Public Client decision reports redaction-safe.

Validation:

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### PMMFS-050 - Verify, document evidence, and close first slice

Status: Complete
Owner: codex
Depends on: PMMFS-030, PMMFS-040

Scope:

- Run fresh focused gates.
- Update evidence, handoff, milestones, and closeout status.
- Split adaptive runtime, fMP4 execution, DLNA profiles, or rsmpeg work into
  follow-ons if they remain outside this first slice.

Validation:

```text
python3 -m json.tool docs/workstreams/playback-media-maturity-first-slices/WORKSTREAM.json
cargo fmt --all -- --check
git diff --check
```
