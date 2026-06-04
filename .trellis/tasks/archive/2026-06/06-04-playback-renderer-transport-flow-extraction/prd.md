# Playback Renderer Transport Flow Extraction

## Goal

Move renderer playback transport session orchestration out of the broad
`PlaybackAppService` root and into a focused `app/playback/renderer_flow.rs`
boundary. Public renderer command/API behavior must stay unchanged.

This slice continues the feature-backed playback app-service deepening started
by the Remux and HLS flow extractions: the playback app root should expose
thin entrypoints, while mode-specific session startup, transcode linkage, and
renderer transport plan construction live in focused flow modules.

## What I Already Know

* `PlaybackAppService::start_renderer_playback_session` still lives in
  `app/playback/mod.rs` and directly orchestrates Direct, Remux, and HLS
  renderer playback startup.
* Remux playback/session lifecycle now lives in `app/playback/remux_flow.rs`.
* HLS source/playlist/session lifecycle now lives in `app/playback/hls_flow.rs`.
* The renderer flow currently:
  * loads source and probe facts;
  * builds playback selection context;
  * resolves effective playback policy;
  * checks `RemoteControl` permission;
  * calls the playback planner;
  * creates Direct playback sessions;
  * starts Remux or HLS transcode sessions and links playback sessions;
  * cancels superseded HLS playback sessions after HLS renderer startup;
  * returns a redaction-safe renderer transport plan.
* `http/renderer.rs` consumes the resulting `RendererPlaybackTransportPlan`
  to issue renderer transport tickets and URLs.

## Assumptions

* This is a behavior-preserving refactor.
* Public HTTP routes, DTOs, ticket payloads, generated SDKs, and schema stay
  unchanged.
* The first slice should not change renderer command transport fallback logic
  in `http/renderer.rs`.
* Remux and HLS startup should continue to use the existing focused flow
  boundaries instead of duplicating their internals in the renderer flow.

## Requirements

* Keep `PlaybackAppService::start_renderer_playback_session` as a thin
  delegate to `renderer_flow::start_renderer_playback_session`.
* Introduce `app/playback/renderer_flow.rs` for renderer transport session
  orchestration.
* Preserve Direct renderer behavior:
  create a Direct playback session and return the direct content type/range
  support from the planner decision.
* Preserve Remux renderer behavior:
  choose the remux output container from the playback decision, start or reuse
  Remux through the existing Remux flow, create and link a Remux playback
  session, and return the current remux transport plan.
* Preserve HLS renderer behavior:
  start or reuse HLS through the existing HLS flow, create and link an HLS
  playback session, cancel superseded HLS playback sessions, and return the
  current HLS transport plan.
* Preserve renderer `RemoteControl` policy enforcement and playback decision
  denial behavior.
* Do not introduce new route shape, DTO, schema, or ticket format changes.

## Acceptance Criteria

* [ ] `PlaybackAppService::start_renderer_playback_session` delegates directly
      to `renderer_flow::start_renderer_playback_session`.
* [ ] `renderer_flow.rs` owns renderer playback planning and transport plan
      orchestration without duplicating Remux/HLS runner internals.
* [ ] Existing renderer HTTP tests continue to pass.
* [ ] Focused playback app tests around Direct/Remux/HLS startup continue to
      pass where relevant.
* [ ] Server architecture/spec notes describe the new renderer flow boundary if
      the boundary lands.
* [ ] No public API, DTO, schema, generated SDK, or route shape changes are
      introduced.

## Definition of Done

* `cargo fmt --all -- --check` passes.
* `cargo check -p nako-server --tests` passes.
* Focused renderer nextest filter passes.
* Focused playback filters for touched Remux/HLS/Direct paths pass if needed.
* `git diff --check` passes.
* Trellis task context validates.
* Task is archived and the session journal is recorded.

## Technical Approach

Create `crates/nako-server/src/app/playback/renderer_flow.rs` and move the
body of `PlaybackAppService::start_renderer_playback_session` into
`renderer_flow::start_renderer_playback_session`.

Keep the new module as orchestration-only:

* use `nako-playback` for playback decisions;
* use `remux_flow` for Remux session startup;
* use `hls_flow` for HLS playlist startup;
* keep renderer transport URL/ticket authoring in `http/renderer.rs`;
* keep `PlaybackAppService` as the app root and shared helper surface.

If helper access becomes awkward, prefer narrow `pub(super)` helpers over
duplicating logic or widening public crate API.

## Decision (ADR-lite)

**Context**: Renderer playback startup is the next broad playback entrypoint
after Remux and HLS flow extraction. It mixes playback planning, permission
checks, mode-specific session startup, transcode linkage, and transport plan
construction in `mod.rs`.

**Decision**: Extract renderer session/transport orchestration into
`renderer_flow.rs`, while delegating Remux/HLS details to their existing flow
modules and leaving public renderer HTTP behavior unchanged.

**Consequences**: Future renderer-specific playback behavior has a focused
server app module. The app root becomes thinner, but the new module needs
careful boundaries so it does not become a second playback planner or renderer
HTTP ticket author.

## Out of Scope

* Renderer command DTO changes, transport ticket format changes, URL shape
  changes, generated SDK updates, or public API changes.
* New renderer capabilities, device profile behavior, remote-control UX, or
  renderer session lifecycle changes.
* New playback compatibility rules, Remux/HLS FFmpeg planning, or HLS artifact
  URL authority changes.
* Durable queueing, remote transcode workers, LL-HLS/CMAF, or per-device
  transport tuning.

## Technical Notes

* Likely implementation files:
  * `crates/nako-server/src/app/playback/mod.rs`
  * `crates/nako-server/src/app/playback/renderer_flow.rs`
  * `.trellis/spec/nako-server/backend/directory-structure.md`
  * `docs/architecture/PLAYBACK.md`
  * focused renderer/playback tests only if needed.
* Relevant existing flow modules:
  * `crates/nako-server/src/app/playback/remux_flow.rs`
  * `crates/nako-server/src/app/playback/hls_flow.rs`
* Focused gates:
  * `cargo check -p nako-server --tests`
  * `cargo nextest run -p nako-server renderer --no-fail-fast`
  * targeted playback filters if implementation touches Remux/HLS helper
    visibility.
