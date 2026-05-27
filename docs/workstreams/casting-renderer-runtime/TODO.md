# Casting Renderer Runtime - TODO

Status: Closed
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

- [x] CAST-030 [owner=codex] [deps=CAST-020] [scope=crates/nako-core/src,crates/nako-db/src]
  Goal: Add Renderer Session and command records plus repository adapters where
  persistence is needed.
  Validation: `cargo nextest run -p nako-core renderer --no-fail-fast`;
  `cargo nextest run -p nako-db renderer --no-fail-fast`.
  Review: Renderer Session must stay separate from Playback Session and
  Transcode Session.
  Evidence: Added `RendererSessionId` and `RendererCommandId`, core
  `RendererSession`/`RendererCommand` records, command lifecycle states,
  `RendererSessionRepository`, and durable SQLite/PostgreSQL baseline tables
  plus repository adapters. Contract coverage proves renderer registration,
  heartbeat, playback-session attachment, queued command ordering, claim, and
  terminal completion. Gates passed: core renderer 3 passed/26 skipped; DB
  renderer 1 passed/152 skipped.
  Handoff: CAST-040 can build app-service registration/heartbeat.

## M3 - Nako Remote Client Adapter

- [x] CAST-040 [owner=codex] [deps=CAST-030] [scope=crates/nako-server/src/app,crates/nako-server/src/http,crates/nako-client-protocol/src]
  Goal: Implement Nako-to-Nako renderer registration, heartbeat, capability
  update, and command polling or delivery.
  Validation: `cargo nextest run -p nako-server renderer --no-fail-fast`;
  `cargo nextest run -p nako-client-protocol public --no-fail-fast`.
  Review: Public DTOs must not expose internal principal IDs, raw paths,
  Source Locators, or token material.
  Evidence: Added Public Client renderer routes and DTOs, `RendererAppService`
  owner/TTL/capability/command lifecycle checks, route tests for registration,
  heartbeat, controllable target listing, command polling/completion, and a
  rejection test for external cast protocol registration. OpenAPI and generated
  TypeScript/Kotlin SDK contracts were refreshed so the public route inventory
  stays complete. Gates passed: server renderer 3 passed/358 skipped; server
  playback/renderer integration 79 passed/282 skipped; Public Client protocol
  11 passed; API admin/openapi/sdk 21 passed/39 skipped.
  Handoff: CAST-050 can bind authorized play commands to Playback App Service.

## M4 - Cast Play Command Flow

- [x] CAST-050 [owner=codex] [deps=CAST-040] [scope=crates/nako-server/src/app/playback,crates/nako-server/src/app,crates/nako-server/src/http]
  Goal: Allow an authorized controlling user to send a play command to a Nako
  renderer target and create the correct Playback Session through the existing
  policy-aware playback app service.
  Validation: `cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast`.
  Review: Denied policy/control must not create Playback Sessions, Transcode
  Sessions, or tickets.
  Evidence: Added `CastingAppService` for controller-to-renderer play command
  orchestration, a playback app method that plans against the renderer's
  registered capabilities and enforces `remote_control`/cast/playback policy,
  and `POST /renderers/{renderer_session_id}/commands/play`. The allowed path
  creates a direct-play Playback Session, queues a play command with the
  Playback Session id, and attaches it to the Renderer Session. The denied path
  proves no Playback Session, Transcode Session, ticket, or renderer command is
  created. Gate passed: server playback/renderer 81 passed/282 skipped; Public
  Client protocol 11 passed; API admin/openapi/sdk 21 passed/39 skipped.
  Handoff: CAST-060 can add diagnostics and split external protocol adapters.

## M5 - Diagnostics And External Adapter Follow-Ons

- [x] CAST-060 [owner=codex] [deps=CAST-050] [scope=crates/nako-api/src,crates/nako-server/src/http/admin.rs,docs/workstreams/casting-renderer-runtime]
  Goal: Add redaction-safe Admin diagnostics/readiness and split Chromecast,
  DLNA, and AirPlay follow-ons with concrete adapter contracts.
  Validation: `cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast`;
  `cargo nextest run -p nako-server renderer --no-fail-fast`.
  Review: Diagnostics expose adapter readiness and active renderer state, not
  secrets or private network details.
  Evidence: Added `GET /admin/v1/playback/renderers` with safe runtime
  readiness, session summary, adapter readiness, and generated Admin Web
  contract coverage. Added API/server redaction tests and
  `ADAPTER_FOLLOW_ONS.md` for Nako non-direct transport, Chromecast, DLNA, and
  AirPlay adapter contracts. Gates passed: API admin/openapi/sdk 22 passed/39
  skipped; server renderer 6 passed/358 skipped.
  Handoff: CAST-070 can close the lane or open protocol-specific workstreams.

## M6 - Closeout

- [x] CAST-070 [owner=planner] [deps=CAST-060] [scope=docs/workstreams/casting-renderer-runtime]
  Goal: Verify casting runtime, update evidence, and close/split remaining
  external protocol adapter work.
  Validation: `cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast`;
  `cargo nextest run -p nako-client-protocol public --no-fail-fast`;
  `cargo fmt --all -- --check`;
  `git diff --check`;
  `python -m json.tool docs/workstreams/casting-renderer-runtime/WORKSTREAM.json`.
  Review: `review-workstream` must find no blocking findings.
  Evidence: `CLOSEOUT.md`; `EVIDENCE_AND_GATES.md`; `WORKSTREAM.json`;
  `HANDOFF.md`. Gates passed: server playback/renderer 82 passed/282 skipped;
  Public Client protocol 11 passed; `cargo fmt --all -- --check` passed;
  `git diff --check` passed; `WORKSTREAM.json` parsed.
  Handoff: DONE. Lane closed with protocol-specific follow-ons split in
  `ADAPTER_FOLLOW_ONS.md`.
