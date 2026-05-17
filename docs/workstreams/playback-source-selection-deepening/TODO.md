# Playback Source Selection Deepening Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] PSSD-010 [owner=codex] [deps=none] [scope=docs/workstreams/playback-source-selection-deepening]
  Goal: Open M43 with problem, target state, non-goals, task ledger, and
  validation gates.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/playback-source-selection-deepening/DESIGN.md`
  Handoff: Continue with the selection model refactor.

## M1 - Selection Model

- [x] PSSD-020 [owner=codex] [deps=PSSD-010] [scope=crates/taru-streaming/src]
  Goal: Replace the narrow `decide_playback(source, probe, client)` Interface
  with a workflow-shaped playback selection request and decision model while
  preserving current direct-play/remux/transcode behavior.
  Validation: `cargo check -p taru-streaming --tests`; focused
  `cargo nextest run -p taru-streaming --no-fail-fast`.
  passed.
  Evidence: Streaming tests prove container, codec, direct-play-disabled, and
  transcode-output behavior through the new request model. `PlaybackDecision`
  now carries selected-source facts and direct/remux/transcode execution
  intent while retaining compatibility fields for public DTO mapping.
  Handoff: Completed; server playback migration is covered by PSSD-030.

## M2 - Server Playback Migration

- [x] PSSD-030 [owner=codex] [deps=PSSD-020] [scope=crates/taru-server/src/app/playback]
  Goal: Make server playback services load facts and execute decisions while
  moving mode-choice reasoning into `taru-streaming`.
  Validation: `cargo check -p taru-server --tests`; focused playback nextest
  route/app tests.
  passed.
  Evidence: Playback route behavior remains compatible, and server-side logic
  now calls `select_playback_source` with source, probe, client, storage, remux
  output, and HLS transcode intent facts. Remux and HLS execution validate and
  execute the returned decision execution plan.
  Handoff: Completed; public DTO mapping is covered by PSSD-040.

## M3 - Public DTO Compatibility

- [x] PSSD-040 [owner=codex] [deps=PSSD-030] [scope=crates/taru-api/src,crates/taru-client-protocol/src]
  Goal: Keep existing Public Client API playback response shape compatible or
  document and test any deliberate additive field changes.
  Validation: `cargo check -p taru-api --tests`; OpenAPI/SDK contract tests
  if DTOs change.
  passed.
  Evidence: `PlaybackDecisionResponse` mapping is explicit and does not leak
  server-only planning types into `taru-client-protocol`.
  `playback_decision_dto_hides_internal_selection_plan` proves
  `selected_source` and `execution` are not serialized into the Public Client
  API DTO.
  Handoff: Closeout can run workspace gates.

## M4 - Validation And Closeout

- [x] PSSD-050 [owner=codex] [deps=PSSD-040] [scope=workspace,docs]
  Goal: Close M43 with focused and workspace gates, plus follow-on ranking.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  passed.
  Evidence: `EVIDENCE_AND_GATES.md` and `docs/GOALS.md`.
  Handoff: M43 is closed. Recommended next goal is Metadata Provider Attempt
  Runtime Extraction if continuing server architecture cleanup, or `taru-api`
  module split if client/API contract clarity is more urgent.
