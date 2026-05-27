# Playback Transcode Policy Deepening - TODO

Status: Completed
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

- [x] PTP-020 [owner=codex] [deps=PTP-010] [scope=crates/nako-server/src/http/tests/playback.rs,crates/nako-server/src/app/tests/playback.rs,docs/workstreams/playback-transcode-policy-deepening]
  Goal: Characterize current direct/remux/HLS/Playback Session behavior against
  Jellyfin-class feature pressure before refactoring.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-transcode --no-fail-fast`.
  Review: prove no path/ticket/command leakage and document current gaps.
  Evidence: Added direct-play characterization proving Direct Playback Sessions
  do not create fake Transcode Session artifacts. Focused playback/transcode
  gates passed.
  Handoff: PTP-030 can add planner records without guessing current behavior.

## M2 - Playback Planner

- [x] PTP-030 [owner=codex] [deps=PTP-020] [scope=crates/nako-playback/src,crates/nako-streaming/src,crates/nako-server/src/app/playback,crates/nako-api/src]
  Goal: Add a Playback Planner Module that returns direct/remux/HLS plans and
  typed decision reasons from source facts, client capabilities, runtime facts,
  and policy.
  Validation: focused planner tests plus `cargo nextest run -p nako-server playback --no-fail-fast`.
  Review: HTTP handlers should adapt planner output instead of owning playback
  compatibility decisions.
  Evidence: Added `nako-playback` as the planner/profile/reason crate; reduced
  `nako-streaming` to direct byte-range serving mechanics; routed server
  playback app and public DTO adapters through planner records. Focused
  playback/API/transcode gates passed.
  Handoff: PTP-040 expands capability/reason vocabulary into stable public
  protocol shapes and richer admin-safe diagnostics.

## M3 - Capabilities And Reasons

- [x] PTP-040 [owner=codex] [deps=PTP-030] [scope=crates/nako-client-protocol/src,crates/nako-api/src,sdk/typescript,sdk/kotlin]
  Goal: Add typed Client Playback Capabilities and Playback Decision Reasons
  without copying Jellyfin's DLNA profiles.
  Validation: `cargo nextest run -p nako-client-protocol public --no-fail-fast`;
  `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`.
  Review: Public Client reasons are stable and safe; Admin diagnostics can be
  richer without leaking into public DTOs.
  Evidence: Added `ClientPlaybackDecisionReason` as a stable wire enum and
  `ClientPlaybackCapabilitiesDto` as the shared public capability DTO; public
  DTO adapters map internal planner reasons to protocol reasons; TypeScript and
  Kotlin SDK outputs were regenerated. Protocol/API/server playback gates
  passed.
  Handoff: PTP-050 can consume capability/reason records in transcode policy.

## M4 - Transcode Policy And Acceleration Plan

- [x] PTP-050 [owner=codex] [deps=PTP-030,PTP-040] [scope=crates/nako-transcode/src,crates/nako-core/src,crates/nako-server/src/app/playback]
  Goal: Replace shallow hardware selection with typed decode/filter/encode
  acceleration plans, fallback policy, bitrate/output constraints, and subtitle
  strategy.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  focused `nako-server` hardware/fallback playback tests.
  Review: no `hardware_acceleration: bool` style policy seam; FFmpeg-specific
  strings stay behind the engine Adapter.
  Evidence: Added `TranscodeExecutionPolicy`,
  `TranscodeAccelerationPlan`, output constraints, and subtitle strategy;
  HLS profile/request identity now carries the policy, FFmpeg command planning
  consumes policy behind the adapter boundary, Public Client transcode plans no
  longer expose server hardware selection, and focused transcode/server/API
  gates passed.
  Handoff: PTP-060 can bind policy to runtime inventory and engine execution.

## M5 - Runtime Inventory And Engine Adapter

- [x] PTP-060 [owner=codex] [deps=PTP-050] [scope=crates/nako-transcode/src,crates/nako-server/src/app/playback]
  Goal: Add Playback Runtime Inventory and make FFmpeg CLI the first
  Transcode Engine Adapter behind typed start/cancel/progress semantics.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-server -E 'test(playback) | test(admin_v1_playback_runtime)' --no-fail-fast`.
  Review: inventory output is redaction-safe and execution does not require
  callers to know FFmpeg command strings.
  Evidence: Added `TranscodeRuntimeInventory` and `TranscodeEngineAdapter`
  records; FFmpeg remux/HLS runners now satisfy typed start/progress outcome
  semantics; server playback orchestration calls engine adapters and Admin
  runtime diagnostics consume the inventory summary. Focused transcode/server
  runtime gates passed.
  Handoff: PTP-070 can expose settings/diagnostics without duplicating policy.

## M6 - Admin Settings, Diagnostics, And Artifact Lifecycle

- [x] PTP-070 [owner=codex] [deps=PTP-060] [scope=crates/nako-api/src,crates/nako-server/src/http/admin.rs,crates/nako-server/src/app/playback,crates/nako-db/src]
  Goal: Align Admin playback runtime settings and diagnostics with planner,
  runtime inventory, selected acceleration, throttling, segment cleanup, and
  artifact lifecycle evidence.
  Validation: focused Admin playback runtime/settings tests plus `cargo nextest
  run -p nako-db --no-fail-fast` if storage changes.
  Review: settings are persisted or clearly runtime-only; Public Client surface
  stays redacted and separate.
  Evidence: Added persisted Admin playback runtime settings, runtime
  diagnostics for artifact lifecycle/throttling, startup cleanup for terminal
  transcode artifacts under the transcode root, generated Admin TypeScript
  contract refresh, and focused server/API/storage gates.
  Handoff: PTP-080 can clean playback routes and compatibility wrappers on top
  of persisted settings and lifecycle evidence.

## M7 - Route Cleanup And Closeout

- [x] PTP-080 [owner=codex] [deps=PTP-070] [scope=crates/nako-server/src/http/playback.rs,crates/nako-server/src/app/playback,docs/workstreams/playback-transcode-policy-deepening]
  Goal: Convert playback HTTP routes into thin adapters over planner/session/
  engine Modules and close the lane.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-transcode --no-fail-fast`; `cargo fmt --all -- --check`;
  `git diff --check`; `python -m json.tool docs/workstreams/playback-transcode-policy-deepening/WORKSTREAM.json`.
  Review: canonical route behavior, browser tickets, Playback Session segment
  URLs, and redaction behavior survive the refactor.
  Evidence: Moved direct/remux/HLS playback orchestration into
  `PlaybackAppService`, kept HTTP routes as auth/query/response adapters, added
  coverage proving Transcode Session id segment URLs are rejected, and ran
  playback/transcode closeout gates.
  Handoff: Split adaptive HLS ladders, optimized versions, remote transcode
  workers, desktop player integration, SyncPlay, and DLNA into separate lanes.
