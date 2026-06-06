# Scan-Originated Source Hash Triggering

## Goal

Add the first scan-originated source fingerprint hash trigger so a successful
library scan source commit can enqueue policy-backed Source Fingerprint Hash
durable jobs through the existing source hash app service and disk-scan
scheduler path.

This closes the current M1 operator journey gap between scan/index and
source-identity diagnostics: operators should no longer need to manually enqueue
every hash job when scan already produced a partial or full hash escalation
decision.

## What I Already Know

- Source fingerprint hash execution, durable input, scheduler integration,
  evidence persistence, Admin manual enqueue, Admin retry/requeue, Admin Jobs
  diagnostics, duplicate reconciliation plan, and explicit duplicate
  reconciliation apply are already shipped as separate slices.
- `nako-library::ingestion` already owns advisory
  `SourceFingerprintEscalationDecision` planning.
- `nako-library::source_hash` already owns the safe
  `SourceFingerprintHashRequest`, `SourceFingerprintHashJobInput`, hash
  executor, and job summary contracts.
- `nako-server::app::source_hash` already owns safe enqueue/execute/retry
  orchestration for Source Fingerprint Hash jobs.
- Existing architecture/spec guidance says scan-originated scheduling must run
  after a successful scan source commit, be policy-backed, idempotent, visible
  through Admin jobs, and must not perform VFS reads inside pure planning.

## Requirements

- Add a server-side scan-originated source hash enqueue boundary that runs only
  after a successful scan source commit or equivalent committed scan result.
- Use the existing advisory escalation decision:
  - `none` does not enqueue work;
  - partial/full decisions enqueue matching partial/full source hash jobs when
    automatic scan-originated scheduling policy is enabled;
  - disabled policy keeps the decision advisory/diagnostic-only.
- Reuse `SourceFingerprintHashAppService::enqueue_source_fingerprint_hash` or a
  narrow wrapper around it as the only durable enqueue authority.
- Persist queued work as existing `JobKind::SourceFingerprintHash` jobs with
  `disk.scan.source_fingerprint_hash`, safe `SourceFingerprintHashJobInput`,
  library/source bindings, and optional priority only if the existing policy
  exposes it.
- Make enqueue idempotent for the same source/mode when an equivalent queued,
  running, or delayed retry source hash job already exists. The implementation
  may return the existing job or record a skipped/idempotent outcome, but must
  not silently create duplicate queued hash work.
- Keep source hash execution on the existing disk-scan scheduler path. Do not
  add a source-hash-specific runtime loop or raw `tokio::spawn`.
- Keep all diagnostics and errors redaction-safe: no Source Locators, local
  paths, backend URLs, etags, credentials, raw digests, raw fingerprints, hash
  material, or durable input JSON in operator-facing surfaces.
- Ensure Admin Jobs / source hash diagnostics can observe scan-originated jobs
  through existing job list/overview facts; add only the minimal source-origin
  fact if current surfaces cannot distinguish manual versus scan-originated
  pressure without exposing unsafe input.
- Preserve hash completion behavior: persist redacted fingerprint evidence only.
  Do not write `SourceDuplicateRelationship` records from scan commit or hash
  completion.

## Acceptance Criteria

- [ ] A committed scan result with `SourceFingerprintEscalationDecision::None`
      does not enqueue a source hash job.
- [ ] A committed scan result with partial escalation enqueues exactly one
      partial Source Fingerprint Hash durable job when scan-originated policy is
      enabled.
- [ ] A committed scan result with full escalation enqueues exactly one full
      Source Fingerprint Hash durable job when scan-originated policy is
      enabled.
- [ ] Disabled automatic policy leaves partial/full escalation advisory-only and
      creates no job.
- [ ] Reprocessing the same committed source/mode while an equivalent queued,
      running, or delayed retry job exists is idempotent and does not create
      duplicate queued work.
- [ ] Scan-originated jobs are visible through the existing Admin Jobs/source
      hash diagnostic surfaces without leaking unsafe source details.
- [ ] Queued scan-originated jobs execute through
      `LibraryScanAppService::schedule_queued_library_scans` under the existing
      `disk.scan` budget path.
- [ ] Hash execution success still persists only redacted source fingerprint
      evidence and redacted job summary.
- [ ] No duplicate relationship is created, updated, confirmed, or merged by
      scan commit or hash completion.
- [ ] Focused tests cover success, disabled policy, idempotency, scheduler
      execution, and redaction.

## Definition Of Done

- Implementation and independent check complete.
- Focused Rust gates pass for touched source hash, scan, durable job, and API
  surfaces.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Trellis task validation passes.
- Task evidence records commands, results, changed files, and residual gaps.
- Spec/architecture updates are completed if the implementation changes
  durable contracts or operator-visible behavior.

## Out Of Scope

- No automatic Source Duplicate Relationship reconciliation.
- No Media Source merge, Media Item merge, Playback Source Selection mutation,
  or Library Access mutation.
- No new Public Client API route.
- No broad Admin Web operator flow beyond existing job/diagnostic visibility.
- No source-hash-specific runtime loop.
- No schema migration unless existing durable-job idempotency queries cannot
  express the required duplicate prevention safely.
- No VFS reads during scan planning or source commit.

## Technical Approach

Add a small app-service boundary in `nako-server` that accepts committed scan
source facts plus the advisory source fingerprint escalation decision and maps
eligible decisions to existing `EnqueueSourceFingerprintHashRequest` calls.

The safest implementation shape is:

- keep pure scan/source observation planning in `nako-library::ingestion`;
- call the scan-originated enqueue boundary after the source record has been
  committed and has a stable `MediaLibraryId` / `MediaSourceId`;
- use existing source hash app-service validation to reload the current
  `MediaSource`, verify library ownership, derive only source scheme, and build
  safe durable input;
- add a repository/app-service idempotency check for same source/mode queued or
  active work before enqueueing;
- route execution through the already shipped disk-scan scheduler.

## Decision

Use the existing durable job and disk-scan scheduler path. Scan commit becomes a
trigger decision point only; it does not hash bytes, execute jobs inline, or
reconcile duplicates.

Consequences:

- M1 scan/index can naturally produce hash evidence work while preserving
  control-plane resource accounting.
- Operators can still inspect/retry work through the existing Admin Jobs and
  source hash surfaces.
- Duplicate reconciliation remains a separate explicit plan/apply workflow with
  its own idempotency and undo boundaries.

## Technical Notes

- Architecture references:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/CONTROL_PLANE.md`
- Source hash policy spec:
  - `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
- Predecessor tasks:
  - `.trellis/tasks/archive/2026-06/06-05-source-hash-triggering-reconciliation-policy/`
  - `.trellis/tasks/archive/2026-06/06-06-admin-source-fingerprint-hash-trigger-first-slice/`
  - `.trellis/tasks/archive/2026-06/06-06-source-hash-retry-requeue-admin-command/`
  - `.trellis/tasks/archive/2026-06/06-06-source-duplicate-reconciliation-plan-first-slice/`
  - `.trellis/tasks/archive/2026-06/06-06-admin-source-duplicate-reconciliation-apply-first-slice/`
  - `.trellis/tasks/archive/2026-06/06-06-m1-operator-journey-smoke/`
