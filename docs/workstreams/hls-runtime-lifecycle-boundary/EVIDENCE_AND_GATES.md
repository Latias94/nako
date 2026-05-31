# HLS Runtime Lifecycle Boundary - Evidence And Gates

Status: Closed
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

### HRLB-030 - Follow-on split decisions

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check
```

`HRLB-030` is docs/planning-only. Do not run Rust gates or implement behavior.

### HRLB-040 - Closeout verification

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/workstreams/hls-progressive-readiness-test-stability docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

The Rust HLS gate must pass before this workstream is marked closed. Individual
test reruns may classify a failure, but they do not replace the full HLS gate.

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

### HRLB-030 - Follow-on split decisions

Status: Done with concerns

Evidence collected:

- Added the HRLB-030 decision table to `DESIGN.md`.
- Classified PAIP artifact I/O pressure, resource admission unification,
  remote workers, LL-HLS/CMAF, player UX, and HLS test stability as separate
  follow-ons rather than HRLB implementation work.
- Recommended `proposed:hls-progressive-readiness-test-stability` as the next
  bounded workstream before PAIP or LL-HLS/CMAF, because HRLB-020 left a
  load-sensitive progressive-readiness concern even though final HLS validation
  passed.
- Kept PAIP as `proposed:hls-artifact-io-pressure-enforcement` and documented
  the required playback/storage coordination.

Fresh validation:

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check
```

Result: passed on 2026-05-31. `git diff --check` emitted only line-ending
normalization warnings for touched Markdown/JSON files and no whitespace
errors.

### HRLB-040 - Closeout verification

Status: Blocked

Review findings:

- Workstream compliance: blocking missing gate. The required full HLS gate
  failed twice under default nextest concurrency, so HRLB must not be marked
  closed.
- Code quality: no HRLB-040 Rust code was changed. The observed failure is
  isolated to progressive readiness tests that pass individually.
- Scope: no PAIP, LL-HLS/CMAF, remote worker, player UX, DTO, schema, or VFS
  behavior work was performed.

Fresh validation:

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes --no-fail-fast
cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result on 2026-05-31:

- `python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json`:
  passed before the closeout documentation update.
- Scoped documentation `git diff --check`: passed before the closeout
  documentation update.
- First `cargo nextest run -p nako-server hls --no-fail-fast`: failed, 69/71
  passed, 434 skipped. Failures:
  `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`
  and
  `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`.
- Individual rerun of
  `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`:
  passed, 1/1.
- Individual rerun of
  `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`:
  passed, 1/1.
- Second `cargo nextest run -p nako-server hls --no-fail-fast`: failed, 69/71
  passed, 434 skipped, with the same two failures.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed before the closeout documentation update.
- Post-documentation
  `python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json`:
  passed.
- Post-documentation
  `python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json`:
  passed.
- Post-documentation scoped `git diff --check`: passed with only Git
  line-ending normalization warnings and no whitespace errors.
- Post-documentation `git diff --check`: passed with only Git line-ending
  normalization warnings and no whitespace errors.
- New untracked HPRTS docs and the HRLB-040 journal were checked with
  `rg -n "[ \t]+$"` for trailing whitespace; no matches were found.

Closeout decision:

- Initial HRLB closeout remained active because the required full HLS gate
  failed.
- The remaining scope was split to
  `docs/workstreams/hls-progressive-readiness-test-stability/`.

### HRLB-040 - Closeout retry after HPRTS

Status: Closed with concerns

Fresh planner verification:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
git diff --check
```

Result on 2026-05-31:

- `cargo nextest run -p nako-server hls --no-fail-fast`: passed, 71/71, 434
  skipped, 26 slow tests, 111.217s.
- `cargo fmt --all -- --check`: passed.
- Both `WORKSTREAM.json` files parsed successfully.
- `git diff --check`: passed with only Git line-ending normalization warnings
  for touched files and no whitespace errors.

Closeout decision:

- HRLB is closed.
- HPRTS is closed.
- PAIP artifact I/O pressure, resource admission unification, remote workers,
  LL-HLS/CMAF, and player UX remain separate follow-ons and are not approved
  inside HRLB.
