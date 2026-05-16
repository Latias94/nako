# Metadata Operations Milestones

## M13: Maintenance Boundary

Outcome: metadata can be maintained at library scale with bounded provider
runtime visibility and safe diagnostics.

Deliverables:

- `metadata_maintenance` jobs for library or explicit item sets.
- Provider/profile overrides for batch maintenance requests.
- Per-job summaries for attempted, succeeded, failed, no-match, rate-limited,
  and skipped items.
- Provider attempt filtering by provider and status.
- Raw cache cleanup by provider and retention cutoff.
- Provider diagnostics with process-local circuit breaker and failure state.
- Workstream documentation and API contract updates.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`

## M13.1: Maintenance Scheduling

Outcome: operators can schedule recurring metadata maintenance policies without
hand-triggering every batch.

Candidate deliverables:

- Configurable recurring maintenance policies.
- Dry-run planning endpoint.
- Per-library maintenance defaults.
- Backoff for repeatedly failing providers.

## M13.2: Raw Cache Lifecycle

Outcome: raw provider responses have an explicit lifecycle beyond manual cleanup.

Candidate deliverables:

- Startup cleanup option.
- Background cleanup job.
- Retention metrics and deletion summaries.
- Raw cache rebuild flow for selected providers.
