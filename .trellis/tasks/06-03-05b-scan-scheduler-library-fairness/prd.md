# Scan Scheduler Library Fairness

## Goal

Improve queued library scan scheduling so storage pressure on one class of work
does not unnecessarily block every queued scan when another library could
proceed safely.

## MVP Scope

- Audit durable job lease claim behavior and current scan scheduler admission.
- Add a small scheduling fairness improvement that can skip or defer blocked
  scan work without fail-draining jobs.
- Preserve the existing concurrency budget and durable job runtime boundaries.
- Add tests for blocked remote scan plus runnable local/healthy scan ordering.

## Out of Scope

- No per-backend staging budget model changes; lane A owns that.
- No new DB scheduler table unless the existing lease claim contract cannot
  express the MVP.
- No raw background task outside ADR 0053 runtime boundaries.
- No Public Client API change.

## Acceptance Criteria

- [ ] Scheduler does not claim-and-fail a job only because another queued scan is
  blocked by storage/staging pressure.
- [ ] A runnable queued library scan can proceed while a blocked one remains
  queued or deferred.
- [ ] Durable job state remains redaction-safe and bounded.
- [ ] Tests cover budget saturation, blocked storage admission, and follow-up
  scheduling.

## Suggested Gates

- `cargo check -p nako-server --tests`
- Focused `cargo nextest run -p nako-server <new filters> --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
