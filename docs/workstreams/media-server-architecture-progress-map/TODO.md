# Media Server Architecture Progress Map TODO

Status: Closed
Last updated: 2026-05-29

## Task Ledger

### MSAP-010 - Freeze architecture map scope

Status: Done
Owner: codex
Depends on: none

Scope:

- Identify existing glossary, ADR, roadmap, and workstream authority.
- Keep this lane documentation-only.

Validation:

```text
docs/workstreams/media-server-architecture-progress-map/DESIGN.md
```

### MSAP-020 - Add top-level architecture map

Status: Done
Owner: codex
Depends on: MSAP-010

Scope:

- Add `docs/ARCHITECTURE.md`.
- Add `docs/architecture/PLAYBACK.md` as the playback-specific agent map.
- Add storage/VFS, library pipeline, state/access, realtime/sync, and
  operations/release deep dives.
- Add the control-plane deep dive for addon lifecycle, observability, durable
  jobs, remote access, API scale, and cache contracts.
- Summarize current system areas, maturity, and next pressure points.
- Make playback/transcode progress navigable after recent HLS work.

Validation:

```text
git diff --check -- docs/ARCHITECTURE.md
```

### MSAP-030 - Record HLS/media-engine boundary

Status: Done
Owner: codex
Depends on: MSAP-020

Scope:

- Add ADR 0052 for FFmpeg CLI-first HLS runtime and manifest-backed artifacts.
- Update the ADR index.

Validation:

```text
git diff --check -- docs/adr/0052-hls-runtime-and-media-engine-boundary.md docs/adr/README.md
```

### MSAP-040 - Update planning indexes and close

Status: Done
Owner: codex
Depends on: MSAP-030

Scope:

- Update docs index, roadmap, and workstream index.
- Record evidence and close the lane.

Validation:

```text
python3 -m json.tool docs/workstreams/media-server-architecture-progress-map/WORKSTREAM.json
git diff --check
```

### MSAP-050 - Add control-plane supplement

Status: Done
Owner: codex
Depends on: MSAP-040

Scope:

- Add `docs/architecture/CONTROL_PLANE.md`.
- Add ADR 0053 for the application control-plane boundary.
- Update architecture, ADR, and workstream indexes.

Validation:

```text
python3 -m json.tool docs/workstreams/media-server-architecture-progress-map/WORKSTREAM.json
git diff --check
```

### MSAP-060 - Organize architecture and workstream links

Status: Done
Owner: codex
Depends on: MSAP-050

Scope:

- Keep `docs/ARCHITECTURE.md` concise by moving detailed execution references
  into architecture deep dives.
- Add `docs/architecture/WORKSTREAM_LINKS.md`.
- Document the `architecture_refs` and `capability_tags` convention for future
  workstreams.

Validation:

```text
python3 -m json.tool docs/workstreams/media-server-architecture-progress-map/WORKSTREAM.json
git diff --check
```
