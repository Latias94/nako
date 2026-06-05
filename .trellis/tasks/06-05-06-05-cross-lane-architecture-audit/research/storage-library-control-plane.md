# Storage, Library, And Control-Plane Audit

## Scope

Reviewed storage/VFS, library intake, source identity, source fingerprint hash,
VFS cache repair, and durable-job control-plane surfaces for the next parallel
development queue.

Primary references:

- `CONTEXT.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/LANES.md`
- `.trellis/tasks/archive/2026-06/06-05-vfs-cache-repair-executable-refresh-action/prd.md`
- `.trellis/tasks/archive/2026-06/06-05-source-fingerprint-hash-evidence-persistence-first-slice/prd.md`
- `.trellis/tasks/archive/2026-06/06-05-source-fingerprint-hash-job-diagnostics-first-slice/prd.md`
- `crates/nako-library/src/source_hash.rs`
- `crates/nako-server/src/app/source_hash.rs`
- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-server/src/app/runtime.rs`

## Current State

- **VFS cache repair** has moved past the old "operator actions" candidate:
  target inventory, target-scoped preview, action plan, and selected-target
  `refresh_cache` execution are shipped. The remaining meaningful work is
  broader non-destructive remediation and durable repair queues, not the older
  preview/action seam.
- **Source Fingerprint** has a mature first queue path: escalation policy, hash
  execution, scheduling diagnostics, durable job contract, job summary,
  internal enqueue, queued execution planner, single-job command, disk-scan
  scheduler integration, evidence persistence, overview diagnostics, and Jobs
  drill-down filters are shipped.
- **Library intake/watchers** remain weaker than source hash: `LIBRARY_PIPELINE`
  still marks watcher/debounce as weak and points to runtime/product
  integration.
- **Control plane durable jobs** now have one more job kind on the typed
  budget-admitted scheduler path. The next control-plane pressure is no longer
  proving source hash can execute; it is choosing whether to migrate more job
  kinds, expose operator/API triggers, or add broader trace/diagnostic
  surfaces.
- **Architecture maps are partly stale**. `LANES.md` still says "Consider
  source fingerprint hash operator/Admin diagnostics" even though Admin overview
  and Jobs drill-down diagnostics are now shipped. `STORAGE_VFS.md` and
  `CONTROL_PLANE.md` still correctly keep API triggering and automatic
  reconciliation as follow-ons.

## Candidate Next Tasks

### 1. Source Fingerprint Hash Triggering Policy

**Type**: architecture audit before implementation.

Define how source hash work is triggered beyond the internal app service:
scan-originated enqueue, Admin manual enqueue, retry/requeue, or policy-backed
automatic scheduling.

Why this needs audit first:

- It touches scan policy, durable jobs, Admin/API, redaction, and budget
  admission.
- It decides whether source hash remains operator-triggered evidence or becomes
  part of automatic source identity reconciliation.
- It can easily create hidden background work if ADR 0053 is not enforced.

Parallel safety:

- Do not run in parallel with broad durable-job scheduler migration.
- Can run in parallel with playback/transcode tasks if `nako-server/src/app/jobs.rs`
  and Admin DTOs are not shared.

### 2. Source Duplicate Relationship Reconciliation From Hash Evidence

**Type**: product/architecture decision before implementation.

Use newly persisted Source Fingerprint evidence to propose or update
**Source Duplicate Relationship** records.

Why it is not immediate refactor:

- The domain explicitly says **Source Fingerprint** is evidence, not source
  identity.
- Automatic duplicate behavior can surprise operators if it jumps straight from
  evidence persistence to relationship mutation.
- Needs a policy for confidence, stale evidence, same-library/cross-library
  visibility, and repair/undo.

Parallel safety:

- Serializes with any scan ingestion/source identity work.
- Can run after a small architecture PRD, then split implementation across
  repository tests and Admin diagnostics if DTO scope is managed.

### 3. VFS Cache Non-Destructive Remediation Plan

**Type**: ready bounded implementation after a short PRD.

Extend the shipped refresh-only action plan into broader non-destructive
operator guidance: classify stale fallback, retryable refresh failure,
operator-action failure, and unknown cases into explicit remediation choices.

Why it is ready:

- Existing target refs and repair action plans provide the interface.
- Current route already proves selected-target mutation with redaction.
- Can stay read-only/plan-first before durable repair jobs.

Parallel safety:

- Avoid parallel changes to `nako-api` storage DTOs or Admin Web storage routes.
- Otherwise independent from playback and Addon lanes.

### 4. Watcher/Debounce Productization

**Type**: ready-ish implementation but higher policy risk than VFS cache.

Move watcher/debounce from "weak foundation" toward product runtime: stable
candidate diagnostics, scan handoff, skipped/unsupported watcher states, and
scheduled reconciliation.

Why it needs careful scoping:

- It touches library intake, scan scheduler, storage availability, and Admin
  diagnostics.
- It can duplicate scan scheduling behavior unless routed through durable jobs.

Parallel safety:

- Do not run with source hash triggering or broad scan scheduling work.
- Can run with Addon or remote access work if Admin DTO scopes are isolated.

### 5. PostgreSQL Runtime Harness For Storage/Source Identity

**Type**: ready bounded verification/infrastructure task.

Prove PostgreSQL runtime parity for storage/source identity query paths that
have accumulated SQLite-first evidence.

Why it is useful:

- Low product ambiguity.
- Good confidence-building task before more source identity mutation.
- Helps future parallel work by catching backend drift.

Parallel safety:

- Safe with most frontend/playback/addon tasks.
- Avoid parallel DB migration work in the same tables.

## Fearless Refactor Candidates

No immediate high-confidence deletion target was found in this lane.

The source hash modules are not shallow pass-throughs:

- `nako-library::source_hash` owns execution, scheduling plan, durable input,
  and redacted summary contracts behind a small API.
- `nako-server::app::source_hash` owns app orchestration, persisted job
  validation, VFS backend resolution, execution, and evidence persistence.
- `jobs.rs` has some repeated `spawn_claimed_*` shape, but only two disk-scan
  job variants currently share it. This is not enough evidence for a new
  generic scheduler abstraction yet.

Potential future refactor:

- If a third or fourth job kind joins `disk.scan`, revisit a typed
  `DiskScanJobExecutor` registry. Today, introducing that interface would be a
  hypothetical seam.

## Recommended Priority

1. **Architecture audit**: source fingerprint hash triggering and duplicate
   reconciliation policy.
2. **Bounded implementation**: VFS cache non-destructive remediation planning.
3. **Bounded verification**: PostgreSQL runtime harness for storage/source
   identity.
4. **Later implementation**: watcher/debounce productization once scan
   scheduling policy is not competing with source hash triggering.

## Documentation Updates Needed

- `docs/architecture/LANES.md`: replace "source fingerprint hash operator/Admin
  diagnostics" with more precise remaining work: API/manual triggering,
  automatic reconciliation policy, and broader scheduler migration.
- `docs/architecture/STORAGE_VFS.md`: archive task links for VFS cache should
  use archived paths for completed Trellis tasks, not active-task paths.
- `docs/architecture/CONTROL_PLANE.md`: source hash follow-ons should
  distinguish Admin read diagnostics already shipped from Admin/API mutation
  triggering still pending.
