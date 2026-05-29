# Playback Planner Transcode Seam Deepening - TODO

Status: Completed
Last updated: 2026-05-29

Task IDs use the `PPTS` prefix.

## M0 - Scope And Evidence

- [x] PPTS-010 [owner=codex] [deps=none] [scope=docs/workstreams/playback-planner-transcode-seam-deepening,docs/workstreams/README.md,docs/architecture]
  Goal: Open the fearless refactor lane, freeze scope, and link it from the
  architecture/workstream indexes.
  Validation: `python3 -m json.tool docs/workstreams/playback-planner-transcode-seam-deepening/WORKSTREAM.json`
  Evidence: `README.md`, `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`
  Handoff: Continue with PPTS-020.

## M1 - Transcode-Owned Playback Profile Builders

- [x] PPTS-020 [owner=codex] [deps=PPTS-010] [scope=crates/nako-transcode,crates/nako-playback]
  Goal: Add transcode-owned remux/HLS playback profile request builders and
  delete `PlaybackTargetProfile` methods that directly construct
  `TranscodeProfile`.
  Validation: `cargo nextest run -p nako-playback --no-fail-fast`; `cargo nextest run -p nako-transcode profile --no-fail-fast`
  Evidence: `PlaybackRemuxProfileRequest`, `PlaybackHlsProfileRequest`,
  `build_playback_remux_profile`, and `build_playback_hls_profile` now live in
  `nako-transcode`; `PlaybackTargetProfile` no longer has
  `*_transcode_profile` methods. Both validation commands passed.
  Handoff: DONE. Continue with PPTS-030.

## M2 - Server Composition Update

- [x] PPTS-030 [owner=codex] [deps=PPTS-020] [scope=crates/nako-server,crates/nako-api]
  Goal: Update playback app and test identity helpers to call transcode-owned
  builders while preserving remux/HLS request identity and runtime behavior.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`
  Evidence: playback app remux/HLS source contexts and app/http identity
  helpers call transcode-owned builders. Server playback gate passed.
  Handoff: DONE. Continue with PPTS-040.

## M3 - Verification And Closeout

- [x] PPTS-040 [owner=codex] [deps=PPTS-030] [scope=docs/workstreams/playback-planner-transcode-seam-deepening]
  Goal: Record evidence, run final checks, and close or split remaining seam
  work.
  Validation: `cargo fmt --all -- --check`; `git diff --check`; focused gates
  from `EVIDENCE_AND_GATES.md`
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`,
  `CLOSEOUT.md`
  Handoff: DONE. Follow-ons remain feature lanes, not incomplete work in this
  seam cleanup.
