# Evidence And Gates

## Required Gates

- `python -m json.tool docs/workstreams/playback-planner-transcode-value-vocabulary/WORKSTREAM.json`
- `cargo check -p nako-playback --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo tree -p nako-playback --depth 1`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Log

### PPTV-010

- 2026-05-29: `python -m json.tool docs/workstreams/playback-planner-transcode-value-vocabulary/WORKSTREAM.json` passed.
- 2026-05-29: `git diff --check -- docs\workstreams\playback-planner-transcode-value-vocabulary docs\architecture\WORKSTREAM_LINKS.md docs\workstreams\README.md` passed with only CRLF conversion warnings from Git.

### PPTV-020

- 2026-05-29: `cargo check -p nako-playback --tests` passed.
- 2026-05-29: `cargo check -p nako-server --tests` passed.
- 2026-05-29: `cargo nextest run -p nako-playback --no-fail-fast` passed, 19 tests.
- 2026-05-29: `cargo nextest run -p nako-server playback --no-fail-fast` first timed out at 244 seconds without test output, then passed on rerun with a longer timeout, 122 tests passed and 329 skipped.
- 2026-05-29: `cargo tree -p nako-playback --depth 1` passed and showed only `nako-core` plus `serde` as direct dependencies.
- 2026-05-29: `cargo fmt --all -- --check` passed.
- 2026-05-29: `git diff --check` passed with only CRLF conversion warnings from Git.

### PPTV-030

- 2026-05-29: `python -m json.tool docs/workstreams/playback-planner-transcode-value-vocabulary/WORKSTREAM.json` passed.
- 2026-05-29: Closeout written and lane marked completed.
