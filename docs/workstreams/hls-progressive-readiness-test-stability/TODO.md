# HLS Progressive Readiness Test Stability - TODO

Status: Closed
Last updated: 2026-05-31

## M0 - Scope And Repro Freeze

- [x] HPRTS-010 [owner=planner] [deps=none] [scope=docs/workstreams/hls-progressive-readiness-test-stability,docs/workstreams/hls-runtime-lifecycle-boundary]
  Goal: Split the progressive readiness gate failure from HRLB closeout and freeze the repro evidence.
  Validation: `python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json`; `git diff --check -- docs/workstreams/hls-progressive-readiness-test-stability docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`
  Evidence: `EVIDENCE_AND_GATES.md` and HRLB-040 evidence.
  Context: `docs/workstreams/hls-progressive-readiness-test-stability/CONTEXT.jsonl`.
  Handoff: DONE_WITH_CONCERNS. Full HLS gate failed twice on progressive readiness tests; both failed tests passed individually. HRLB remains blocked until this follow-on passes.

## M1 - Stabilize Progressive Readiness Gate

- [x] HPRTS-020 [owner=codex] [deps=HPRTS-010] [scope=crates/nako-server/src/app/tests/playback.rs,crates/nako-server/src/http/tests/playback.rs,crates/nako-server/src/app/playback/hls.rs,crates/nako-server/src/app/playback/hls_artifact.rs]
  Goal: Identify and fix the default full-suite HLS progressive readiness instability without introducing new HLS runtime behavior unless the planner explicitly approves it.
  Validation: `cargo nextest run -p nako-server app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes --no-fail-fast`; `cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`
  Review: Use `review-workstream` before accepting completion.
  Evidence: updated tests or diagnostics and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/hls-progressive-readiness-test-stability/CONTEXT.jsonl`.
  Handoff: DONE_WITH_CONCERNS. Classified as Windows full-suite process-backed test timing under default nextest concurrency. The fix is test-only: the two progressive readiness tests now use a named process-backed playlist readiness timeout that remains 60s off Windows and is 180s on Windows. No production runtime, DTO, schema, or VFS behavior changed.

## M2 - Closeout And HRLB Retry

- [x] HPRTS-030 [owner=planner] [deps=HPRTS-020] [scope=docs/workstreams/hls-progressive-readiness-test-stability,docs/workstreams/hls-runtime-lifecycle-boundary,docs/architecture/PLAYBACK.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Verify final gates, close this follow-on, and rerun HRLB-040 closeout.
  Validation: `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and HRLB-040 closeout notes.
  Context: `docs/workstreams/hls-progressive-readiness-test-stability/CONTEXT.jsonl`.
  Handoff: DONE_WITH_CONCERNS. Fresh full HLS, fmt, JSON, and diff gates passed; the follow-on is closed and HRLB-040 closeout was retried successfully. The residual concern is broader Windows HLS suite slowness, which remains a separate follow-on candidate.
