# Crate Boundary and Public Protocol Hardening TODO

Status: Completed
Last updated: 2026-05-17

## M28.0 Scope And Evidence Freeze

- [x] CBH-010 [owner=planner] [deps=none] [scope=docs/workstreams/crate-boundary-hardening]
  Goal: Freeze the problem, target state, boundary rules, and first proof slice.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: docs/workstreams/crate-boundary-hardening/DESIGN.md
  Handoff: Planner owns this before code changes start.

## M28.1 Public Client Protocol Extraction

- [x] CBH-020 [owner=codex] [deps=CBH-010] [scope=crates/taru-api, crates/taru-client-protocol]
  Goal: Split public client wire types into a permissive protocol crate and keep `taru-api` as the server adapter.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, focused nextest for the protocol crate and `taru-api`, and a dependency-direction check that the protocol crate does not depend on server internals.
  Evidence: `crates/taru-client-protocol` owns `HealthResponse`, `ErrorResponse`, and `PageInfo`; `taru-api` re-exports them and owns `page_info_from_request`.
  Handoff: Continue moving public wire types in small slices; do not import `taru-core`, `taru-streaming`, or `taru-server` into `taru-client-protocol`.

## M28.2 Core Module Deepening

- [x] CBH-030 [owner=codex] [deps=CBH-020] [scope=crates/taru-core]
  Goal: Split `taru-core` into deeper media and repository modules without changing public behavior.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, focused nextest for `taru-core` and `taru-db`, and diff review of the module moves.
  Evidence: `crates/taru-core/src/media.rs` is now a re-export facade over concept modules under `crates/taru-core/src/media/`; `crates/taru-core/src/repository.rs` is now a facade over repository trait modules under `crates/taru-core/src/repository/`.
  Handoff: Keep public re-exports stable while future work narrows repository traits by use case.

## M28.3 Library And NFO Decomposition

- [x] CBH-040 [owner=codex] [deps=CBH-030] [scope=crates/taru-library, crates/taru-nfo]
  Goal: Split `taru-library` and `taru-nfo` into focused workflow modules.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, focused nextest for `taru-library` and `taru-nfo`, and behavior-preserving test updates.
  Evidence: `crates/taru-library/src/lib.rs` is now a facade over `summary`, `scan`, `index`, `probe`, `local_inference`, and private `failure`; `crates/taru-nfo/src/lib.rs` is now a facade over `codec`, `summary`, `workflow`, `import`, and `export`.
  Handoff: Keep scan/index/probe and codec/import/export/workflow responsibilities separate; do not move VFS/database workflow code into codec.

## M28.4 Playback Seam Clarification

- [x] CBH-050 [owner=codex] [deps=CBH-040] [scope=crates/taru-streaming, crates/taru-transcode, crates/taru-server/src/app/playback]
  Goal: Make playback planning, runtime, and server orchestration responsibilities explicit.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, focused nextest for `taru-streaming`, `taru-transcode`, and `taru-server`, and diff review of the seam cleanup.
  Evidence: `crates/taru-streaming/src/lib.rs` is now a facade over `selection` and `direct`; `crates/taru-transcode/src/lib.rs` is now a facade over `plan`, `hardware`, `ffmpeg`, `session`, `runtime`, `remux`, `hls`, and private `runner_util`; `crates/taru-server/src/app/playback/*` continues to compose streaming decisions and transcode runtime with persistence and HTTP translation.
  Handoff: Keep `taru-streaming` focused on source selection/direct-play planning, `taru-transcode` focused on FFmpeg/runtime, and server playback focused on orchestration, permissions, persistence, and HTTP translation.

## M28.5 Closeout

- [x] CBH-060 [owner=codex] [deps=CBH-050] [scope=docs/workstreams/crate-boundary-hardening]
  Goal: Close the lane or split a narrower follow-on if the public protocol crate deserves its own dedicated workstream.
  Validation: final validation gate set is recorded and docs match the shipped seams.
  Evidence: EVIDENCE_AND_GATES.md records the full closeout gate; WORKSTREAM.json marks the lane completed; HANDOFF.md records follow-ons.
  Handoff: Follow-ons remain for broader public protocol DTO migration and behavior-preserving repository trait narrowing.
