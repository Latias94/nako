# Audio Compatibility Downmix Normalization - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Required Gates

```text
python -m json.tool docs/workstreams/audio-compatibility-downmix-normalization/WORKSTREAM.json
cargo nextest run -p nako-playback audio --no-fail-fast
cargo nextest run -p nako-transcode hls audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run the playback package gate before transcode propagation. Broaden to server
HLS only after runtime adaptation changes.

## Evidence Ledger

### ACDN-010 - Scope and evidence freeze

Status: Done

Evidence:

- `docs/workstreams/audio-compatibility-downmix-normalization/DESIGN.md`
- `docs/workstreams/audio-compatibility-downmix-normalization/TODO.md`
- `docs/workstreams/audio-compatibility-downmix-normalization/WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Notes:

- The first executable task is playback requirement vocabulary only.
- HDR tone mapping, subtitle burn-in, persisted preferences, and web controls
  are outside this lane.

### ACDN-020 - Playback requirement vocabulary

Status: Done

Evidence:

- `crates/nako-playback/src/values.rs`
- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/lib.rs`
- `cargo nextest run -p nako-playback audio --no-fail-fast`
  - 2026-05-30: Passed, 7 tests run and 23 skipped by filter.
- `cargo nextest run -p nako-playback --no-fail-fast`
  - 2026-05-30: Passed, 30 tests run.
- `cargo fmt --all -- --check`
  - 2026-05-30: Passed.
- `git diff --check`
  - 2026-05-30: Passed after task evidence updates.

Planner verification on 2026-05-30:

- `cargo nextest run -p nako-playback audio --no-fail-fast` - passed; 7 tests
  run and 23 skipped by filter.
- `cargo nextest run -p nako-playback --no-fail-fast` - passed; 30 tests run.
- `cargo fmt --all -- --check` - passed.
- `git diff --check` - passed with only Windows line-ending warnings.

Notes:

- `nako-playback` now owns `PlaybackAudioOutputRequirement` plus audio
  downmix, normalization, and compatibility reason values.
- Channel-limited selected audio now produces a transcode requirement with a
  downmix target channel count.
- Remux evaluation now respects client audio channel support so remux is not
  selected when an audio downmix is required.
- This task did not edit `nako-transcode`, server playback, HDR, subtitle
  burn-in, or web player code.

### ACDN-030 - Transcode policy propagation

Status: Done

Evidence:

- `crates/nako-transcode/src/policy.rs`
- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/tests/playback.rs`
- `crates/nako-server/src/http/tests/playback.rs`
- `cargo nextest run -p nako-transcode audio --no-fail-fast`
  - 2026-05-30: Passed, 10 tests run and 72 skipped by filter.
- `cargo nextest run -p nako-server hls --no-fail-fast`
  - 2026-05-30: Failed under the full HLS filter with 59 tests passed and 2
    existing running-playlist timeout tests failed:
    `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`
    and
    `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`.
- `cargo nextest run -p nako-server hls --no-fail-fast`
  - 2026-05-30: Passed after HLS gate stabilization, 61 tests run and 432
    skipped by filter.
- `cargo nextest run -p nako-server app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes --no-fail-fast`
  - 2026-05-30: Passed when isolated, 1 test run.
- `cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running --no-fail-fast`
  - 2026-05-30: Passed when isolated, 1 test run.
- `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
  - 2026-05-30: Failed in the current ACDN-030 worktree with 9 tests passed
    and the same 2 running-playlist timeout tests failed.
- `cargo nextest run -p nako-server hls_playlist --no-fail-fast -j 1`
  - 2026-05-30: Passed in the current ACDN-030 worktree, 11 tests run.
- `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
  - 2026-05-30: Failed in a temporary detached baseline worktree at
    `b770cac9` before the ACDN-030 uncommitted changes, with 10 tests passed
    and
    `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`
    failed.
- `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
  - 2026-05-30: Passed after HLS gate stabilization, 11 tests run and 482
    skipped by filter.
- `cargo fmt --all -- --check`
  - 2026-05-30: Passed.
- `git diff --check`
  - 2026-05-30: Passed with only Windows line-ending warnings.

Notes:

- `nako-transcode` now carries `TranscodeAudioOutputRequirement` through HLS
  execution policy, pipeline request/plan, and transcode profile identity.
- Server playback maps the playback-owned audio output requirement into
  transcode policy at the HLS adaptation boundary.
- Compatible audio source facts are collapsed to the empty transcode audio
  output requirement so ordinary HLS request keys do not churn when no downmix
  or normalization is requested.
- The task did not edit HDR, subtitle burn-in, web player, public DTO, or
  generated contract code.
- ACDN-030 diagnosis found the full-filter HLS failure is an existing
  concurrency-sensitive test issue: the narrowed `hls_playlist` filter fails in
  both current and pre-ACDN-030 baseline worktrees under default nextest
  parallelism, while the same narrowed filter passes with `-j 1`.
- HLS gate stabilization kept runtime behavior unchanged and made the tests
  reflect the existing contract more accurately: playlist readiness does not
  guarantee the first segment is immediately route-ready under concurrent load,
  so the HTTP running-playlist test now uses the existing segment readiness
  retry helper.

Planner verification on 2026-05-30:

- `python -m json.tool docs/workstreams/audio-compatibility-downmix-normalization/WORKSTREAM.json`
  passed.
- `cargo nextest run -p nako-transcode audio --no-fail-fast` passed with 10
  tests run and 72 skipped by filter.
- `cargo nextest run -p nako-server hls --no-fail-fast` passed with 61 tests
  run and 432 skipped by filter.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending normalization
  warnings.
- ACDN-030 is planner-verified and can be accepted.

## Residual Risks

- Real device audio capability databases may require a later profile import or
  calibration lane.
- Normalization defaults can become product-sensitive; keep initial behavior
  deterministic and explainable.
- Downmix and HDR implementation both touch playback/transcode seams. Do not
  implement HDR code concurrently with this lane.
