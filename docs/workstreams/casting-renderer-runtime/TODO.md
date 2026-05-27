# Casting Renderer Runtime - TODO

Status: Active
Last updated: 2026-05-27

## M0 - Workstream Open

- [x] CAST-010 [owner=planner] [deps=none] [scope=docs/workstreams/casting-renderer-runtime,docs/adr]
  Goal: Open the casting renderer runtime lane and record the ADR for Renderer
  Session, Renderer Adapter, command, ticket, and protocol boundaries.
  Validation: `python -m json.tool docs/workstreams/casting-renderer-runtime/WORKSTREAM.json`;
  `git diff --check -- docs/workstreams/casting-renderer-runtime docs/adr/0040-casting-as-renderer-session-adapter.md`.
  Evidence: `DESIGN.md`; ADR 0040; `WORKSTREAM.json`.
  Handoff: `playback-policy-and-renderer-targets` is closed, so CAST-020 can
  start.

## M1 - Readiness And Characterization

- [x] CAST-020 [owner=codex] [deps=CAST-010,PRT-070] [scope=crates/nako-server/src/http/tests/playback.rs,crates/nako-client-protocol/src]
  Goal: Characterize current gaps for renderer registration, remote control,
  playback session handoff, and cast-safe URL transport.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-client-protocol public --no-fail-fast`.
  Review: Prove Playback Session exists but Renderer Session/control does not.
  Evidence: Added protocol route inventory coverage proving Public Client has
  Playback Session routes but no renderer/cast control surface yet. Added HTTP
  playback coverage proving browser playback creates a Playback Session and
  heartbeat state, while session JSON has no renderer session, target, command,
  or supported-command fields. Gates passed: server playback 77 passed/282
  skipped; Public Client protocol 11 passed.
  Handoff: CAST-030 can add renderer-session records.

## M2 - Renderer Session Domain

- [ ] CAST-030 [owner=codex] [deps=CAST-020] [scope=crates/nako-core/src,crates/nako-db/src]
  Goal: Add Renderer Session and command records plus repository adapters where
  persistence is needed.
  Validation: `cargo nextest run -p nako-core renderer --no-fail-fast`;
  `cargo nextest run -p nako-db renderer --no-fail-fast`.
  Review: Renderer Session must stay separate from Playback Session and
  Transcode Session.
  Evidence: Repository and domain tests.
  Handoff: CAST-040 can build app-service registration/heartbeat.

## M3 - Nako Remote Client Adapter

- [ ] CAST-040 [owner=codex] [deps=CAST-030] [scope=crates/nako-server/src/app,crates/nako-server/src/http,crates/nako-client-protocol/src]
  Goal: Implement Nako-to-Nako renderer registration, heartbeat, capability
  update, and command polling or delivery.
  Validation: `cargo nextest run -p nako-server renderer --no-fail-fast`;
  `cargo nextest run -p nako-client-protocol public --no-fail-fast`.
  Review: Public DTOs must not expose internal principal IDs, raw paths,
  Source Locators, or token material.
  Evidence: Route/app tests for register, heartbeat, list controllable targets,
  and command delivery.
  Handoff: CAST-050 can bind play commands to Playback App Service.

## M4 - Cast Play Command Flow

- [ ] CAST-050 [owner=codex] [deps=CAST-040] [scope=crates/nako-server/src/app/playback,crates/nako-server/src/app,crates/nako-server/src/http]
  Goal: Allow an authorized controlling user to send a play command to a Nako
  renderer target and create the correct Playback Session through the existing
  policy-aware playback app service.
  Validation: `cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast`.
  Review: Denied policy/control must not create Playback Sessions, Transcode
  Sessions, or tickets.
  Evidence: Integration tests for allow/deny play command behavior.
  Handoff: CAST-060 can add diagnostics and split external protocol adapters.

## M5 - Diagnostics And External Adapter Follow-Ons

- [ ] CAST-060 [owner=codex] [deps=CAST-050] [scope=crates/nako-api/src,crates/nako-server/src/http/admin.rs,docs/workstreams/casting-renderer-runtime]
  Goal: Add redaction-safe Admin diagnostics/readiness and split Chromecast,
  DLNA, and AirPlay follow-ons with concrete adapter contracts.
  Validation: `cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast`;
  `cargo nextest run -p nako-server renderer --no-fail-fast`.
  Review: Diagnostics expose adapter readiness and active renderer state, not
  secrets or private network details.
  Evidence: Admin/API tests and workstream handoff notes.
  Handoff: CAST-070 closes or opens protocol-specific workstreams.

## M6 - Closeout

- [ ] CAST-070 [owner=planner] [deps=CAST-060] [scope=docs/workstreams/casting-renderer-runtime]
  Goal: Verify casting runtime, update evidence, and close/split remaining
  external protocol adapter work.
  Validation: `cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast`;
  `cargo nextest run -p nako-client-protocol public --no-fail-fast`;
  `cargo fmt --all -- --check`;
  `git diff --check`;
  `python -m json.tool docs/workstreams/casting-renderer-runtime/WORKSTREAM.json`.
  Review: `review-workstream` must find no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`; `WORKSTREAM.json`; `HANDOFF.md`.
  Handoff: External adapter lanes can start from the accepted Renderer Adapter
  contract.
