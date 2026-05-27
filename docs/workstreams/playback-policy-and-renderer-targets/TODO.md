# Playback Policy And Renderer Targets - TODO

Status: Active
Last updated: 2026-05-27

## M0 - Workstream Open

- [x] PRT-010 [owner=planner] [deps=none] [scope=docs/workstreams/playback-policy-and-renderer-targets,docs/adr]
  Goal: Open the playback policy/renderer target lane and record the ADR for
  policy, target, planner, API, and renderer adapter boundaries.
  Validation: `python -m json.tool docs/workstreams/playback-policy-and-renderer-targets/WORKSTREAM.json`;
  `git diff --check -- docs/workstreams/playback-policy-and-renderer-targets docs/adr/0039-playback-policy-and-renderer-target-boundary.md`.
  Evidence: `DESIGN.md`; ADR 0039; `WORKSTREAM.json`.
  Handoff: First executable implementation task is PRT-020.

## M1 - Current Behavior Characterization

- [x] PRT-020 [owner=codex] [deps=PRT-010] [scope=crates/nako-server/src/http/tests/playback.rs,crates/nako-server/src/app/tests/playback.rs,crates/nako-playback/src/lib.rs]
  Goal: Characterize current playback policy gaps before changing behavior.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-playback --no-fail-fast`.
  Review: Tests must prove current playback only gates on Library Access and
  has no separate direct/remux/transcode/remote/cast policy.
  Evidence: Added Public Client browser-ticket coverage showing a Play-scoped
  viewer can currently request direct/remux/HLS tickets; added app-service
  coverage showing remux starts without principal/policy input; added planner
  coverage showing remote context is profile identity, not a permission gate.
  Targeted and task gates passed.
  Handoff: PRT-030 can add policy records without guessing current behavior.

## M2 - Policy And Target Domain Records

- [x] PRT-030 [owner=codex] [deps=PRT-020] [scope=crates/nako-core/src,crates/nako-playback/src]
  Goal: Add playback permission policy and renderer target records at the
  correct crate boundary.
  Validation: `cargo nextest run -p nako-core playback --no-fail-fast`;
  `cargo nextest run -p nako-playback --no-fail-fast`.
  Review: Records must be playback-shaped, not copied Jellyfin `UserPolicy` or
  DLNA `DeviceProfile` shapes.
  Evidence: Added `PlaybackPermissionPolicy`, `EffectivePlaybackPolicy`,
  `PlaybackPermission`, target kind/network/transport/control vocabulary in
  `nako-core`, plus `PlaybackTarget` records in `nako-playback`. Unit tests
  cover current playback defaults, mode-specific denial reasons, admin
  control/cast defaults, cast target vocabulary, command capabilities, and
  browser/Nako remote target transport separation. PRT-030 gates passed.
  Handoff: PRT-040 can feed policy and targets into planner decisions.

## M3 - Planner Enforcement

- [ ] PRT-040 [owner=codex] [deps=PRT-030] [scope=crates/nako-playback/src,crates/nako-api/src]
  Goal: Make playback planning consume effective policy and renderer target
  capabilities, returning typed allow/deny decisions and safe public reasons.
  Validation: `cargo nextest run -p nako-playback --no-fail-fast`;
  `cargo nextest run -p nako-api public --no-fail-fast`.
  Review: Planner must not query repositories or depend on HTTP/server auth.
  Evidence: Planner tests for direct denied, remux denied, audio/video transcode
  denied, remote denied, cast denied, and compatible allowed playback.
  Handoff: PRT-050 can wire server effective policy resolution.

## M4 - Server App And HTTP Integration

- [ ] PRT-050 [owner=codex] [deps=PRT-040] [scope=crates/nako-server/src/app/playback,crates/nako-server/src/http/playback.rs,crates/nako-server/src/http/access.rs]
  Goal: Resolve effective playback policy in the server app and enforce it
  before creating Playback Sessions, Transcode Sessions, or browser tickets.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`.
  Review: HTTP routes remain auth/query/ticket/response adapters; app service
  owns policy-aware playback orchestration.
  Evidence: Route/app tests proving denied modes do not create sessions or
  artifacts.
  Handoff: PRT-060 can expose safe API/Admin surfaces.

## M5 - API And Diagnostics

- [ ] PRT-060 [owner=codex] [deps=PRT-050] [scope=crates/nako-client-protocol/src,crates/nako-api/src,apps/admin-web/src/adminApi/generated/contract.ts]
  Goal: Add safe Public Client target/capability/denial DTOs and Admin
  diagnostics for effective playback policy readiness.
  Validation: `cargo nextest run -p nako-client-protocol public --no-fail-fast`;
  `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`.
  Review: Public DTOs expose effective outcomes, not raw policy rows or role
  internals.
  Evidence: Protocol/API tests and generated contract diffs.
  Handoff: PRT-070 can close or split persistent policy editing as needed.

## M6 - Closeout

- [ ] PRT-070 [owner=planner] [deps=PRT-060] [scope=docs/workstreams/playback-policy-and-renderer-targets,docs/workstreams/casting-renderer-runtime]
  Goal: Verify the lane, update evidence, and hand off to casting renderer
  runtime.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-playback --no-fail-fast`;
  `cargo fmt --all -- --check`;
  `git diff --check`;
  `python -m json.tool docs/workstreams/playback-policy-and-renderer-targets/WORKSTREAM.json`.
  Review: `review-workstream` must find no blocking workstream or code-quality
  issues before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`; `WORKSTREAM.json`; HANDOFF update.
  Handoff: Start CAST-020 after PRT closeout unless a blocker is recorded.
