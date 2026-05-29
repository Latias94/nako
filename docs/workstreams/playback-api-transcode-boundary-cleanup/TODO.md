# TODO

## PATB-010 - Open Boundary Cleanup Lane

Status: Completed
Owner: Codex

Scope:

- Create the workstream docs and task ledger.
- Link the lane from playback/transcode architecture indexes.
- Record validation gates before code changes.

Validation:

- `python -m json.tool docs/workstreams/playback-api-transcode-boundary-cleanup/WORKSTREAM.json`
- `git diff --check -- docs/workstreams/playback-api-transcode-boundary-cleanup docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`

Handoff:

- Continue with PATB-020.

## PATB-020 - Remove `nako-api -> nako-transcode`

Status: Completed
Owner: Codex
Depends on: PATB-010

Scope:

- Remove `nako-transcode` from `crates/nako-api/Cargo.toml`.
- Replace Admin hardware/readiness fields that directly use transcode runtime
  types with API-local DTO types preserving serialized values.
- Move transcode-to-API conversion responsibility into `nako-server`.
- Keep Public Client playback decision DTO output unchanged without naming
  `nako_transcode` in `nako-api`.

Validation:

- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast`
- `cargo tree -p nako-api --depth 1`
- `cargo fmt --all -- --check`
- `git diff --check`

Handoff:

- PATB-020 is complete only when `cargo tree -p nako-api` has no direct
  `nako-transcode` edge and focused tests pass.

## PATB-030 - Review Remaining Playback/Transcode Surface

Status: Completed
Owner: Codex
Depends on: PATB-020

Scope:

- Inventory the remaining `nako-playback -> nako-transcode` public surface.
- Decide whether follow-on work should move planner-facing types into
  `nako-core`, a protocol crate, or server-owned adapters.
- Record a bounded task or separate workstream if the next cleanup crosses
  planner/runtime ownership.

Validation:

- `rg -n "nako_transcode|TranscodePlan|OutputContainer|HlsVariantPolicy|HlsSegmentContainer" crates/nako-playback crates/nako-api`
- Updated handoff notes with the recommended next slice.

## PATB-040 - Verify And Close Lane

Status: Completed
Owner: Codex
Depends on: PATB-030

Scope:

- Re-run lane gates.
- Record final evidence.
- Write closeout if no remaining tasks are in scope.

Validation:

- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
