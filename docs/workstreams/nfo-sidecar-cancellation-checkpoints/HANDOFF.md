# NFO Sidecar Cancellation Checkpoints - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is active. It follows `worker-job-cancellation-checkpoints`, which
closed with app-level NFO cancellation checkpoints before and after the whole
`NfoService` call, but explicitly split per-sidecar NFO cancellation into this
follow-on.

`NSCC-010` is complete. No code has been changed in this lane yet.

## Active Task

- Task ID: `NSCC-020`
- Owner: codex
- Files:
  - `crates/taru-nfo/src/summary.rs`
  - `crates/taru-nfo/src/import.rs`
  - `crates/taru-nfo/src/export.rs`
  - `crates/taru-nfo/src/lib.rs`
- Validation:
  - `cargo check -j 2 -p taru-nfo --tests`
  - `cargo nextest run -j 2 -p taru-nfo nfo_service --no-fail-fast`
- Status: READY
- Review: The checkpoint contract must remain server-independent and redacted.
- Evidence: New or updated `taru-nfo` tests proving no-op compatibility and a
  distinct cancelled outcome.

## Decisions Since Last Update

- Do not make `taru-nfo` depend on `taru-server`.
- Treat service cancellation as a distinct outcome, not as `NfoFailure` and not
  as a generic `TaruError`.
- Checkpoints stop before the next sidecar source unit; they do not interrupt
  an in-flight storage read/write.
- Keep retry/backoff, lease stealing, and child-process cancellation out of
  this lane.

## Blockers

- None.

## Next Recommended Action

Implement `NSCC-020`: add redacted sidecar checkpoint payload types and
checkpoint-aware import/export service variants with no-op wrappers preserving
existing callers.
