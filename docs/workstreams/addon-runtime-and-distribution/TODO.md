# Addon Runtime And Distribution — TODO

Status: Active
Last updated: 2026-05-22

Task IDs use the `ARD` prefix.

## M0 — Scope And Evidence Freeze

- [x] ARD-010 [owner=planner] [deps=AILO-050] [scope=docs/workstreams/addon-runtime-and-distribution,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open the Addon Runtime / Distribution lane with sidecar package,
  install-guide, runtime-readiness, task/event/artifact routing, non-goal, and
  follow-on boundaries frozen.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, HANDOFF.md, parent umbrella, and workstream index agree.
  Evidence: `docs/workstreams/addon-runtime-and-distribution/DESIGN.md`.
  Handoff: Continue with ARD-020.

## M1 — Package And Install Guide Boundary

- [x] ARD-020 [owner=codex] [deps=ARD-010] [scope=crates/taru-addon-protocol,crates/taru-api/src/extension.rs,crates/taru-server/src/app,crates/taru-server/src/http/addons.rs]
  Goal: Define a stable Addon package/install descriptor and validation model
  that can summarize sidecar image/binary/runtime requirements, manifest facts,
  Secret Reference needs, grants, network exposure prerequisites, and generated
  install-guide snippets without storing or exposing secrets.
  Progress: Added `AddonInstallDescriptor`, runtime reference validation,
  Secret Reference binding validation, redacted `AddonInstallGuide`
  generation, Admin preview DTOs, `POST
  /admin/v1/addons/install-guide-preview`, app validation, and redaction tests.
  Validation: focused `taru-addon-protocol` tests; focused Admin DTO/server
  tests for redacted install-guide previews; `cargo fmt --all -- --check`;
  `git diff --check`; `git diff --name-only -- crates/taru-client-protocol`.
  Evidence: `cargo nextest run -p taru-addon-protocol --no-fail-fast`; `cargo
  nextest run -p taru-server admin_addon_install_guide_preview
  --no-fail-fast`; `cargo nextest run -p taru-server addons --no-fail-fast`;
  `cargo nextest run -p taru-api --no-fail-fast`; `cargo fmt --all -- --check`;
  `git diff --check`; `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must check permissive protocol boundaries,
  no server-internal dependencies in protocol crates, no admin-token leakage,
  and no Addon Manager/process-supervision behavior.
  Handoff: Add runtime readiness diagnostics in ARD-030.

## M2 — Runtime Readiness And Sidecar Compatibility

- [x] ARD-030 [owner=codex] [deps=ARD-020] [scope=crates/taru-addon-client,crates/taru-server/src/app,crates/taru-api/src/extension.rs,crates/taru-server/src/http/addons.rs,apps/admin-web/src/adminApi]
  Goal: Add Admin-only runtime readiness diagnostics that classify sidecar
  reachability, protocol version mismatch, manifest mismatch, grant gaps,
  missing Secret References, network policy blockers, and unsafe response
  conditions without echoing raw network errors or sidecar payloads.
  Progress: Added `AdminAddonRuntimeReadinessResponse`, Admin contract/client
  support, and `POST /admin/v1/addons/{addon_id}/runtime-readiness`.
  Readiness checks now classify grants, Secret Reference configuration gaps,
  network policy blockers, sidecar OK/degraded/unhealthy status, protocol
  mismatch, manifest mismatch, transport/HTTP failures, and unsafe sidecar
  payloads with safe error codes only.
  Validation: focused addon-client/app/Admin tests; `cargo nextest run -p
  taru-server addons --no-fail-fast`; `cargo nextest run -p taru-api
  admin_contract --no-fail-fast`; `npm run check` from `apps/admin-web`;
  `git diff --name-only -- crates/taru-client-protocol`.
  Review: check Admin boundary ownership, redaction, timeout/resource policy,
  and separation from automatic process/container supervision.
  Evidence: `cargo nextest run -p taru-server admin_addon_runtime_readiness
  --no-fail-fast`; `cargo nextest run -p taru-server addons --no-fail-fast`;
  `cargo nextest run -p taru-api admin_contract --no-fail-fast`; `npm run
  check`; `npm test -- src/adminApi/client.test.ts`; generated Admin Web
  contract sync.
  Handoff: Route task/event declarations in ARD-040.

## M3 — Declared Task/Event Routing Without Hidden Schedulers

- [ ] ARD-040 [owner=codex] [deps=ARD-030] [scope=crates/taru-core,crates/taru-db,crates/taru-server/src/app,crates/taru-server/src/http/admin.rs]
  Goal: Turn manifest-declared Addon Tasks and Event Subscriptions into
  Taru-owned routing plans: executable only where existing job/outbox/addon
  side-effect boundaries can own lifecycle, otherwise blocked with explicit
  deferred reasons.
  Validation: focused app/db tests for routing plans, idempotency, stale
  manifest checks, and no hidden scheduler/event delivery side effects; relevant
  Admin/system tests; `cargo fmt --all -- --check`; `git diff --check`.
  Review: check durable audit, target revalidation, no background scheduler
  hidden in this lane, and no direct Addon filesystem/library authority.
  Evidence: routing-plan tests and Admin diagnostics.
  Handoff: Add artifact/intake handoff in ARD-050.

## M4 — Addon Artifact And Intake Handoff

- [ ] ARD-050 [owner=codex] [deps=ARD-040] [scope=crates/taru-server/src/app,crates/taru-core,crates/taru-db,crates/taru-api/src/admin.rs]
  Goal: Prove Addon-produced Generated Artifacts and acquisition candidates
  enter existing AILO/DWI proposal/intake boundaries rather than creating direct
  Canonical Metadata, NFO sidecar, Media Source, Managed Import, or library-file
  writes.
  Validation: focused app/db tests for Addon artifact/intake handoff and
  stale-target checks; relevant Admin/system tests; `cargo fmt --all
  -- --check`; `git diff --check`; `git diff --name-only --
  crates/taru-client-protocol`.
  Review: check acceptance/audit routing, redaction, and no direct canonical
  or library mutation.
  Evidence: Addon-to-Generated-Artifact and Addon-to-Acquisition-Intake tests.
  Handoff: Close or split Addon Manager/process supervision in ARD-060.

## M5 — Closeout And Follow-On Split

- [ ] ARD-060 [owner=planner] [deps=ARD-050] [scope=docs/workstreams/addon-runtime-and-distribution,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Verify final gates, close or split Addon Manager discovery/install,
  package signing, marketplace, process supervision, logs, rollback, Native
  Plugin ABI, downloader protocols, Public Client surfaces, and local AI/model
  runtime follow-ons, then return the next lane decision to the post-RPD
  umbrella.
  Validation: `verify-rust-workstream` records fresh final evidence; workstream
  JSON and parent umbrella JSON validate with `python -m json.tool`; `git diff
  --check`; `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must have no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and parent umbrella
  re-score notes.
  Handoff: Return to `post-rpd-product-hardening` with the next lane decision.
