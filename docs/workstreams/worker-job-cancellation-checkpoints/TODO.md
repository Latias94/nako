# Worker Job Cancellation Checkpoints - TODO

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] WJCC-010 [owner=planner] [deps=none] [scope=docs/workstreams/worker-job-cancellation-checkpoints]
  Goal: Open the lane, freeze problem/target/non-goals, and record the first
  executable cancellation checkpoint slice.
  Validation: `Get-Content docs\workstreams\worker-job-cancellation-checkpoints\WORKSTREAM.json | ConvertFrom-Json`; `git diff --check`.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Result: DONE. Lane opened after `durable-job-ownership-leases` closeout.
  Handoff: Continue with `WJCC-020`; do not wire worker code before the runtime
  distinguishes cancellation from failure.

## M1 - Runtime Cancellation Context

- [x] WJCC-020 [owner=codex] [deps=WJCC-010] [scope=crates/taru-server/src/app/job_runtime.rs]
  Goal: Add a per-run cancellation context/checkpoint API and make
  `DurableJobRuntime` acknowledge observed cancellation through
  `cancel_leased_job` instead of `fail_leased_job`.
  Validation: `cargo nextest run -p taru-server job_runtime --no-fail-fast`.
  Review: Cancellation must be fenced by the current `JobLeaseGuard`; success
  and failure paths must keep existing behavior.
  Evidence: Runtime tests covering heartbeat-observed cancellation and terminal
  `JobStatus::Cancelled`.
  Result: DONE. Added `DurableJobContext`, cooperative checkpoint helpers,
  `run_job_with_context`, heartbeat publication of observed cancel intent, and
  fenced runtime acknowledgement through `cancel_leased_job`.
  Handoff: `WJCC-030` owns the first real worker integration after the runtime
  contract is stable. Existing `run_job` callers keep success/failure behavior
  until they migrate to context-aware checkpoints.

## M2 - First Real Worker Checkpoints

- [ ] WJCC-030 [owner=codex] [deps=WJCC-020] [scope=crates/taru-server/src/app/metadata.rs]
  Goal: Wire cancellation checkpoints into metadata maintenance before each
  new item refresh so a running Admin cancel request stops the next side-effect
  unit and persists terminal `cancelled`.
  Validation: `cargo nextest run -p taru-server job_cancel --no-fail-fast`; `cargo nextest run -p taru-server metadata --no-fail-fast`.
  Review: Do not emit metadata-maintenance-completed outbox events after a
  cancelled run; do not expose raw job payloads or provider responses.
  Evidence: Server app/HTTP tests showing requested cancellation becomes
  acknowledged cancellation after a checkpoint.
  Handoff: `WJCC-040` broadens or splits remaining worker integrations.

## M3 - Additional Worker Boundaries

- [ ] WJCC-040 [owner=codex] [deps=WJCC-030] [scope=crates/taru-server/src/app/jobs.rs,crates/taru-server/src/app/nfo.rs,docs/api]
  Goal: Add cancellation checkpoints or explicit follow-on notes for library
  scan/probe and NFO import/export without pretending mid-operation rollback.
  Validation: `cargo nextest run -p taru-server job_runtime --no-fail-fast`; targeted package checks for touched modules.
  Review: Checkpoints must sit before new side-effect units, not after success
  events or after irreversible writes.
  Evidence: Tests or documented split decisions for each touched worker.
  Handoff: Remaining job kinds should be split if they require deeper crate
  changes.

## M4 - Closeout Or Split Remaining Worker Migrations

- [ ] WJCC-050 [owner=planner] [deps=WJCC-040] [scope=docs/workstreams/worker-job-cancellation-checkpoints]
  Goal: Close the lane or split retry/backoff, lease-stealing, child-process
  cancellation, and remaining worker migrations.
  Validation: `verify-rust-workstream` records fresh final gate evidence.
  Review: `review-workstream` has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Follow-ons must name whether they need cooperative checkpoints,
  process cancellation, or retry policy.
