# Source Hash Retry Requeue Admin Command

## Goal

Expose a source-fingerprint-hash-specific Admin retry command so operators can
retry failed Source Fingerprint hash jobs without seeing or resubmitting raw job
input, Source Locators, paths, hashes, fingerprints, etags, backend URLs, or
credentials.

## Requirements

- Add an Admin DTO for retrying a failed Source Fingerprint hash job.
- Add an Admin route:
  `POST /admin/v1/source-fingerprint-hashes/jobs/{job_id}/retry`.
- Require the existing Admin route guard.
- Delegate to `SourceFingerprintHashAppService` rather than calling generic job
  retry logic directly from HTTP.
- Reuse durable `enqueue_job_retry` semantics:
  - create a new queued retry job;
  - preserve the failed job as audit history;
  - copy the safe source-hash job input, resource class, priority, library id,
    and source id;
  - set `retry_of_job_id` to the failed source job;
  - increment the retry attempt number.
- Default retry behavior when the request omits `max_attempts`:
  `max(source.max_attempts, source.attempt + 1)`.
- Accept optional `next_attempt_at` so an operator can delay the retry; delayed
  retries remain unclaimable until due through existing durable job logic.
- Return the existing redaction-safe `AdminJobListItem` shape with `202
  Accepted`.
- Reject jobs that are not `JobKind::SourceFingerprintHash`, have an unsupported
  resource class, are not failed, have exhausted attempts, or have malformed /
  unsafe source hash input.
- Preserve Admin Jobs filtering as the drilldown surface for the failed and
  retry jobs.
- Keep the command out of Public Client contracts.

## Acceptance Criteria

- [ ] Admin can retry a failed source fingerprint hash job and receives a new
      queued retry job.
- [ ] Retry job preserves `library_id`, `source_id`, resource class, priority,
      and safe job input, while setting `retry_of_job_id`.
- [ ] The original failed job remains failed and visible for audit.
- [ ] Delayed retries are summarized in source hash overview as delayed retry
      jobs and are not claimable until due.
- [ ] The scheduler can execute an immediate retry through the existing
      disk-scan source-hash path.
- [ ] Non-admin callers are rejected.
- [ ] Wrong-kind, wrong-resource, non-failed, exhausted, malformed-input, and
      unsafe-input cases fail without leaking job input, Source Locators, paths,
      raw hashes, raw fingerprints, etags, backend URLs, credentials, or raw
      error bodies.
- [ ] Admin contract route inventory covers the new route or explicitly
      excludes it.

## Definition Of Done

- `cargo check -p nako-api -p nako-server --tests` passes.
- Focused `cargo nextest` gates for source hash app and HTTP behavior pass.
- Admin contract and server route inventory gates pass.
- `cargo fmt --all -- --check`, `git diff --check`, and Trellis task validation
  pass.
- Source hash Trellis spec records the retry command contract.

## Technical Approach

- Add `AdminSourceFingerprintHashRetryRequest` in `nako-api`, with optional
  `max_attempts` and `next_attempt_at`.
- Add TypeScript contract definitions and Admin route inventory entry for
  `sourceFingerprintHashJobRetry`.
- Add `RetrySourceFingerprintHashRequest` to `nako-server::app::source_hash`.
- Implement `SourceFingerprintHashAppService::retry_source_fingerprint_hash_job`
  by loading the job, validating source hash job kind/resource/input/bindings
  with existing source hash validators, calculating the effective max attempts,
  and calling `JobRepository::enqueue_job_retry`.
- Add `POST /admin/v1/source-fingerprint-hashes/jobs/{job_id}/retry` in
  `http/admin.rs`, returning `AdminJobListItem`.
- Add focused app-service and route tests for success, scheduling, auth,
  redaction, and invalid states.

## Decision (ADR-lite)

**Context**: Source hash failures are visible through Admin Jobs and overview
pressure, but operators currently need a safe way to retry a failed hash without
copying raw durable job input or forcing a brand-new source-hash enqueue.

**Decision**: Use a source-hash-specific Admin retry command that wraps the
existing durable job retry primitive. The retry creates a new job for audit
continuity rather than resetting the failed job in place.

**Consequences**: Operators get an explicit recovery command and the scheduler
continues to use the existing disk-scan source-hash path. A later generic Admin
job retry command can still be designed, but this slice keeps source-hash
redaction and job-contract validation local to the source hash app service.

## Out Of Scope

- No automatic retry policy or scheduler-driven retry creation.
- No source hash deduplication policy.
- No scan-originated source hash enqueue.
- No automatic Source Duplicate Relationship mutation.
- No job detail endpoint exposing raw durable input, summary, or error payloads.
- No Admin Web UI changes beyond generated contract output.
- No database schema migration.

## Technical Notes

- General durable retry already exists as
  `JobRepository::enqueue_job_retry(EnqueueJobRetry)`.
- Source hash durable input is `SourceFingerprintHashJobInput`; it stores
  library/source ids, source scheme, and mode, not the Source Locator.
- Existing source hash scheduler executes queued source hash jobs through
  `LibraryScanAppService::schedule_queued_library_scans`.
- Reuse the existing redacted `AdminJobListItem` response shape.
- Relevant specs:
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`,
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`.
