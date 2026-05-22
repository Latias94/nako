# Playback Transcode Ops Hardening — TODO

Status: Complete
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

- [x] PTOH-020 [owner=codex] [deps=PTOH-010] [scope=crates/nako-transcode/src/hardware.rs,crates/nako-transcode/src/lib.rs,crates/nako-api/src/admin.rs,crates/nako-api/src/admin_contract.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/tests,apps/admin-web/src/adminApi/generated/contract.ts,apps/admin-web/src/adminApi/mockData.ts]
  Goal: Add or refine a stable Admin-only playback runtime readiness contract
  that classifies FFmpeg probe, hardware acceleration, selected fallback,
  transcode budget, remote playback budget, and staging prerequisites without
  exposing raw paths or command internals.
  Validation: `cargo nextest run -p nako-transcode hardware --no-fail-fast`;
  `cargo nextest run -p nako-server admin_v1_playback_runtime --no-fail-fast`;
  `cargo nextest run -p nako-api admin_playback --no-fail-fast`.
  Review: `review-workstream` before accepting completion.
  Evidence: `HardwareAccelerationReadiness`,
  `AdminPlaybackReadinessDiagnostics`, `GET /admin/v1/playback/runtime`
  tests, and Admin TypeScript contract sync.
  Handoff: PTOH-020 is DONE. Continue with PTOH-030 validation and fallback
  reason hardening.

## M2 — Validation And Fallback Reasons

- [x] PTOH-030 [owner=codex] [deps=PTOH-020] [scope=crates/nako-transcode/src/profile.rs,crates/nako-transcode/src/plan.rs,crates/nako-streaming/src,crates/nako-server/src/app/playback]
  Goal: Validate playback transcode request/profile facts before session
  creation or execution, and replace fragile fallback strings with stable
  reason categories plus redacted operator messages.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-streaming --no-fail-fast`;
  focused playback app tests if server composition changes.
  Review: `review-workstream` must check that validation stays in the narrowest
  owning crate and does not create Public Client API churn.
  Evidence: `TranscodeProfileValidationReason`,
  `TranscodePlanValidationReason`, `PlaybackProfile::try_*_transcode_profile`,
  playback app pre-session validation call sites, and redaction-focused tests.
  Handoff: PTOH-030 is DONE. Continue with PTOH-040 session failure taxonomy.

## M3 — Session Failure Taxonomy

- [x] PTOH-040 [owner=codex] [deps=PTOH-030] [scope=crates/nako-core,crates/nako-api/src/public_client.rs,crates/nako-db/src/tests.rs,crates/nako-db/src/contract_tests.rs,crates/nako-server/src/app/playback,crates/nako-server/src/http/tests]
  Goal: Map playback transcode startup and runtime failures into stable,
  support-oriented categories across probe, plan, staging, budget, runner,
  timeout, cancellation, and hardware fallback boundaries.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-server http::tests::system --no-fail-fast`;
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

- [x] PTOH-050 [owner=codex] [deps=PTOH-040] [scope=crates/nako-api/src/admin.rs,crates/nako-server/src/app/playback,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/tests]
  Goal: Add a bounded Admin-only playback support evidence read model for a
  runtime/session/source context, built from existing session, runtime,
  staging, and hardware evidence without persisting or exporting secrets.
  Validation: `cargo nextest run -p nako-api admin_playback --no-fail-fast`;
  `cargo nextest run -p nako-server http::tests::system --no-fail-fast`;
  `git diff --name-only -- crates/nako-client-protocol`.
  Review: `review-workstream` must check Admin API ownership and redaction.
  Evidence: `GET /admin/v1/playback/support`,
  `AdminPlaybackSupportEvidenceResponse`, support evidence DTO and HTTP
  redaction tests, mismatched session/source context rejection, Admin
  TypeScript contract sync, and unchanged `nako-client-protocol`.
  Handoff: PTOH-050 is DONE. Split retention/export/UI if operators need
  downloadable bundles. Continue with PTOH-060 closeout and parent re-score.

## M5 — Closeout And Parent Re-Score

- [x] PTOH-060 [owner=planner] [deps=PTOH-050] [scope=docs/workstreams/playback-transcode-ops-hardening,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Verify final gates, close or split follow-ons, and re-score
  downloads/watch-folder, network, AI, and addon runtime in the post-RPD
  umbrella.
  Validation: `verify-rust-workstream` records fresh final evidence; final
  workstream JSON and parent umbrella JSON validate with `python -m json.tool`;
  `git diff --check`.
  Review: `review-workstream` must have no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and parent umbrella
  closeout/re-score notes.
  Handoff: DONE. Returned to `post-rpd-product-hardening`; PRPH-120 opened
  `downloads-watch-folder-intake`. Continue with DWI-020. Network remains a
  high-value sidecar; AI and Addon runtime stay downstream consumers of
  accepted Nako-owned boundaries.
