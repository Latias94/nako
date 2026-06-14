# Source Fingerprint Job Diagnostics Gap

## Question

What concrete M2 follow-on should deepen durable job diagnostics without
reopening completed source duplicate or VFS cache repair work?

## Findings

- `docs/ROADMAP.md` defines M2 as large-library reliability: watcher and
  incremental scan, source hash scheduling, VFS repair, job priority/retry,
  SQLite/PostgreSQL parity, and backup/recovery gates.
- The archived `M2 large-library reliability plan` selected watcher and
  incremental scan reliability first, and recent follow-ons have already
  shipped watch-folder reliability and VFS cache repair automation work.
- `docs/architecture/CONTROL_PLANE.md` keeps durable jobs, tracing, Admin
  diagnostics, API scale/cache, and broader job-kind scheduler migration as
  control-plane concerns under ADR 0053.
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` says Admin Jobs
  drilldown work should open only for a concrete stuck job class, not as a
  broad platform.
- `nako-api::admin::AdminJobDiagnostics` currently expands only
  `JobKind::VfsCacheRepair`. `JobKind::SourceFingerprintHash` has queue
  pressure, Admin enqueue/retry, scheduler execution, and overview counters,
  but not a typed Admin Jobs drilldown diagnostic.
- `nako-library::source_hash::SourceFingerprintHashJobSummary` is already a
  redaction-safe summary shape: mode, evidence kind, confidence, stale state,
  and bytes hashed. It excludes raw Source Locators, paths, etags, hashes, and
  fingerprint material.
- `SourceFingerprintHashJobInput` is safe enough for durable persistence but
  should still not be exposed wholesale through Admin Jobs. A drilldown DTO can
  expose selected safe facts if useful: mode, source scheme, library/source
  bindings already present on the job row, and optional request correlation
  only if it follows the existing request-id redaction policy.

## Recommendation

Open a focused M2 task that adds source-fingerprint-specific Admin Jobs
diagnostics using the existing `diagnostics` field, without adding a new route,
schema, executor, scheduler, or Admin Web workflow.

The first implementation slice should:

- extend `AdminJobDiagnostics` with an optional `source_fingerprint_hash`
  branch;
- parse redaction-safe source hash summary JSON into a typed Admin DTO;
- classify pending, summary-available, and failed states consistently with the
  VFS cache repair diagnostic style;
- expose only safe mode/evidence/confidence/stale/bytes-hashed facts and
  redacted failure state;
- add focused API serialization tests and server route/list tests that reject
  raw input/error leakage.

## Non-Recommendations

- Do not reopen source duplicate reconciliation. Backend plan/apply and Admin
  Web operator flow are already shipped.
- Do not add automatic source merge or duplicate reconciliation from this job
  drilldown.
- Do not build a generic job detail platform. The safe shape should stay
  job-kind-specific.
- Do not expose durable `input_json`, raw `summary_json`, raw errors, Source
  Locators, local paths, backend URLs, etags, hashes, fingerprints, or
  credentials.
