# NFO Sidecar Cancellation Checkpoints - Handoff

Status: Complete
Last updated: 2026-05-19

## Current State

The lane is closed. It followed `worker-job-cancellation-checkpoints`, which
left only the NFO service's per-sidecar loop boundary outside the previous
runtime-level cancellation work.

Completed tasks:

- `NSCC-010`: lane opened and boundary frozen.
- `NSCC-020`: redacted `taru-nfo` checkpoint contract and no-op wrappers.
- `NSCC-030`: import checks before each source sidecar unit.
- `NSCC-040`: export checks before each source sidecar unit.
- `NSCC-050`: server durable import/export cancellation mapping and tests.
- `NSCC-060`: closeout evidence recorded.

## Decisions Since Last Update

- Do not make `taru-nfo` depend on `taru-server`.
- Treat service cancellation as a distinct outcome, not as `NfoFailure` and not
  as a generic `TaruError`.
- Checkpoints stop before the next sidecar source unit; they do not interrupt
  an in-flight storage read/write.
- Keep retry/backoff, lease stealing, and child-process cancellation out of
  this lane.
- Server cancelled outcomes intentionally discard partial summaries and persist
  terminal `cancelled` with no success summary, no error, and no success outbox
  event.

## Blockers

- None.

## Validation

- `cargo check -j 2 -p taru-nfo --tests`: passed.
- `cargo nextest run -j 2 -p taru-nfo nfo_service --no-fail-fast`: passed.
- `cargo nextest run -j 2 -p taru-nfo import --no-fail-fast`: passed.
- `cargo nextest run -j 2 -p taru-nfo export --no-fail-fast`: passed.
- `cargo nextest run -j 2 -p taru-server nfo --no-fail-fast`: passed.
- `cargo nextest run -j 2 -p taru-server job_cancel --no-fail-fast`: passed.
- `cargo check -j 2 -p taru-core -p taru-db -p taru-nfo -p taru-server --tests`:
  passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF working-copy warnings only.

## Next Recommended Action

No immediate follow-on is required from this lane. Future retry/backoff, lease
stealing, child-process cancellation, or richer NFO write policy work should
open separate workstreams.
