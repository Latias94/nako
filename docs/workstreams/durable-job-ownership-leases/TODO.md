# Durable Job Ownership Leases - TODO

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] DJOL-010 [owner=planner] [deps=none] [scope=docs/workstreams/durable-job-ownership-leases]
  Goal: Open the durable ownership/lease lane, link ADR/workstream authority,
  and record the initial schema/runtime inventory.
  Validation: `Get-Content docs\workstreams\durable-job-ownership-leases\WORKSTREAM.json | ConvertFrom-Json`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `docs/workstreams/README.md`.
  Handoff: Continue with `DJOL-020`; do not add schema until the state machine
  and ADR impact are explicit.

## M1 - State Machine And Contract Freeze

- [x] DJOL-020 [owner=codex] [deps=DJOL-010] [scope=docs/adr,crates/taru-core/src/job.rs,crates/taru-core/src/repository/jobs.rs]
  Goal: Freeze the job ownership state machine, cancellation semantics,
  terminal-status choice, repository contract names, and safe Admin fields.
  Validation: `cargo check -p taru-core --tests`; `cargo fmt --all -- --check`.
  Review: Confirm no raw job input, summary, error, Source Locator, storage
  handle, path, token, or provider payload becomes part of DTO planning.
  Evidence: `DESIGN.md`, ADR delta if required, core type/repository diffs.
  Result: DONE. `cancelled` is a terminal status distinct from `failed`;
  `JobWorkerId` is diagnostic; `JobRunToken` is the write fence; repository
  lease/cancel methods are present with default unsupported behavior until
  SQLite implements them.
  Handoff: `DJOL-030` owns schema and SQLite tests after this contract is stable.

## M2 - Durable Schema And Repository Proof

- [x] DJOL-030 [owner=codex] [deps=DJOL-020] [scope=crates/taru-db,crates/taru-core]
  Goal: Add the durable columns and SQLite repository operations for fenced
  claim, heartbeat, finish, fail, cancel request, cancellation acknowledgement,
  and lease-aware recovery.
  Validation: `cargo nextest run -p taru-db job_lease --no-fail-fast`; `cargo nextest run -p taru-db job_cancel --no-fail-fast`.
  Review: Every mutating operation must fence on `job_id` plus run token where
  ownership is required.
  Evidence: SQLite migration, repository adapter tests.
  Result: DONE. Added `0029_job_ownership_leases.sql`, fenced claim,
  heartbeat, leased success/failure, durable cancel request, leased cancel
  acknowledgement, expired-lease recovery, and startup preservation of queued
  jobs. Legacy generic startup recovery now fails only running jobs.
  Handoff: `DJOL-040` wires one runtime path to the new contract.

## M3 - First Runtime Integration

- [x] DJOL-040 [owner=codex] [deps=DJOL-030] [scope=crates/taru-core,crates/taru-db,crates/taru-server/src/app/job_runtime.rs]
  Goal: Make one real durable job execution path use leased ownership from
  claim/start through heartbeat and completion.
  Validation: `cargo nextest run -p taru-server job_runtime --no-fail-fast`.
  Review: The runtime supervisor remains process-local; durable truth stays in
  repository operations.
  Evidence: Server runtime tests and startup recovery tests.
  Result: DONE. `DurableJobRuntime::run_job` now exact-claims the queued job,
  persists heartbeats under a stable process worker ID, and completes or fails
  through the run-token fence. The existing library, metadata, and NFO app
  services call this shared runtime path.
  Handoff: `DJOL-050` can add truthful Admin cancel-request controls only after
  this proof exists.

## M4 - Truthful Cancel Request Controls

- [ ] DJOL-050 [owner=codex] [deps=DJOL-040] [scope=crates/taru-api,crates/taru-server/src/http,docs/api]
  Goal: Add redacted Admin cancel-request behavior for leased jobs if the
  worker can observe and acknowledge cancellation.
  Validation: `cargo nextest run -p taru-server job_cancel --no-fail-fast`; `cargo check -p taru-api -p taru-server --tests`.
  Review: Queued cancellation, running cancellation request, terminal-job
  rejection, and expired-lease behavior must be distinct.
  Evidence: HTTP/API tests and docs.
  Handoff: Split if only the repository/runtime half is ready.

## M5 - Closeout Or Split Worker Migrations

- [ ] DJOL-060 [owner=planner] [deps=DJOL-050] [scope=docs/workstreams/durable-job-ownership-leases]
  Goal: Close the lane or split metadata/webhook/NFO/automation/scan worker
  migrations into follow-ons.
  Validation: `verify-rust-workstream` records fresh final gate evidence.
  Review: `review-workstream` has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Remaining migrations must name the leased contract they use.
