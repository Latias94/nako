# Playback Transcode Ops Hardening — TODO

Status: Active
Last updated: 2026-05-22

Task IDs use the `PTOH` prefix.

## M0 — Scope And Evidence Freeze

- [x] PTOH-010 [owner=planner] [deps=none] [scope=docs/workstreams/playback-transcode-ops-hardening,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open the workstream, freeze runtime/diagnostic scope, and connect it
  to the post-RPD umbrella without duplicating completed M7/M25/M56 work.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, HANDOFF.md, and parent umbrella docs agree.
  Evidence: `docs/workstreams/playback-transcode-ops-hardening/DESIGN.md`
  Handoff: Continue with PTOH-020.

## M1 — Runtime Readiness Contract

- [x] PTOH-020 [owner=codex] [deps=PTOH-010] [scope=crates/taru-transcode/src/hardware.rs,crates/taru-transcode/src/lib.rs,crates/taru-api/src/admin.rs,crates/taru-api/src/admin_contract.rs,crates/taru-server/src/http/admin.rs,crates/taru-server/src/http/tests,apps/admin-web/src/adminApi/generated/contract.ts,apps/admin-web/src/adminApi/mockData.ts]
  Goal: Add or refine a stable Admin-only playback runtime readiness contract
  that classifies FFmpeg probe, hardware acceleration, selected fallback,
  transcode budget, remote playback budget, and staging prerequisites without
  exposing raw paths or command internals.
  Validation: `cargo nextest run -p taru-transcode hardware --no-fail-fast`;
  `cargo nextest run -p taru-server admin_v1_playback_runtime --no-fail-fast`;
  `cargo nextest run -p taru-api admin_playback --no-fail-fast`.
  Review: `review-workstream` before accepting completion.
  Evidence: `HardwareAccelerationReadiness`,
  `AdminPlaybackReadinessDiagnostics`, `GET /admin/v1/playback/runtime`
  tests, and Admin TypeScript contract sync.
  Handoff: PTOH-020 is DONE. Continue with PTOH-030 validation and fallback
  reason hardening.

## M2 — Validation And Fallback Reasons

- [x] PTOH-030 [owner=codex] [deps=PTOH-020] [scope=crates/taru-transcode/src/profile.rs,crates/taru-transcode/src/plan.rs,crates/taru-streaming/src,crates/taru-server/src/app/playback]
  Goal: Validate playback transcode request/profile facts before session
  creation or execution, and replace fragile fallback strings with stable
  reason categories plus redacted operator messages.
  Validation: `cargo nextest run -p taru-transcode --no-fail-fast`;
  `cargo nextest run -p taru-streaming --no-fail-fast`;
  focused playback app tests if server composition changes.
  Review: `review-workstream` must check that validation stays in the narrowest
  owning crate and does not create Public Client API churn.
  Evidence: `TranscodeProfileValidationReason`,
  `TranscodePlanValidationReason`, `PlaybackProfile::try_*_transcode_profile`,
  playback app pre-session validation call sites, and redaction-focused tests.
  Handoff: PTOH-030 is DONE. Continue with PTOH-040 session failure taxonomy.

## M3 — Session Failure Taxonomy

- [x] PTOH-040 [owner=codex] [deps=PTOH-030] [scope=crates/taru-core,crates/taru-api/src/public_client.rs,crates/taru-db/src/tests.rs,crates/taru-db/src/contract_tests.rs,crates/taru-server/src/app/playback,crates/taru-server/src/http/tests]
  Goal: Map playback transcode startup and runtime failures into stable,
  support-oriented categories across probe, plan, staging, budget, runner,
  timeout, cancellation, and hardware fallback boundaries.
  Validation: `cargo nextest run -p taru-server playback --no-fail-fast`;
  `cargo nextest run -p taru-server http::tests::system --no-fail-fast`;
  package-specific DB tests only if persisted categories change.
  Review: `review-workstream` must check redaction, persistence compatibility,
  and client error stability.
  Evidence: `TranscodeFailureCategory` support taxonomy,
  redacted persisted playback failure messages, public client coarse-category
  compatibility mapping, DB round-trip/contract coverage, and app/HTTP
  redaction tests.
  Handoff: PTOH-040 is DONE. Continue with PTOH-050 Admin-only support
  evidence read model.

## M4 — Support Evidence Read Model

- [ ] PTOH-050 [owner=unassigned] [deps=PTOH-040] [scope=crates/taru-api/src/admin.rs,crates/taru-server/src/app/playback,crates/taru-server/src/http/admin.rs,crates/taru-server/src/http/tests]
  Goal: Add a bounded Admin-only playback support evidence read model for a
  runtime/session/source context, built from existing session, runtime,
  staging, and hardware evidence without persisting or exporting secrets.
  Validation: `cargo nextest run -p taru-api admin_playback --no-fail-fast`;
  `cargo nextest run -p taru-server http::tests::system --no-fail-fast`;
  `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must check Admin API ownership and redaction.
  Evidence: support evidence tests and docs.
  Handoff: Split retention/export/UI if operators need downloadable bundles.

## M5 — Closeout And Parent Re-Score

- [ ] PTOH-060 [owner=planner] [deps=PTOH-050] [scope=docs/workstreams/playback-transcode-ops-hardening,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Verify final gates, close or split follow-ons, and re-score
  downloads/watch-folder, network, AI, and addon runtime in the post-RPD
  umbrella.
  Validation: `verify-rust-workstream` records fresh final evidence; final
  workstream JSON and parent umbrella JSON validate with `python -m json.tool`;
  `git diff --check`.
  Review: `review-workstream` must have no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and parent umbrella
  closeout/re-score notes.
  Handoff: Return to `post-rpd-product-hardening` with the next lane decision.
