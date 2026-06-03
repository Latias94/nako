# Verification

Final gate evidence for HLS seek/restart command identity.

## Commands

* `cargo check -p nako-transcode --tests`
  - Pass.
* `cargo fmt --all -- --check`
  - Pass.
* `git diff --check`
  - Pass; only Git CRLF conversion warnings were printed.
* `cargo nextest run -p nako-transcode hls --no-fail-fast`
  - Pass: 71 tests run, 71 passed, 45 skipped.
* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-hls-seek-restart-command-identity`
  - Pass: `implement.jsonl` and `check.jsonl` valid.

## Review Notes

* `HlsSeekCommandPlan` is constructed once from `HlsRequest` command assembly
  context.
* HLS input, video encoder, and muxer builders consume the plan instead of
  rechecking `HlsPlaybackGeneration`.
* Public routes, DTOs, playlist parsing, session persistence, and playback
  planner selection were not changed.
