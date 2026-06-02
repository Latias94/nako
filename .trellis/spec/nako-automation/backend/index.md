# nako-automation Backend Guidelines

`nako-automation` owns automation provider configuration, durable automation
jobs, provider execution, cancellation, and automation artifacts. It must not
let external automation mutate canonical metadata directly.

## Current Evidence

- `crates/nako-automation/src/lib.rs`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/adr/0053-control-plane-runtime-baseline.md`

## Boundaries

- Use `AutomationProvider` for provider execution abstraction.
- Use `AutomationJobService` for enqueue and one-shot execution workflow.
- Enqueue durable `JobKind::Automation` jobs.
- Use resource class `automation.external_api` for external provider work.
- Keep canonical metadata acceptance in metadata/catalog workflows.

## Required Patterns

- Validate provider enabled state and capability before enqueueing.
- Execute provider work through a bounded timeout.
- Pass cancellation state into provider execution.
- Persist automation artifacts for provider outcomes.
- Reject outcomes marked as directly accepted into canonical metadata.

## Forbidden Patterns

- Do not call automation providers without a durable job.
- Do not log provider secrets.
- Do not let automation results bypass canonical acceptance workflows.
- Do not create raw background tasks outside the control-plane model.

## Validation

- Focused:
  `cargo nextest run -p nako-automation --no-fail-fast`
- Control-plane compile:
  `cargo check -p nako-automation -p nako-core --tests`
