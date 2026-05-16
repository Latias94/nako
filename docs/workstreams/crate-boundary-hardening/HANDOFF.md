# Crate Boundary And Public Protocol Hardening Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M28 is complete. The lane shipped the first public protocol extraction, core
module deepening, workflow decomposition, and playback seam clarification:

- `taru-client-protocol` is an Apache-2.0 crate with the first public system
  envelope types.
- `taru-api` re-exports those protocol types and keeps server-only pagination
  mapping in `page_info_from_request`.
- `taru-core/src/media.rs` is now a facade over concept modules under
  `crates/taru-core/src/media/`.
- `taru-core/src/repository.rs` is now a facade over repository trait groups
  under `crates/taru-core/src/repository/`.
- `taru-library/src/lib.rs` is now a facade over `summary`, `scan`, `index`,
  `probe`, `local_inference`, and private `failure`.
- `taru-nfo/src/lib.rs` is now a facade over `codec`, `summary`, `workflow`,
  `import`, and `export`.
- `taru-streaming/src/lib.rs` is now a facade over playback `selection` and
  direct-play response planning.
- `taru-transcode/src/lib.rs` is now a facade over `plan`, `hardware`,
  `ffmpeg`, `session`, `runtime`, `remux`, `hls`, and private runner helpers.
- Server playback remains the app composition layer.

## Completed Closeout

- Task ID: CBH-060
- Owner: codex
- Files: `docs/workstreams/crate-boundary-hardening/*`
- Validation: `cargo fmt --all -- --check`, `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast`, `git diff --check`

## Decisions Since Last Update

- Public client wire types should move toward a permissive protocol crate.
- `taru-api` should remain the AGPL server adapter layer.
- `taru-core` should be deepened by module before any new crate split.
- `taru-library` and `taru-nfo` should be decomposed before behavior changes
  land in those workflows.
- Playback ownership should stay explicit across `taru-streaming`,
  `taru-transcode`, and `taru-server`.
- The first public protocol slice should stay intentionally small; move more
  DTOs only when the server adapter mapping remains clear.
- `taru-core` media records should keep public re-exports stable while
  internals move by concept.
- `taru-core` repository contracts should keep public re-exports stable while
  internal trait groups move into concept modules.
- Library scanning, indexing, probing, local inference, and summaries now have
  separate modules inside `taru-library`.
- NFO XML round-trip, service workflow, import, and export now have separate
  modules inside `taru-nfo`.
- Playback source selection/direct-play response planning is separate from
  FFmpeg runtime execution.
- `taru-transcode` owns FFmpeg command plans, hardware acceleration selection,
  process-local session transitions, runtime limits, cancellation, and remux
  or HLS runner execution.
- `taru-server/src/app/playback/*` should stay focused on VFS input staging,
  persistence, domain events, cancellation registry, and HTTP-facing outputs.

## Blockers

- None.

## Follow-Ons

- Broaden `taru-client-protocol` in small slices when client-facing DTOs can
  move without pulling in server internals.
- Narrow repository traits by use case when a concrete caller benefits from a
  smaller contract; keep the current public re-export facade until migration
  pressure justifies breaking it.
- Continue behavior work in separate domain lanes rather than reopening this
  structural boundary lane.
