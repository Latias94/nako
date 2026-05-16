# Crate Boundary and Public Protocol Hardening Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Smallest Current Repro

The current baseline is architectural rather than behavioral:

- `crates/taru-api/Cargo.toml` depends directly on `taru-core` and
  `taru-streaming`.
- `crates/taru-core/src/media.rs` and `crates/taru-core/src/repository.rs`
  are broad aggregation files.
- `crates/taru-library/src/lib.rs` and `crates/taru-nfo/src/lib.rs` are
  large workflow modules.
- `crates/taru-server/src/app/playback/*` already has submodules, but the
  orchestration contract still needs explicit ownership lines.

## Gate Set

### Targeted Iteration Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
```

### Public Protocol Gate

```bash
cargo nextest run -p taru-api --no-fail-fast
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo tree -p taru-client-protocol
```

Use the tree output to confirm the public protocol crate stays dependency-light
and does not import `taru-core`, `taru-streaming`, or `taru-server`.

### Core And Workflow Gate

```bash
cargo nextest run -p taru-core --no-fail-fast
cargo nextest run -p taru-db --no-fail-fast
cargo nextest run -p taru-library --no-fail-fast
cargo nextest run -p taru-nfo --no-fail-fast
```

### Playback Gate

```bash
cargo nextest run -p taru-streaming --no-fail-fast
cargo nextest run -p taru-transcode --no-fail-fast
cargo nextest run -p taru-server --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Anchors

- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
- `docs/workstreams/crate-boundary-hardening/DESIGN.md`
- `docs/workstreams/crate-boundary-hardening/TODO.md`
- `docs/workstreams/crate-boundary-hardening/MILESTONES.md`
- `docs/workstreams/crate-boundary-hardening/PHASE28_0_CRATE_BOUNDARY_BASELINE.md`
- `crates/taru-client-protocol`
- `crates/taru-api`
- `crates/taru-core/src/media.rs`
- `crates/taru-core/src/media/*`
- `crates/taru-core/src/repository.rs`
- `crates/taru-core/src/repository/*`
- `crates/taru-library/src/lib.rs`
- `crates/taru-nfo/src/lib.rs`
- `crates/taru-server/src/app/playback/*`

## Notes

The public protocol boundary should be verified both by tests and by a
dependency-direction review. A green build is not enough if the new crate can
still reach AGPL server internals.

## Recorded Evidence

### CBH-020 Public Client Protocol Extraction

- `crates/taru-client-protocol` uses `license = "Apache-2.0"`.
- `crates/taru-client-protocol` owns `CLIENT_PROTOCOL_VERSION`,
  `HealthResponse`, `ErrorResponse`, and `PageInfo`.
- `taru-api` re-exports those protocol types and keeps `PageRequest` mapping
  inside the server adapter through `page_info_from_request`.
- `cargo tree -p taru-client-protocol` shows only `serde` as a normal
  dependency and `serde_json` as a dev-dependency; it does not import
  `taru-core`, `taru-streaming`, or `taru-server`.
- `cargo nextest run -p taru-client-protocol --no-fail-fast`: 1 test passed.
- `cargo nextest run -p taru-api --no-fail-fast`: 4 tests passed.
- `cargo nextest run -p taru-server --no-fail-fast`: 93 tests passed.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.

### CBH-030 Core Module Deepening

- `crates/taru-core/src/media.rs` is now a facade that re-exports concept
  modules from `crates/taru-core/src/media/`.
- New `taru-core` media modules: `library`, `profile`, `item`, `source`,
  `catalog`, `scan`, `artwork`, `probe`, `provider`, and `metadata`.
- `crates/taru-core/src/repository.rs` is now a facade that re-exports
  repository trait groups from `crates/taru-core/src/repository/`.
- New `taru-core` repository modules: `addon`, `automation`, `catalog`,
  `ingestion`, `jobs`, `library`, `media`, `metadata`, `pagination`, `scan`,
  `transaction`, `transcode`, `vfs`, and `webhook`.
- Repository module imports are scoped to each trait group instead of sharing
  the previous broad prelude.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- `cargo nextest run -p taru-core --no-fail-fast`: 3 tests passed.
- `cargo nextest run -p taru-db --no-fail-fast`: 32 tests passed.

### CBH-040 Library And NFO Decomposition

- `crates/taru-library/src/lib.rs` is now a facade that re-exports focused
  workflow modules.
- New `taru-library` modules: `summary`, `scan`, `index`, `probe`,
  `local_inference`, and private `failure`.
- `crates/taru-nfo/src/lib.rs` is now a facade that re-exports public NFO
  API types and keeps `NfoService` state in one place.
- New `taru-nfo` modules: `codec`, `summary`, `workflow`, `import`, and
  `export`.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- `cargo check -p taru-library --tests` passed.
- `cargo nextest run -p taru-library --no-fail-fast`: 15 tests passed.
- `cargo check -p taru-nfo --tests` passed.
- `cargo nextest run -p taru-nfo --no-fail-fast`: 8 tests passed.

### CBH-050 Playback Seam Clarification

- `crates/taru-streaming/src/lib.rs` is now a facade over `selection` and
  `direct`.
- `taru-streaming::selection` owns playback source selection decisions and
  client capability matching.
- `taru-streaming::direct` owns direct-play range parsing, response planning,
  and content-type inference.
- `crates/taru-transcode/src/lib.rs` is now a facade over `plan`, `hardware`,
  `ffmpeg`, `session`, `runtime`, `remux`, `hls`, and private `runner_util`.
- `taru-transcode::hardware` owns hardware acceleration detection and
  selection.
- `taru-transcode::ffmpeg` owns command argument planning.
- `taru-transcode::session` owns the process-local transcode session state
  machine.
- `taru-transcode::runtime`, `remux`, and `hls` own runtime limits,
  cancellation, and FFmpeg runner execution.
- `crates/taru-server/src/app/playback/*` remains the composition layer that
  combines streaming decisions, VFS input staging, transcode runtime calls,
  persisted sessions, domain events, and HTTP-facing app outputs.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- `cargo check -p taru-streaming --tests` passed.
- `cargo nextest run -p taru-streaming --no-fail-fast`: 5 tests passed.
- `cargo check -p taru-transcode --tests` passed.
- `cargo nextest run -p taru-transcode --no-fail-fast`: 21 tests passed.
- `cargo nextest run -p taru-server --no-fail-fast`: 93 tests passed.

### CBH-060 Closeout

- Workstream docs now match shipped code seams for public protocol, core,
  library, NFO, streaming, transcode, and server playback boundaries.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- `cargo nextest run --workspace --no-fail-fast`: 251 tests passed.
- `git diff --check` passed.
