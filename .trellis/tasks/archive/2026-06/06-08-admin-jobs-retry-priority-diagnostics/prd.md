# Admin Jobs Retry And Priority Diagnostics

## Problem

Nako durable jobs already persist safe lifecycle facts such as priority, retry
attempts, retry linkage, and delayed retry time, but the Admin Jobs contract
does not expose them. Operators can see whether a job is queued, running, or
failed, yet they cannot distinguish a normal first attempt from a delayed retry
or a high-priority manual command.

Jellyfin's Scheduled Tasks surface exposes operator-facing lifecycle state for
scheduled work. Nako should not copy that framework directly because the current
control-plane baseline is durable jobs, not an in-memory scheduled-task catalog.
The right next step is to deepen the existing Admin Jobs read model.

## Scope

- Add safe job lifecycle fields to `JobResponse` and `AdminJobListItem`:
  `priority`, `attempt`, `max_attempts`, `retry_of_job_id`, and
  `next_attempt_at`.
- Regenerate Admin Web TypeScript contracts from `nako-api`.
- Render lifecycle facts in Admin Web Jobs without exposing raw job payloads.
- Update redaction and route tests so the new facts are covered.
- Record the contract rule that generic lifecycle facts are safe, while durable
  input JSON, summary JSON, raw errors, storage locators, paths, URI digests,
  etags, fingerprints, credentials, and cache payloads remain forbidden.

## Non-Goals

- Do not add a new scheduled-task framework.
- Do not add automatic VFS repair scheduling.
- Do not expose raw durable job `input_json`, `summary_json`, or `error`.
- Do not expose storage target refs, storage URIs, local paths, backend URLs,
  credentials, etags, fingerprints, URI digests, or cache payload material.
- Do not add retry parameter editing in Admin Web.

## Acceptance Criteria

- Admin Jobs list and job command responses include the five lifecycle fields.
- Source fingerprint hash and VFS cache repair retry route tests assert the
  response lifecycle fields match persisted jobs.
- API serialization tests prove lifecycle fields serialize while raw payloads
  remain absent.
- Admin Web mock data and generated contracts compile against the new fields.
- Jobs UI renders priority, attempt budget, retry source, and delayed retry
  time.
- Admin Web redaction tests still reject unsafe storage and durable payload
  terms.
- Focused Rust and Admin Web validation passes or any unavailable gate is
  recorded with the failure reason.
