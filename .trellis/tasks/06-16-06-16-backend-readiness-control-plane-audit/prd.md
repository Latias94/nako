# Backend readiness control-plane audit

## Goal

Implement the first backend maturity slice from the self-hosted execution plan:
audit backend operator readiness, control-plane ownership, and obsolete
compatibility paths so the next code changes are evidence-led. The task should
produce a backend-owned readiness/control-plane gap report and, where the gap is
narrow and already covered by tests, land the smallest safe cleanup.

## Plan Anchor

This task is the first execution item from
`docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`,
unit U1.

## What I Already Know

* Nako already exposes Admin readiness, startup diagnostics, network
  diagnostics, watch-folder runtime coverage, durable jobs, and VFS repair
  facts.
* The execution plan says U1 should establish the operator evidence surface and
  identify deletion candidates before deeper backend refactors.
* Historical analysis identified possible cleanup candidates:
  legacy watch-folder source-key fallback, legacy Addon task resource-class
  mapping, and legacy principal normalization.
* Cleanup is allowed only after search and characterization tests prove the
  current path owns behavior.
* This task should not change public API shape unless the audit finds a narrow
  missing readiness fact that is already backed by existing internal data.

## Requirements

* Audit existing backend readiness and control-plane read models across startup,
  Admin overview, network diagnostics, durable jobs, runtime supervision, VFS
  repair, and playback dependency readiness.
* Produce a repo-local audit document that maps current evidence, gaps, and
  first safe implementation candidates.
* Keep all operator-facing facts redaction-safe: no raw local paths, Source
  Locators, fingerprints, credentials, provider payloads, FFmpeg command lines,
  backend URLs, or secret query strings.
* Search for obsolete compatibility paths and classify each as `keep`, `remove
  now`, or `remove after test`.
* If a compatibility path is proven removable with existing tests, remove it in
  this task; otherwise record the evidence needed before deletion.
* Keep runtime work under existing control-plane or runtime-supervisor seams; do
  not introduce raw `tokio::spawn`, schema migrations, generated contract
  changes, or frontend changes in this slice unless a discovered narrow blocker
  forces it.

## Acceptance Criteria

* [ ] A task-local audit document records current readiness facts, missing
      backend evidence, and cleanup candidates.
* [ ] The audit includes the first recommended code-bearing follow-up with file
      areas, tests, and deletion safety criteria.
* [ ] Legacy source-key, legacy Addon task resource-class, and legacy principal
      normalization paths are searched and classified.
* [ ] Any code cleanup performed by this task has focused tests or existing
      passing characterization evidence.
* [ ] No new public API, schema, generated contract, or frontend runtime change
      is introduced unless explicitly justified in the audit.
* [ ] Focused verification runs for changed layers; if the task remains
      documentation-only, Trellis validation and `git diff --check` are enough.

## Definition of Done

* Audit document is written under the task directory.
* Any safe cleanup is implemented and verified, or all cleanup is deferred with
  evidence requirements.
* Trellis context validates.
* `git diff --check` passes.
* Focused Rust tests pass when Rust code changes.
* Commit message is conventional.

## Out of Scope

* Building the full readiness overview UI.
* Large-library browse/search contract implementation.
* Playback reason public contract implementation.
* VFS repair mutation policy or automatic destructive repair.
* Addon Manager process supervision.
* Remote relay or central account service.
* GPL reference source copying.

## Technical Notes

Primary code areas to inspect:

* `crates/nako-api/src/admin.rs`
* `crates/nako-api/src/admin/network.rs`
* `crates/nako-server/src/app/startup.rs`
* `crates/nako-server/src/app/runtime.rs`
* `crates/nako-server/src/app/jobs.rs`
* `crates/nako-server/src/app/acquisition_intake.rs`
* `crates/nako-server/src/http/admin.rs`
* `crates/nako-server/src/http/network.rs`
* `crates/nako-server/src/http/auth.rs`
* `crates/nako-server/src/app/tests/startup.rs`
* `crates/nako-server/src/http/tests/admin_route_inventory.rs`

Expected audit artifact:

* `.trellis/tasks/06-16-06-16-backend-readiness-control-plane-audit/audit.md`

## Open Questions

None. Implementation should start with audit and only perform safe cleanup when
evidence is strong.
