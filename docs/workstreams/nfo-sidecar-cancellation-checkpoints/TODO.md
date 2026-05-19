# NFO Sidecar Cancellation Checkpoints - TODO

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Boundary Freeze

- [x] NSCC-010 [owner=planner] [deps=none] [scope=docs/workstreams/nfo-sidecar-cancellation-checkpoints]
  Goal: Open the lane, freeze the NFO per-sidecar cancellation problem, and
  split the first executable slices.
  Validation: `Get-Content docs\workstreams\nfo-sidecar-cancellation-checkpoints\WORKSTREAM.json | ConvertFrom-Json`; `git diff --check`.
  Review: Scope must stay on per-sidecar cooperative cancellation, not retry,
  lease stealing, child-process cancellation, or NFO XML semantics.
  Evidence: `README.md`, `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Result: DONE. Lane opened after `worker-job-cancellation-checkpoints`
  closeout identified NFO service internals as the remaining boundary.
  Handoff: Continue with `NSCC-020`; do not wire server code until the
  `taru-nfo` crate owns a server-independent checkpoint contract.

## M1 - NFO Service Checkpoint Contract

- [ ] NSCC-020 [owner=codex] [deps=NSCC-010] [scope=crates/taru-nfo/src]
  Goal: Add `taru-nfo` sidecar checkpoint types and no-op-compatible
  import/export service variants without depending on `taru-server`.
  Validation: `cargo check -j 2 -p taru-nfo --tests`; `cargo nextest run -j 2 -p taru-nfo nfo_service --no-fail-fast`.
  Review: Checkpoint payload must not include locators, sidecar URIs, XML,
  backup URIs, local paths, storage handles, or raw storage errors.
  Evidence: New or updated `taru-nfo` tests proving existing no-op paths still
  work and cancellation returns a distinct outcome.
  Handoff: `NSCC-030` owns import loop placement after the API shape is stable.

## M2 - Import Per-Sidecar Checkpoints

- [ ] NSCC-030 [owner=codex] [deps=NSCC-020] [scope=crates/taru-nfo/src/import.rs]
  Goal: Check cancellation before each import source sidecar read/parse/commit
  unit and return a cancelled outcome without recording that source as failed.
  Validation: `cargo nextest run -j 2 -p taru-nfo import --no-fail-fast`.
  Review: Partial summaries must not expose XML or sidecar paths; cancellation
  must not be counted as an NFO failure.
  Evidence: Service tests with two sources proving first source can complete
  and second source is skipped when checkpoint returns cancel.
  Handoff: `NSCC-040` mirrors the same contract on export.

## M3 - Export Per-Sidecar Checkpoints

- [ ] NSCC-040 [owner=codex] [deps=NSCC-020] [scope=crates/taru-nfo/src/export.rs]
  Goal: Check cancellation before each export source sidecar stat/read/render/
  write unit and return a cancelled outcome without writing the next sidecar.
  Validation: `cargo nextest run -j 2 -p taru-nfo export --no-fail-fast`.
  Review: Cancellation must not trigger backup/write reports for skipped
  sources and must not change `export_media_source` behavior unexpectedly.
  Evidence: Service tests with two sources proving the next sidecar is not
  written after a cancel checkpoint.
  Handoff: `NSCC-050` wires the crate outcome into server durable jobs.

## M4 - Server Durable NFO Integration

- [ ] NSCC-050 [owner=codex] [deps=NSCC-030,NSCC-040] [scope=crates/taru-server/src/app/nfo.rs,crates/taru-server/src/app/tests/nfo.rs,docs/api/HTTP_API.md]
  Goal: Map `DurableJobContext::check_cancelled()` into the NFO service
  checkpoint API and map NFO cancelled outcomes back to terminal durable
  `cancelled` without success outbox publication.
  Validation: `cargo nextest run -j 2 -p taru-server nfo --no-fail-fast`; `cargo nextest run -j 2 -p taru-server job_cancel --no-fail-fast`.
  Review: Admin/API responses remain redacted and do not expose source
  locators, sidecar URIs, XML, storage handles, or local paths.
  Evidence: Server tests showing import/export background jobs acknowledge
  cancellation before the next sidecar and skip `NfoImported`/`NfoExported`.
  Handoff: `NSCC-060` closes or splits any remaining NFO-specific follow-ons.

## M5 - Closeout

- [ ] NSCC-060 [owner=planner] [deps=NSCC-050] [scope=docs/workstreams/nfo-sidecar-cancellation-checkpoints]
  Goal: Close the lane or split any remaining NFO cancellation edge cases.
  Validation: `verify-rust-workstream` records fresh final gate evidence.
  Review: `review-workstream` has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Follow-ons must stay scoped to NFO behavior and not absorb retry or
  lease policy.
