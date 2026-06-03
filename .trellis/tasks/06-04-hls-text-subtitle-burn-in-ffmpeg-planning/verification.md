# Verification

Final gate evidence for HLS text subtitle burn-in FFmpeg planning.

## Commands

* `cargo nextest run -p nako-transcode hls_subtitle_burn_in_plan --no-fail-fast`
  - Pass: 3 tests run, 3 passed.
* `cargo fmt --all`
  - Pass.
* `cargo nextest run -p nako-transcode hls --no-fail-fast`
  - Pass: 69 tests run, 69 passed.
* `cargo check -p nako-transcode -p nako-server --tests`
  - Pass.
* `cargo nextest run -p nako-server hls_source_selected_subtitle_uses_sidecar_rendition_identity_and_artifacts --no-fail-fast`
  - Pass: 1 test run, 1 passed.
* `cargo fmt --all -- --check`
  - Pass.
* `git diff --check`
  - Pass; only Git CRLF conversion warnings were printed.
* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-hls-text-subtitle-burn-in-ffmpeg-planning`
  - Pass: `implement.jsonl` and `check.jsonl` valid.
