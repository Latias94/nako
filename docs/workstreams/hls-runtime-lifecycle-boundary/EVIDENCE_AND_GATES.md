# HLS Runtime Lifecycle Boundary - Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Required Gates

### HRLB-010 - Lifecycle invariant freeze

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

`HRLB-010` is docs/research-only. Do not run Rust gates unless code changes are
explicitly approved.

### HRLB-020 - Behavior-preserving lifecycle boundary

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Broaden to playback/session or storage gates only if the task scope is expanded
by the planner.

## Evidence Ledger

### HRLB-010 - Lifecycle invariant freeze

Status: Done with concerns

Evidence collected:

- lifecycle state and transition table;
- readiness and segment wait semantics;
- cleanup ownership map;
- test coverage map;
- follow-on split decision for artifact I/O pressure and resource admission.

Notes:

- `DESIGN.md` now freezes active same-generation request handling, finished
  session reuse, different-generation supersede, running playlist readiness,
  segment readiness and one-shot wait, cancellation/timeout cleanup, startup
  stale-session cleanup, terminal artifact cleanup, staging input release, and
  PAIP artifact I/O pressure split guidance.
- Artifact I/O pressure should be split into a PAIP follow-on, using the
  existing `proposed:hls-artifact-io-pressure-enforcement` lane name unless the
  planner chooses a different slug. It should not be implemented inside
  `HRLB-020`.
- Coverage concerns for `HRLB-020`: focused HLS timeout cleanup, HLS-specific
  startup stale-session recovery, and HLS remote staged-input lease release are
  not yet directly covered even though adjacent generic/runner/lease tests
  exist.

Fresh validation:

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Result: passed on 2026-05-31. `git diff --check` emitted only existing Git
line-ending normalization warnings for touched Markdown/JSON files and no
whitespace errors.

### HRLB-020 - Behavior-preserving lifecycle boundary

Status: Done with concerns

Evidence collected:

- Added `hls_source_timeout_fails_session_and_cleans_visible_output` to assert
  `ffmpeg_hls` timeout mapping, persisted `Timeout` failure category, operator
  message, and serve-visible HLS output cleanup.
- Added `app_startup_marks_stale_hls_transcode_sessions_failed` to cover stale
  startup recovery with `HlsTranscode` session kind and HLS output layout.
- Added remote WebDAV HLS fixtures covering staged-input lease release after:
  successful HLS completion, runner failure, and HLS playlist admission
  rejection before background start.
- Added local test helpers for remote HLS app construction and released
  `FfmpegInput` staging manifest assertions.

Implementation decision:

- No server-local lifecycle coordinator/facade was introduced. The HRLB-010
  gaps were focused coverage gaps, and the existing local lifecycle boundary is
  sufficient for this behavior-preserving task until review identifies real
  duplication or ownership drift.
- PAIP artifact I/O pressure remains a follow-on. `HlsArtifactIo` enforcement
  was not changed.

Fresh validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result on 2026-05-31:

- Initial `cargo nextest run -p nako-server hls --no-fail-fast` was blocked
  before tests ran by `E0004` in `crates/nako-server/src/api_mapping.rs:134`.
  Planner accepted a narrow scope-out fix to map
  `HardwarePipelineStage::{ToneMap, SubtitleBurnIn}` into
  `AdminHardwarePipelineStage`.
- After the mapping fix, one full HLS run reached 69/70 and hit an existing
  load-sensitive progressive-readiness timeout in
  `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`.
  The same test passed when rerun individually.
- Final `cargo nextest run -p nako-server hls --no-fail-fast`: passed, 70/70
  run, 434 skipped.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed; Git emitted only line-ending normalization
  warnings for touched files.

## Residual Risks

- HLS lifecycle, artifact I/O pressure, and storage health can easily overlap.
  Keep implementation tasks serialized when they touch `resource.rs` or
  `hls_artifact.rs`.
- Do not let this lane become a catch-all for LL-HLS, DASH/CMAF, remote
  workers, hardware policy, or player UX.
- The HLS test suite remains load-sensitive around progressive playlist
  readiness under full-suite concurrency; final verification passed, but
  `HRLB-030` should decide whether to split a test-stability follow-on.
