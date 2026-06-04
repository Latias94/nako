# Lane E: Playback Artifact Cleanup Service Extraction

Status: DONE
Date: 2026-06-05

## Summary

Extracted startup playback artifact cleanup into a focused
`playback_artifact_cleanup` app helper module. `startup.rs` now keeps startup
workflow orchestration and report wrapping, while the cleanup helper owns
transcode-session paging, artifact target resolution, canonical-root checks,
retention checks, recursive summary, and file/directory removal.

## Changed Files

- `crates/nako-server/src/app.rs`
- `crates/nako-server/src/app/startup.rs`
- `crates/nako-server/src/app/playback_artifact_cleanup.rs`
- `.trellis/tasks/06-04-10-hour-media-server-architecture-campaign/campaign-plan.md`
- `.trellis/tasks/06-04-10-hour-media-server-architecture-campaign/implementation/lane-e-playback-artifact-cleanup.md`

## Scope Guard

- No HTTP route changes.
- No schema changes.
- No public or Admin API contract changes.
- No playback planner changes.
- No storage repair route changes.

## Validation

- `cargo nextest run -p nako-server startup --no-fail-fast` passed: 47/47.
- `cargo check -p nako-server --tests` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only LF/CRLF warnings.
- `python ./.trellis/scripts/task.py validate ./.trellis/tasks/06-04-10-hour-media-server-architecture-campaign` passed.

## Follow-Ups

- Consider adding direct unit coverage for the helper if future work adds an
  Admin cleanup preview or manual cleanup action. Existing startup tests still
  cover the current startup behavior.

WORKSTREAM_RESULT: DONE
