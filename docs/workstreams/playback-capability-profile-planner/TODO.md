# Playback Capability Profile Planner - TODO

Status: Active
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

- [x] PCP-010 [owner=planner] [deps=none] [scope=docs/adr,docs/workstreams/playback-capability-profile-planner]
  Goal: Freeze the profile-driven planner decision, scope, non-goals, and gates.
  Validation: docs agree on status, target state, and authority.
  Evidence: `docs/adr/0044-playback-capability-profile-planner.md`
  Handoff: Planner owns this before code slices start.

## M1 - Characterization And Profile Model

- [x] PCP-020 [owner=agent] [deps=PCP-010] [scope=crates/nako-playback]
  Goal: Add characterization tests for current direct play, remux, requested transcode, codec mismatch, policy denial, and profile identity behavior.
  Validation: `cargo nextest run -p nako-playback playback --no-fail-fast`
  Review: Confirm tests describe behavior to preserve or intentionally replace.
  Evidence: `crates/nako-playback/src/lib.rs`
  Handoff: Mark old behavior that should be deleted under fearless refactor.

- [x] PCP-030 [owner=agent] [deps=PCP-020] [scope=crates/nako-playback]
  Goal: Introduce `PlaybackTargetProfile`, compatibility conditions, and `PlaybackDecisionReport` with default browser/native profile builders.
  Validation: `cargo nextest run -p nako-playback profile --no-fail-fast`
  Review: Interface must be playback-shaped, not FFmpeg- or Jellyfin-shaped.
  Evidence: `crates/nako-playback/src/lib.rs`
  Handoff: Keep `nako-transcode` dependencies no wider than existing transcode plan records.

## M2 - Planner Migration

- [x] PCP-040 [owner=agent] [deps=PCP-030] [scope=crates/nako-playback]
  Goal: Migrate `PlaybackPlanner` to evaluate target profiles and return decision reports with typed reasons.
  Validation: `cargo nextest run -p nako-playback --no-fail-fast`
  Review: Remove or demote shallow codec-list-only semantics.
  Evidence: `crates/nako-playback/src/lib.rs`
  Handoff: Server adapter changes may compile-break until PCP-050 lands.

- [x] PCP-050 [owner=agent] [deps=PCP-040] [scope=crates/nako-server,crates/nako-api,crates/nako-client-protocol]
  Goal: Update server and DTO adapters to construct target profiles and expose safe decision/report fields.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`
  Review: Public Client API must not leak Source Locators, FFmpeg paths, or raw command details.
  Evidence: server playback tests and DTO mappings.
  Handoff: If SDK generation is affected, update the focused generated-sdk gate.

## M3 - Decode-Ready Follow-On Split

- [x] PCP-060 [owner=agent] [deps=PCP-050] [scope=docs/workstreams,docs/adr]
  Goal: Split concrete follow-ons for hardware decode pipeline, subtitle/HDR maturity, and HLS output maturity.
  Validation: follow-on docs or TODO entries exist with explicit gates.
  Review: Do not start FFmpeg breadth in this lane unless required by planner migration.
  Evidence: `docs/workstreams/playback-capability-profile-planner/HANDOFF.md`
  Handoff: First follow-on should be `ffmpeg-hardware-pipeline-planner`.

## M4 - Verification And Closeout

- [x] PCP-070 [owner=planner] [deps=PCP-060] [scope=workspace]
  Goal: Verify, commit, and close or hand off the lane.
  Validation:
  - `cargo nextest run -p nako-playback --no-fail-fast`
  - `cargo nextest run -p nako-server playback --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
  Review: `review-workstream` before closeout.
  Evidence: `docs/workstreams/playback-capability-profile-planner/EVIDENCE_AND_GATES.md`
  Handoff: Record residual risks and next recommended lane.
