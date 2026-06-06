# M1 Workspace Evidence Run

## Commands

- 2026-06-06, git `b2c28ecb`:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode workspace`
  - Initial result: failed in delegated `cargo nextest run --workspace --no-fail-fast`.
  - Summary: `1433 tests run: 1431 passed (30 slow), 2 failed, 51 skipped`.
  - First failures:
    - `nako-server app::tests::playback::remux_playback_preflight_reuses_active_session_and_links_playback_sessions`;
    - `nako-transcode tests::hls_runner_can_publish_output_while_process_is_running`.
- Focused reruns before repair:
  - `cargo nextest run -p nako-server remux_playback_preflight_reuses_active_session_and_links_playback_sessions --no-fail-fast`
    passed with `1 test run: 1 passed, 654 skipped`.
  - `cargo nextest run -p nako-transcode hls_runner_can_publish_output_while_process_is_running --no-fail-fast`
    passed with `1 test run: 1 passed, 121 skipped`.
- Repair validation:
  - `cargo nextest run -p nako-server hls_playlist_playback --no-fail-fast`
    passed with `4 tests run: 4 passed, 651 skipped`.
  - `cargo nextest run -p nako-server remux_playback_preflight_reuses_active_session_and_links_playback_sessions --no-fail-fast`
    passed with `1 test run: 1 passed, 654 skipped`.
  - `cargo nextest run -p nako-transcode hls_runner_can_publish_output_while_process_is_running --no-fail-fast`
    passed with `1 test run: 1 passed, 121 skipped`.
- Final gate:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode workspace`
  - Delegated steps included `cargo fmt --all -- --check`,
    `git diff --check`, `cargo check --workspace --tests`, and
    `cargo nextest run --workspace --no-fail-fast`.
- Repair commit:
  `7769bc47 test(playback): stabilize workspace HLS timing gates`.

## Result

- Passed after repair: final M1 ladder `workspace` mode completed with exit
  code 0.
- Final workspace nextest summary:
  `1433 tests run: 1433 passed, 51 skipped`.
- Raw logs were kept under `target/m1-evidence/` for local inspection only and
  were not committed because they include local absolute paths.

## Classification

- Initial failure classification: repository-owned release-gate stability issue
  under playback/transcode tests, not an environment skip.
- Root cause classification: full-suite Windows execution starts many
  process-backed fake FFmpeg/HLS tests concurrently. Several tests observed
  transcode state or published HLS artifacts too early for that host load even
  though the same behavior passed in focused reruns.
- Repair:
  - `crates/nako-server/src/app/tests/playback.rs` now waits for remux
    preflight sessions to reach `Running` instead of asserting the immediately
    observed state.
  - Process-backed HLS playlist readiness tests use a larger bounded full-suite
    timeout.
  - `crates/nako-transcode/src/lib.rs` HLS runner artifact wait helper uses a
    bounded 5 second wait instead of the former 800 ms window.
- Follow-up classification: no separate implementation task was opened because
  the blocker was fixed narrowly in this workspace evidence task and the final
  workspace ladder passed.
