# HLS Progressive Runtime Boundary — TODO

Status: Completed
Last updated: 2026-05-29

## Task Ledger

### HPRB-010 — Open workstream and freeze runtime problem

Status: Completed
Owner: planner
Depends on: none

Scope:

- Create durable workstream docs.
- Link the lane from playback architecture indexes.
- Freeze the first executable proof target and non-goals.

Validation:

```text
python3 -m json.tool docs/workstreams/hls-progressive-runtime-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-progressive-runtime-boundary docs/workstreams/README.md docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md
```

Evidence:

- `docs/workstreams/hls-progressive-runtime-boundary/DESIGN.md`
- `docs/workstreams/hls-progressive-runtime-boundary/EVIDENCE_AND_GATES.md`

Handoff:

- Planner owns this before implementation workers start.

### HPRB-020 — Make HLS output publication policy explicit

Status: Completed
Owner: codex
Depends on: HPRB-010

Scope:

- `crates/nako-transcode/src/hls.rs`
- `crates/nako-transcode/src/runner_util.rs`
- `crates/nako-transcode/src/lib.rs`

Goal:

- Replace the implicit temporary-directory-only HLS publication behavior with a
  typed runtime publication policy.
- Preserve the current atomic VOD behavior behind explicit tests while adding
  the smallest serve-visible running-output proof needed by later tasks.

Validation:

```text
cargo nextest run -p nako-transcode hls_runner --no-fail-fast
cargo nextest run -p nako-transcode hls --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-transcode/src/lib.rs`
- `crates/nako-transcode/src/hls.rs`
- `cargo nextest run -p nako-transcode hls_runner --no-fail-fast`
- `cargo nextest run -p nako-transcode hls --no-fail-fast`
- `docs/workstreams/hls-progressive-runtime-boundary/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: HLS output publication is now explicit in `nako-transcode`.
- The default runner path remains atomic VOD promotion.
- `ServeWhileRunning` writes to the final session directory so later server work
  can observe playlist and segment artifacts before process exit.
- Cancel and failure paths clean the selected publication directory.

### HPRB-030 — Return HLS playlists from a running session after readiness

Status: Completed
Owner: codex
Depends on: HPRB-020

Scope:

- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/tests/playback.rs`
- `crates/nako-server/src/http/tests/playback.rs`

Goal:

- Start or reuse an HLS transcode session without waiting for the full FFmpeg
  process to exit before returning the playlist.
- Add tests for running-session playlist readiness, generated segment serving,
  missing segment not-ready conflicts, cancellation, and failure cleanup.

Validation:

```text
cargo nextest run -p nako-server hls_source --no-fail-fast
cargo nextest run -p nako-server hls_segment --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-server/src/app/tests/playback.rs`
- `crates/nako-server/src/http/tests/playback.rs`
- `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
- `cargo nextest run -p nako-server hls_source --no-fail-fast`
- `cargo nextest run -p nako-server hls_segment --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`
- `docs/workstreams/hls-progressive-runtime-boundary/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: playlist-facing HLS paths now return once the session playlist exists
  while FFmpeg continues running.
- Browser playlist and segment routes preserve playback-session segment URLs.
- Running sessions can serve generated manifest-approved segments; missing
  running segments return bounded not-ready conflicts.
- Cancellation still drives the linked transcode session to `Cancelled`.
- Keep browser and renderer ticket behavior intact in HPRB-050.

### HPRB-040 — Move HLS artifact reconstruction to a typed transcode boundary

Status: Completed
Owner: codex
Depends on: HPRB-030

Scope:

- `crates/nako-transcode/src/artifact.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-server/src/app/playback/hls_artifact.rs`
- persistence code only if typed reconstruction requires a schema-backed
  artifact manifest.

Goal:

- Delete server-local request-key substring parsing for HLS output shape and
  request variants.
- Make HLS artifact reconstruction consume typed transcode identity or a
  persisted artifact spec.

Validation:

```text
cargo nextest run -p nako-transcode hls_request_variant --no-fail-fast
cargo nextest run -p nako-server hls_artifact --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-server/src/app/playback/hls_artifact.rs`
- `crates/nako-transcode/src/artifact.rs`
- `crates/nako-transcode/src/lib.rs`
- `cargo nextest run -p nako-transcode hls_request_variant --no-fail-fast`
- `cargo nextest run -p nako-server hls_artifact --no-fail-fast`
- `docs/workstreams/hls-progressive-runtime-boundary/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: `nako-transcode` now owns `HlsArtifactSpec` reconstruction from the
  persisted transcode request identity.
- `nako-server` no longer parses `request_key` substrings for HLS variant,
  segment container, adaptive ladder, or media renditions.
- No schema migration was needed; the existing persisted request identity has
  enough typed material for manifest reconstruction.
- HPRB-050 should keep consuming `HlsArtifactManifest`/`HlsArtifactSpec` rather
  than reintroducing route-local artifact parsing.

### HPRB-050 — Consolidate playlist authoring and auth decoration

Status: Completed
Owner: codex
Depends on: HPRB-040

Scope:

- `crates/nako-server/src/app/playback/playlist.rs`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/http/tests/playback.rs`
- `crates/nako-server/src/http/tests/renderer.rs`

Goal:

- Replace separate playlist rewrite and ticket query-appending passes with one
  manifest-aware HLS playlist authoring boundary.
- Preserve Public Client and renderer HLS transport URLs.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server renderer --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-server/src/app/playback/playlist.rs`
- `crates/nako-server/src/app/playback/hls_artifact.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/http/playback.rs`
- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server renderer --no-fail-fast`
- `docs/workstreams/hls-progressive-runtime-boundary/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: HLS session playlist authoring now combines entry playlist authoring,
  session route binding, and browser/renderer query decoration in one
  manifest-aware app-layer boundary.
- DONE: HTTP playback no longer performs a separate HLS playlist ticket
  append pass.
- DONE: Public Client browser tickets and renderer cast-ticket HLS transport
  URLs remain covered by existing route tests.
- Do not add LL-HLS, key delivery, DASH, or DRM in this lane; split those from
  HPRB-060 if still desired.

### HPRB-060 — Verify, document, and close or split follow-ons

Status: Completed
Owner: planner
Depends on: HPRB-050

Scope:

- `docs/workstreams/hls-progressive-runtime-boundary`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Goal:

- Run fresh focused gates.
- Record final evidence and residual risks.
- Close the lane or split follow-ons for resource scheduler, LL-HLS, DASH,
  audio duplication removal, or remote transcode workers.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Review:

- Use `review-workstream` and `verify-rust-workstream` before closeout.

Evidence:

- `docs/workstreams/hls-progressive-runtime-boundary/EVIDENCE_AND_GATES.md`
- `docs/workstreams/hls-progressive-runtime-boundary/HANDOFF.md`
- `docs/workstreams/hls-progressive-runtime-boundary/WORKSTREAM.json`

Handoff:

- DONE: final closeout gates passed with fresh evidence.
- DONE: playlist readiness now rejects partially written running playlists that
  do not yet contain a media or variant URI line.
- DONE: architecture and workstream indexes mark the lane as completed.
- DONE: LL-HLS, DASH/CMAF, DRM/key delivery, remote transcode workers, selected
  audio cleanup, and resource scheduling remain split follow-ons.
