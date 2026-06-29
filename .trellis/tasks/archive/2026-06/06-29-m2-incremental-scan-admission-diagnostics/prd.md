# M2 Incremental Scan Admission Diagnostics

## Goal

Make watch-folder incremental scan admission decisions easier to diagnose for
self-hosted operators and future M2 reliability work. The current runtime
already detects stable candidates and admits scans through the existing
`LibraryScanAppService` path; this task should deepen the operator evidence for
why a tick did or did not admit a scan without adding a second scheduler, an OS
watcher daemon, or broad UI work.

## What I Already Know

* `docs/ROADMAP.md` defines M2 large-library reliability around bounded and
  observable watcher/incremental scan, retries, long-running jobs, and storage
  behavior.
* `docs/architecture/LIBRARY_PIPELINE.md` records the shipped watcher/intake
  MVP: repeated watch observations, stable-size detection, Admin overview
  watch-folder coverage, latest tick status, and no OS watcher daemon yet.
* `crates/nako-server/src/app/watch_folder_runtime.rs` owns supervised
  watch-folder ticks and currently records:
  * `WatchFolderRuntimeOutcomeStatus`;
  * `WatchFolderIntakePlan`;
  * discovery failures;
  * `WatchFolderScanAdmissionStatus`;
  * optional `scan_job_id`;
  * `reused_existing_scan`;
  * `backoff_required`.
* `crates/nako-server/src/app/tests/startup.rs` already covers stable
  admission, completed-scan non-reenqueue, missing-size evidence, restart
  continuation, discovery failure backoff, queued/running scan reuse,
  changed-size waiting, suppression, and reconciliation completion.
* `.trellis/spec/nako-server/backend/quality-guidelines.md` requires
  watch-folder intake tests to prove size-only fallback, missing size evidence,
  and changed-size reset before scan admission.

## Requirements

* Keep scan admission behind `LibraryScanAppService::admit_watch_folder_library_scan`.
* Add or refine typed watch-folder runtime evidence so operators and Admin
  readiness code can distinguish:
  * admitted because new stable candidates exist;
  * reused an existing queued or running scan;
  * skipped because candidates are still inspecting;
  * skipped because candidates are already ready but not newly ready;
  * skipped because candidates are suppressed, blocked, or discovery failed.
* Preserve redaction boundaries: no raw local paths, Source Locators,
  `StorageUri`, filenames, credentials, durable input JSON, raw backend
  errors, or job summaries in Admin/API diagnostics.
* Prefer a server-side first slice. Touch `nako-api` or Admin Web only if the
  existing Admin overview/readiness DTO cannot expose the new evidence safely.
* Add focused tests that prove the evidence is correct for at least:
  * admitted new stable candidates;
  * existing scan reuse;
  * repeated already-ready candidates;
  * missing or changing stability evidence;
  * failure/suppression/blocking paths if touched.

## Acceptance Criteria

* [ ] Watch-folder runtime records a typed admission/skip reason that is more
      explicit than `scan_admission_status` alone.
* [ ] The new evidence is derived from existing intake plan and scan-admission
      outcome, not from raw storage paths or duplicated scheduling logic.
* [ ] Repeated stable candidates and copy-in-progress candidates remain
      non-admitting where they already should.
* [ ] Existing Admin readiness semantics remain compatible unless intentionally
      refined with tests.
* [ ] Focused Rust tests cover the new evidence and pass under `cargo nextest`.
* [ ] Specs or architecture docs are updated if the task establishes a reusable
      contract beyond tests.

## Definition Of Done

* Focused tests are added or updated.
* `cargo fmt --all` or focused formatting is run when Rust files change.
* Focused `cargo nextest` gates pass.
* `cargo check` is run for touched backend crates where practical.
* Any API/Admin Web contract changes are regenerated and checked.
* Trellis task is checked, finished, archived, committed, and the working tree
  is clean.

## Technical Approach

Start in `crates/nako-server/src/app/watch_folder_runtime.rs` and model the
admission decision close to the existing tick construction. If the evidence is
purely a projection of `WatchFolderIntakePlan` plus
`WatchFolderScanAdmissionStatus`, keep it in the server runtime layer and test
through app-level startup tests. Only promote fields into `nako-api` Admin DTOs
if the current Admin overview/readiness evidence cannot represent the decision
without ambiguity.

## Decision (ADR-lite)

Context: M2 needs large-library intake behavior to be observable before adding
larger scheduling or OS watcher machinery. The existing runtime has enough
execution behavior, but the decision vocabulary is still split across intake
counts, enqueue reason, scan status, and reuse booleans.

Decision: Add a narrow typed diagnostic vocabulary for incremental scan
admission decisions in the existing watch-folder runtime path.

Consequences: Operators and future tests get a clearer reason model without
creating another scheduler or widening storage/runtime authority. If this
later needs to be rendered in Admin Web, the backend evidence can be promoted
through existing redaction-safe Admin DTO patterns.

## Out Of Scope

* OS-native filesystem watcher integration.
* New durable scan scheduler or alternate enqueue path.
* Remote-storage watcher support beyond existing local-root coverage.
* Destructive storage repair, purge, delete, or invalidation behavior.
* Broad Admin Web redesign.
* Media Web browse/player hardening.
* Automatic Source Duplicate Relationship merge or undo policy.

## Technical Notes

* Roadmap: `docs/ROADMAP.md`
* Library pipeline: `docs/architecture/LIBRARY_PIPELINE.md`
* Control plane: `docs/architecture/CONTROL_PLANE.md`
* Server spec: `.trellis/spec/nako-server/backend/index.md`
* API spec: `.trellis/spec/nako-api/backend/index.md`
* Admin Web spec: `.trellis/spec/admin-web/frontend/index.md`
* Key code paths:
  * `crates/nako-server/src/app/watch_folder_runtime.rs`
  * `crates/nako-server/src/app/acquisition_intake.rs`
  * `crates/nako-server/src/app/jobs.rs`
  * `crates/nako-server/src/app/tests/startup.rs`
