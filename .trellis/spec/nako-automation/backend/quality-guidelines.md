# Quality Guidelines

Automation changes must keep external work durable, cancellable, and separated
from canonical metadata mutation.

## Required Patterns

- Test enqueue validation for enabled state and capability.
- Test run-once workflow with fake providers and fake job repositories.
- Keep provider execution timeout behavior deterministic in tests.
- Pass cancellation state into providers.
- Verify artifacts are persisted only for accepted provider outcomes.

## Forbidden Patterns

- Do not add fire-and-forget provider calls.
- Do not accept provider output directly into canonical metadata.
- Do not log or store secrets while testing failure paths.
- Do not make tests depend on real external APIs.

## Tests Required

- Enqueue success and failure tests.
- Provider success, failure, timeout, and cancellation tests.
- Max-attempt tests.
- Artifact persistence tests.
- Direct canonical acceptance rejection tests.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-automation --no-fail-fast`
- Control-plane compile:
  `cargo check -p nako-automation -p nako-core --tests`
