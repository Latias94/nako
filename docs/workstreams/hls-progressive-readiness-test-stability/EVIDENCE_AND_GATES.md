# HLS Progressive Readiness Test Stability - Evidence And Gates

Status: Closed
Last updated: 2026-05-31

## Required Gates

### HPRTS-010 - Scope and repro freeze

```text
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-progressive-readiness-test-stability docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

### HPRTS-020 - Stabilize progressive readiness

```text
cargo nextest run -p nako-server app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes --no-fail-fast
cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

### HPRTS-030 - Closeout

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Evidence Ledger

### HPRTS-010 - Scope and repro freeze

Status: Done with concerns

Evidence collected:

- HRLB-040 first full HLS gate failed 69/71 with failures in:
  `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`
  and
  `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`.
- Both failed tests passed individually.
- HRLB-040 second full HLS gate failed 69/71 with the same two failures.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed before follow-on documentation was added.

Scope decision:

- The follow-on owns HLS progressive readiness test stability only.
- PAIP artifact I/O pressure, resource admission unification, remote workers,
  LL-HLS/CMAF, player UX, DTO changes, schema changes, and VFS behavior changes
  remain out of scope.

Fresh validation:

```text
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-progressive-readiness-test-stability docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Result: passed on 2026-05-31. The `WORKSTREAM.json` file parsed successfully.
Scoped `git diff --check` emitted only Git line-ending normalization warnings
for tracked touched files and no whitespace errors. Because the HPRTS files are
new and untracked until staged, `rg -n "[ \t]+$"` was also run across the HPRTS
directory and the HRLB-040 journal; it found no trailing whitespace.

### HPRTS-020 - Stabilize progressive readiness

Status: Done with concerns

Diagnosis:

- The two failing progressive readiness tests passed individually before the
  fix:
  `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`
  passed in 6.803s and
  `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`
  passed in 7.336s.
- The same `hls` filter passed with reduced nextest concurrency:
  `cargo nextest run -p nako-server hls --no-fail-fast -j 4` passed 71/71 in
  143.358s.
- The default full HLS gate later returned the app readiness test in 69.218s
  and the HTTP readiness test in 71.187s, proving the original fixed 60s
  guard was below the Windows full-suite process startup tail.

Change:

- Added a named `process_backed_hls_playlist_readiness_timeout()` helper in
  the app and HTTP playback test modules.
- The helper keeps the guard at 60s off Windows and uses 180s on Windows.
- No production HLS runtime behavior, DTO, schema, VFS, PAIP, LL-HLS, remote
  worker, or player UX behavior changed.

Fresh validation:

```text
cargo nextest run -p nako-server app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes --no-fail-fast
cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check -- crates/nako-server/src/app/tests/playback.rs crates/nako-server/src/http/tests/playback.rs
```

Result: passed on 2026-05-31. The default full HLS gate passed 71/71 in
116.102s with 26 slow tests. The two target tests passed individually after
the change in 10.748s and 6.702s. The scoped diff check emitted only Git
line-ending normalization warnings for the two touched Rust files and no
whitespace errors.

### HPRTS-030 - Closeout

Status: Done with concerns

Fresh planner verification:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
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

- HPRTS is closed.
- The full HLS gate is trustworthy enough to retry HRLB-040 closeout.
- Broader Windows HLS fixture scheduling or nextest grouping remains a
  separate follow-on candidate, not part of this workstream.

## Residual Risks

- A test-stability investigation may reveal a real HLS runtime bug. If so,
  stop and return to planner before implementing new production runtime
  behavior.
- The full HLS gate still has many process-backed tests above 60s on Windows.
  `HPRTS-030` should review whether broader fixture scheduling or nextest
  grouping is worth a separate follow-on after HRLB closeout is unblocked.
