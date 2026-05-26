# Playback Transcode Policy Deepening - TODO

Status: Active
Last updated: 2026-05-27

## M0 - Workstream Open

- [x] PTP-010 [owner=planner] [deps=none] [scope=docs/workstreams/playback-transcode-policy-deepening,docs/adr]
  Goal: Open the playback/transcode policy lane and record the ADR for
  planner/policy/runtime/engine seams.
  Validation: `python -m json.tool docs/workstreams/playback-transcode-policy-deepening/WORKSTREAM.json`;
  `git diff --check -- docs/workstreams/playback-transcode-policy-deepening docs/workstreams/README.md docs/adr`.
  Evidence: `DESIGN.md`; ADR 0038; `WORKSTREAM.json`.
  Handoff: First executable implementation task is PTP-020.

## M1 - Feature Pressure And Characterization

- [ ] PTP-020 [owner=codex] [deps=PTP-010] [scope=crates/nako-server/src/http/tests/playback.rs,crates/nako-server/src/app/tests/playback.rs,docs/workstreams/playback-transcode-policy-deepening]
  Goal: Characterize current direct/remux/HLS/Playback Session behavior against
  Jellyfin-class feature pressure before refactoring.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-transcode --no-fail-fast`.
  Review: prove no path/ticket/command leakage and document current gaps.
  Evidence: tests and an updated evidence log.
  Handoff: PTP-030 can add planner records without guessing current behavior.

## M2 - Playback Planner

- [ ] PTP-030 [owner=codex] [deps=PTP-020] [scope=crates/nako-core/src,crates/nako-server/src/app/playback]
  Goal: Add a Playback Planner Module that returns direct/remux/HLS plans and
  typed decision reasons from source facts, client capabilities, runtime facts,
  and policy.
  Validation: focused planner tests plus `cargo nextest run -p nako-server playback --no-fail-fast`.
  Review: HTTP handlers should adapt planner output instead of owning playback
  compatibility decisions.
  Evidence: planner Module and tests.
  Handoff: PTP-040 expands capability/reason vocabulary.

## M3 - Capabilities And Reasons

- [ ] PTP-040 [owner=codex] [deps=PTP-030] [scope=crates/nako-client-protocol/src,crates/nako-api/src,crates/nako-core/src]
  Goal: Add typed Client Playback Capabilities and Playback Decision Reasons
  without copying Jellyfin's DLNA profiles.
  Validation: `cargo nextest run -p nako-client-protocol public --no-fail-fast`;
  `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`.
  Review: Public Client reasons are stable and safe; Admin diagnostics can be
  richer without leaking into public DTOs.
  Evidence: protocol/API contract tests and regenerated SDKs if needed.
  Handoff: PTP-050 can consume capability/reason records in transcode policy.

## M4 - Transcode Policy And Acceleration Plan

- [ ] PTP-050 [owner=codex] [deps=PTP-030,PTP-040] [scope=crates/nako-transcode/src,crates/nako-core/src,crates/nako-server/src/app/playback]
  Goal: Replace shallow hardware selection with typed decode/filter/encode
  acceleration plans, fallback policy, bitrate/output constraints, and subtitle
  strategy.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  focused `nako-server` hardware/fallback playback tests.
  Review: no `hardware_acceleration: bool` style policy seam; FFmpeg-specific
  strings stay behind the engine Adapter.
  Evidence: transcode policy tests and route behavior parity.
  Handoff: PTP-060 can bind policy to runtime inventory and engine execution.

## M5 - Runtime Inventory And Engine Adapter

- [ ] PTP-060 [owner=codex] [deps=PTP-050] [scope=crates/nako-transcode/src,crates/nako-server/src/app/playback]
  Goal: Add Playback Runtime Inventory and make FFmpeg CLI the first
  Transcode Engine Adapter behind typed start/cancel/progress semantics.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-server -E 'test(playback) | test(admin_v1_playback_runtime)' --no-fail-fast`.
  Review: inventory output is redaction-safe and execution does not require
  callers to know FFmpeg command strings.
  Evidence: adapter tests and diagnostics tests.
  Handoff: PTP-070 can expose settings/diagnostics without duplicating policy.

## M6 - Admin Settings, Diagnostics, And Artifact Lifecycle

- [ ] PTP-070 [owner=codex] [deps=PTP-060] [scope=crates/nako-api/src,crates/nako-server/src/http/admin.rs,crates/nako-server/src/app/playback,crates/nako-db/src]
  Goal: Align Admin playback runtime settings and diagnostics with planner,
  runtime inventory, selected acceleration, throttling, segment cleanup, and
  artifact lifecycle evidence.
  Validation: focused Admin playback runtime/settings tests plus `cargo nextest
  run -p nako-db --no-fail-fast` if storage changes.
  Review: settings are persisted or clearly runtime-only; Public Client surface
  stays redacted and separate.
  Evidence: Admin contract tests and evidence log.
  Handoff: PTP-080 can clean routes and compatibility wrappers.

## M7 - Route Cleanup And Closeout

- [ ] PTP-080 [owner=codex] [deps=PTP-070] [scope=crates/nako-server/src/http/playback.rs,crates/nako-server/src/app/playback,docs/workstreams/playback-transcode-policy-deepening]
  Goal: Convert playback HTTP routes into thin adapters over planner/session/
  engine Modules and close the lane.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-transcode --no-fail-fast`; `cargo fmt --all -- --check`;
  `git diff --check`; `python -m json.tool docs/workstreams/playback-transcode-policy-deepening/WORKSTREAM.json`.
  Review: route compatibility, browser tickets, legacy segment URLs, and
  redaction behavior survive the refactor.
  Evidence: `EVIDENCE_AND_GATES.md`; `HANDOFF.md`; commits.
  Handoff: Split adaptive HLS ladders, optimized versions, remote transcode
  workers, desktop player integration, SyncPlay, and DLNA into separate lanes.

