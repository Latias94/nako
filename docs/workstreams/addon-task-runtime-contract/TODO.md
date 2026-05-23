# Addon Task Runtime Contract - TODO

Status: Completed
Last updated: 2026-05-23

## M0 - Boundary Freeze

- [x] ATRC-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-task-runtime-contract,docs/adr,docs/workstreams]
  Goal: Freeze the Addon Task runtime problem, target state, non-goals, and
  first runtime slice without folding source catalog, package signing, or
  process supervision into the contract.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json,
  and HANDOFF.md agree on the selected slice and split follow-ons.
  Review: Confirm the task runtime boundary stays separate from manifest
  declaration samples such as `bulk-metadata-scrape`.
  Evidence: docs/workstreams/addon-task-runtime-contract/DESIGN.md
  Handoff: First executable task is ATRC-020.
  Progress: Done 2026-05-23. Boundary frozen around host-owned execution,
  progress, result, cancellation, and retry; source catalog, package signing,
  provider breadth, and process supervision remain follow-ons.

## M1 - Run Model

- [x] ATRC-020 [owner=codex] [deps=ATRC-010] [scope=crates/nako-server,crates/nako-api,docs]
  Goal: Define the first host-owned Addon Task run model and the minimal
  execution/progress/result surface for one task declaration.
  Validation: task runtime docs and API shapes are explicit enough that the
  first execution/progress/result slice can be implemented without revisiting
  the boundary split.
  Review: Keep catalog discovery, package signing, provider breadth, and
  process/container supervision out of the runtime bootstrap slice.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: Continue with the first execution/progress/result surface only after
  the run model is stable.
  Progress: Done 2026-05-23. Added `AddonTaskRun` domain/repository contracts,
  `addon_task_runs` SQLite/PostgreSQL storage, redaction-safe DTOs, and
  host-owned Admin/runtime route shapes.

## M2 - Runtime Surface

- [x] ATRC-030 [owner=codex] [deps=ATRC-020] [scope=crates/nako-server,crates/nako-api,docs]
  Goal: Implement the smallest task runtime surface that can accept, track, and
  complete one Addon Task run with explicit progress, result, and cancellation
  semantics.
  Validation: focused server tests plus a repeatable smoke or docs workflow for
  one host-owned task run.
  Review: Confirm the runtime does not absorb package signing, source catalog,
  or process/container supervision.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: Keep progress/result handling bounded to the runtime surface.
  Progress: Done 2026-05-23. Implemented create, list, get, claim, progress,
  complete, fail, cancel acknowledgement, and retry for one host-owned Addon
  Task run surface.

## M3 - Closeout Or Split

- [x] ATRC-040 [owner=codex] [deps=ATRC-030] [scope=crates/nako-addon-protocol,crates/nako-addon-client,crates/nako-server,crates/nako-api,docs]
  Goal: Layer direct Addon Sidecar task-path dispatch on top of the
  host-owned Addon Task run model without moving declaration, scheduling,
  retry, progress, result, or cancellation ownership into the sidecar.
  Validation: focused addon-client envelope test, focused server direct
  dispatch tests, task-run regression tests, and compile gate.
  Review: Confirm direct dispatch claims the specific target run and stays
  separate from Addon Source Catalog discovery and declaration routing.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: Authenticated outbound task dispatch remains a credential-management
  follow-on unless the lane adds sidecar outbound secret storage.
  Progress: Done 2026-05-23. Added task request/response envelopes, direct
  dispatch mode, target-job claim filtering, direct success/failure/retry/cancel
  handling, and focused HTTP tests.

- [x] ATRC-060 [owner=planner] [deps=ATRC-040] [scope=docs/workstreams/addon-task-runtime-contract]
  Goal: Close the lane or split marketplace/package-signing/provider-breadth
  and process-supervision follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: DONE. The host-owned Addon Task runtime contract is complete.
  Authenticated outbound task dispatch credential management, official-addon
  task-path smoke coverage, source catalog / marketplace discovery, package
  signing, provider breadth, and process/container supervision are follow-ons.
  Progress: Done 2026-05-23. Closed the lane after final docs, format, diff,
  compile, and addon runtime gates.
