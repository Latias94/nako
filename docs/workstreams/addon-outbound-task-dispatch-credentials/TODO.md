# Addon Outbound Task Dispatch Credentials - TODO

Status: Active
Last updated: 2026-05-24

## M0 - Boundary Freeze

- [x] AOTDC-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-outbound-task-dispatch-credentials,docs/workstreams/addon-task-runtime-contract,docs/workstreams/README.md]
  Goal: Freeze the outbound task-dispatch credential boundary, confirm the
  current direct-dispatch path still passes no credential, and keep the
  storage/resolution follow-on separate from host-owned task scheduling.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json,
  and HANDOFF.md agree on the chosen slice and split follow-ons.
  Review: Confirm the lane only addresses outbound credential storage and
  resolution, not inbound Addon Token auth or Addon Manager lifecycle.
  Evidence: design and code audit notes in EVIDENCE_AND_GATES.md.
  Handoff: DONE. The host-owned task runtime is intact; direct dispatch still
  passes `None` and needs a dedicated credential-management lane for
  `AddonAuth::Bearer` and `AddonAuth::SharedSecret`.

## M1 - Credential Model

- [x] AOTDC-020 [owner=codex] [deps=AOTDC-010] [scope=crates/nako-core,crates/nako-db,crates/nako-server,crates/nako-addon-client,crates/nako-api,docs]
  Goal: Define the first outbound credential storage and resolution model for
  authenticated task dispatch, including a safe API shape for `Bearer` and
  `SharedSecret`.
  Validation: focused compile and test gates, plus docs that describe where the
  credential reference lives and how it resolves.
  Review: Keep the host-owned task runtime boundary intact and do not expose
  raw secret values in public DTOs.
  Evidence: EVIDENCE_AND_GATES.md. DONE. The addon registration record now
  stores `outbound_task_dispatch_secret_env`, the Admin registration request and
  summary surface carry the same env reference, and the server exposes a host
  resolver for dispatch-time lookup.
  Handoff: Continue with direct dispatch injection only after the storage
  model is stable.

## M2 - Direct Dispatch Injection

- [x] AOTDC-030 [owner=codex] [deps=AOTDC-020] [scope=crates/nako-server/src/app/addons/task_runtime.rs,crates/nako-server/src/http/tests,docs]
  Goal: Wire direct task dispatch to the resolved outbound credential and
  prove the headers on redaction-safe tests.
  Validation: focused server tests, addon runtime regression gates, and a
  compile check that exercises the touched crates.
  Review: Confirm missing or unresolved credentials fail safely.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: DONE. Direct dispatch now resolves host-owned outbound credentials
  for bearer and shared-secret addons, injects the matching header, and fails
  safely when the configured secret reference cannot be resolved.

## M3 - Closeout Or Split

- [ ] AOTDC-060 [owner=planner] [deps=AOTDC-030] [scope=docs/workstreams/addon-outbound-task-dispatch-credentials]
  Goal: Close the lane or split any secret-provider follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Summarize residual risks in HANDOFF.md.
