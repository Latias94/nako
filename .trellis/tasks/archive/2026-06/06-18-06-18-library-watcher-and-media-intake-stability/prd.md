# Library watcher and media intake stability

## Goal

Make incremental library intake reliable for large files, slow copies, and remote storage by adding a stable-size fallback to watch-folder intake. The first slice should let repeated unchanged size observations graduate to stable without needing richer metadata markers, while keeping the existing redaction-safe scan handoff and watcher supervision model intact.

## What I already know

* `docs/architecture/LIBRARY_PIPELINE.md` still marks watcher/debounce as weak and calls out stable-size detection, copy-in-progress handling, scheduled reconciliation scan, and per-library intake diagnostics as remaining follow-ons.
* The repo already has stable-candidate intake classification in `crates/nako-library/src/intake.rs`.
* The server already has supervised watch-folder runtime logic in `crates/nako-server/src/app/watch_folder_runtime.rs` and intake discovery/apply flow in `crates/nako-server/src/app/acquisition_intake.rs`.
* Admin overview already exposes watch-folder runtime coverage and latest tick diagnostics, plus scan intake evidence/action-plan reads.
* Existing archived tasks already covered latest tick readiness, scan job posture readiness, degraded reconciliation diagnostics, and intake action plan evidence.
* The legacy watch-folder source-key cleanup slice is already archived, so this task should not reopen that line.

## Assumptions (temporary)

* The next useful slice should stay backend-first and keep the Admin overview/read model as the primary operator surface.
* This task should not add a new watcher daemon or a new scan executor.
* We should prefer deepening existing intake/runtime seams before inventing a broader orchestration layer.

## Requirements (evolving)

* Preserve the existing stable-candidate observation model and watcher runtime supervision.
* Add a stable-size fallback so repeated same-size observations can graduate to `Stable` with the existing consecutive-observation threshold.
* Keep size-only and copy-in-progress observations in `Inspecting` until the threshold is met.
* Keep the fallback redaction-safe and operator-visible through the existing intake/runtime path.
* Avoid schema changes and avoid widening public DTOs unless the slice proves them necessary.
* Keep scan admission, intake classification, and watch-folder runtime behavior aligned with current M2 maturity goals.

## Acceptance Criteria (evolving)

* [ ] Repeated same-size watch-folder observations can graduate to `Stable` through the existing intake primitive.
* [ ] A changed size resets the candidate back to `Inspecting`.
* [ ] Missing size evidence remains `Inspecting` and does not spuriously enqueue a scan.
* [ ] Server/runtime tests still show scan handoff only after the intake becomes stable.
* [ ] No raw roots, Source Locators, paths, tokens, or raw backend errors are exposed.
* [ ] Focused server/library tests and formatting pass.

## Definition of Done (team quality bar)

* Tests added/updated (unit/integration where appropriate)
* Lint / typecheck / CI green
* Docs/notes updated if behavior changes
* Rollout/rollback considered if risky

## Out of Scope (explicit)

* OS filesystem watcher daemon
* New scan executor or worker loop
* Schema migrations
* Frontend changes
* Automatic source duplicate reconciliation
* Legacy watch-folder source-key cleanup

## Technical Notes

* Relevant roadmap: `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`
* Relevant architecture: `docs/architecture/LIBRARY_PIPELINE.md`, `docs/architecture/STORAGE_VFS.md`, `docs/architecture/CONTROL_PLANE.md`, `docs/architecture/STATE_ACCESS.md`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`
* Relevant code: `crates/nako-library/src/intake.rs`, `crates/nako-server/src/app/watch_folder_runtime.rs`, `crates/nako-server/src/app/acquisition_intake.rs`, `crates/nako-server/src/http/admin.rs`
* Related archived tasks: watch-folder latest tick readiness, scan job posture readiness, degraded reconciliation diagnostics, and intake action plan evidence

## Technical Approach

Teach the existing stable-candidate primitive to accept repeated same-size observations as a weaker but still redaction-safe stability signal. Keep the observation threshold and the watch-folder runtime path unchanged, add focused tests for size-only stability and changed-size reset, and let existing server/runtime handoff logic consume the strengthened library contract.

## Decision (ADR-lite)

**Context**: The watcher/debounce lane is still marked weak because the current intake primitive depends on richer metadata markers that are not always available for large files or remote storage.

**Decision**: Add a size-only stable fallback inside the existing stable-candidate evidence seam instead of introducing a new watcher daemon, scheduler, or schema.

**Consequences**: This improves reliability for slow-copy and remote-storage cases while keeping the interface shallow. The trade-off is that the first slice remains conservative and does not try to solve full filesystem event ingestion in one step.
