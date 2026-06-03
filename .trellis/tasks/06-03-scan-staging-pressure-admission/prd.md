# Scan Staging Pressure Admission

## Goal

Add the next bounded storage-vfs follow-on after `04b` by teaching library scan
entry to respect existing staging-pressure signals, so Nako does not keep
starting scan/probe work when staging capacity is already exhausted or near
exhaustion.

## What I already know

* `06-02-04b-library-scan-scheduling-storage-admission` already blocks scan
  work on durable `Storage Circuit Breaker` state before the scan/probe/metadata
  pipeline starts.
* Existing staging diagnostics already summarize manifest pressure and classify
  `Healthy`, `Elevated`, `Critical`, and `Exhausted` pressure states.
* Current staging-pressure logic is visible in
  `crates/nako-server/src/app/storage.rs` and `crates/nako-server/src/http/admin.rs`.
* `docs/architecture/STORAGE_VFS.md` explicitly keeps staging-pressure-based
  admission as a separate follow-on from durable storage circuit admission.
* The likely implementation seam is the same typed scan-entry admission path
  used by `LibraryScanAppService::run_library_scan`.

## Assumptions (temporary)

* MVP should reuse existing staging-pressure summaries instead of adding a new
  storage schema or scheduler service.
* Admission should stay redaction-safe and avoid exposing paths, Source
  Locators, fingerprints, or raw backend errors.
* Queued background scans should keep using the durable job/runtime path rather
  than adding a hidden bypass.

## Open Questions

* None for the MVP boundary. This task is locked to the smallest useful slice:
  scan-entry admission only, without a new Admin DTO or operator-specific
  diagnostics surface.

## Requirements (evolving)

* Audit current staging-pressure signals, scan scheduling, and scan-entry
  admission seams before applying the existing typed entry boundary.
* Prefer a small typed admission policy over a broad scheduler rewrite.
* Reuse Nako terms: Storage Backend Health, Storage Circuit Breaker, staging
  manifest pressure, Source Locator, Source Fingerprint.
* Keep diagnostics redaction-safe.
* Add focused tests for synchronous scan and queued background scan behavior.
* Keep the MVP at scan-entry admission only; do not add a new Admin DTO or
  operator-specific failure surface unless implementation proves it is required.

## Acceptance Criteria (evolving)

* [ ] The selected slice is documented with the reason it is the smallest useful
  follow-on after `04b`.
* [ ] Scan admission reacts to existing staging-pressure state through a typed
  boundary before scan/probe work starts.
* [ ] Existing local/WebDAV scan behavior remains compatible when staging
  pressure is healthy.
* [ ] Existing operator-visible diagnostics remain compatible without adding a
  new staging-admission-specific DTO.
* [ ] Deferred follow-ons are recorded for PostgreSQL runtime parity, scheduler
  fairness, watcher/debounce, or broader pressure policies.

## Definition of Done (team quality bar)

* Tests added or updated for synchronous and queued scan behavior.
* `cargo fmt --all -- --check` passes.
* Focused `cargo check` / `cargo nextest` gates pass for touched packages.
* `git diff --check` passes.
* Evidence notes record the selected slice, commands, and deferred follow-ons.

## Out of Scope (explicit)

* No broad scheduler rewrite.
* No new schema migration unless a later planner explicitly approves it.
* No Public Client API change.
* No raw path, Source Locator, Source Fingerprint, credential, or raw backend
  error exposure.
* No PostgreSQL parity harness in the same MVP unless the task scope is revised.
* No new Admin DTO or extra operator-facing rejection surface in this slice.

## Technical Notes

* Likely modules:
  - `crates/nako-server/src/app/jobs.rs`
  - `crates/nako-server/src/app/storage.rs`
  - `crates/nako-server/src/app/tests/startup.rs`
  - `crates/nako-server/src/http/admin.rs` if diagnostics expand
* Existing pressure summary logic:
  - `StorageDiagnosticsAppService::summarize_staging_manifest_pressure`
  - `storage_staging_pressure_status(...)`
* Authority:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/CONTROL_PLANE.md`
