# TODO

## PPTV-010 - Open Value Vocabulary Lane

Status: Completed
Owner: Codex

Scope:

- Create workstream docs and task ledger.
- Link the lane from playback architecture indexes.
- Record the first executable implementation task.

Validation:

- `python -m json.tool docs/workstreams/playback-planner-transcode-value-vocabulary/WORKSTREAM.json`
- `git diff --check -- docs/workstreams/playback-planner-transcode-value-vocabulary docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`

## PPTV-020 - Remove `nako-playback -> nako-transcode`

Status: Completed
Owner: Codex
Depends on: PPTV-010

Scope:

- Add playback-owned planner value types in `nako-playback`.
- Replace public planner fields and tests that name transcode execution types.
- Add server-side adapters from playback values to `nako-transcode` execution
  values.
- Update server playback, renderer, and test call sites.
- Remove `nako-transcode` from `crates/nako-playback/Cargo.toml` and
  `Cargo.lock`.

Validation:

- `cargo check -p nako-playback --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo tree -p nako-playback --depth 1`
- `cargo fmt --all -- --check`
- `git diff --check`

## PPTV-030 - Verify And Close Lane

Status: Completed
Owner: Codex
Depends on: PPTV-020

Scope:

- Re-run lane gates.
- Record final evidence.
- Write closeout and residual follow-ons if needed.

Validation:

- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
