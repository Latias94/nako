# Backend readiness control-plane audit

Date: 2026-06-16

Scope: U1 backend readiness/control-plane audit for the current admin backend
surface. This slice is documentation-only. No public API, schema, generated
contract, frontend runtime, or Rust code change is introduced here.

## Inputs Reviewed

Task and architecture context:

* `.trellis/tasks/06-16-06-16-backend-readiness-control-plane-audit/prd.md`
* `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`
* `docs/architecture/CONTROL_PLANE.md`
* `docs/architecture/OPERATIONS_RELEASE.md`
* `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`
* `docs/architecture/STORAGE_VFS.md`
* `docs/architecture/PLAYBACK.md`
* `docs/deployment/SELF_HOSTED.md`
* `docs/deployment/BACKUP_RESTORE_UPGRADE.md`
* `docs/development/REFACTORING_POLICY.md`

Required Trellis specs:

* `.trellis/spec/guides/index.md`
* `.trellis/spec/guides/code-reuse-thinking-guide.md`
* `.trellis/spec/guides/cross-layer-thinking-guide.md`
* `.trellis/spec/nako-server/backend/index.md`
* `.trellis/spec/nako-server/backend/directory-structure.md`
* `.trellis/spec/nako-server/backend/quality-guidelines.md`
* `.trellis/spec/nako-server/backend/http-api-patterns.md`
* `.trellis/spec/nako-server/backend/logging-guidelines.md`
* `.trellis/spec/nako-api/backend/index.md`
* `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
* `.trellis/spec/nako-api/backend/directory-structure.md`
* `.trellis/spec/nako-api/backend/quality-guidelines.md`
* `.trellis/spec/nako-api/backend/error-handling.md`
* `.trellis/spec/nako-api/backend/logging-guidelines.md`

Primary code evidence:

* `crates/nako-api/src/admin.rs`
* `crates/nako-api/src/admin/network.rs`
* `crates/nako-api/src/admin/operations.rs`
* `crates/nako-server/src/app/startup.rs`
* `crates/nako-server/src/app/runtime.rs`
* `crates/nako-server/src/app/jobs.rs`
* `crates/nako-server/src/app/acquisition_intake.rs`
* `crates/nako-server/src/http/admin.rs`
* `crates/nako-server/src/http/network.rs`
* `crates/nako-server/src/http/auth.rs`
* `crates/nako-server/src/app/tests/startup.rs`
* `crates/nako-server/src/http/tests/system.rs`
* `crates/nako-server/src/http/tests/admin_route_inventory.rs`

## Current Readiness Surface

The backend already exposes a single admin overview readiness surface through
`AdminOverviewResponse` in `crates/nako-api/src/admin.rs`. The response includes
API versions, overall status, six operator-readiness checks, storage/catalog/
metadata/runtime summaries, source-fingerprint hash backlog facts, and startup
facts. The readiness areas are setup, media library scan, playback, storage,
network, and backup.

`crates/nako-server/src/http/admin.rs` composes this surface in
`admin_overview_response` and `operator_readiness_summary`. The composition is
read-only and pulls from existing app seams:

* setup/auth config
* configured media library counts
* runtime task diagnostics
* job queue/source-fingerprint hash summaries
* latest watch-folder runtime tick diagnostics
* storage backend and VFS cache repair pressure
* playback runtime diagnostics
* network access diagnostics
* startup recovery/cleanup report

This is the correct control-plane direction for U1: the HTTP layer aggregates
facts and maps them to operator-ready/degraded/unavailable status; ownership of
runtime work remains under app/runtime/startup seams.

## Readiness Evidence Matrix

| Area | Current evidence | Coverage strength | Missing backend evidence |
| --- | --- | --- | --- |
| Setup/auth | `setup_readiness_check` reports auth configured, token-reference missing, local-only disabled auth, and disabled auth with remote exposure. Auth middleware inserts current principal facts. | Strong for config posture. Tests cover valid token and auth-disabled principal insertion. | Principal compatibility cleanup remains; one admin access route still consumes the legacy `UserPrincipalId` extension. |
| Media library scan | `media_library_scan_readiness_check` combines configured library count, source-hash queue pressure, failed runtime/job pressure, and watch-folder runtime coverage gaps. | Strong for watch-folder/source-hash pressure and redaction. Tests cover queue pressure, repair pressure, latest tick diagnostics, stable observations, suppression, reuse of active scans, and redacted skipped roots. | No general per-library "last successful scan / last failed scan" operator check beyond current queue/runtime pressure. |
| Playback | `playback_readiness_diagnostics` checks FFmpeg probe, hardware selection, CPU fallback, budget clamping, remote stream/stage permits, policy readiness, staging capacity, artifact lifecycle, and throttle state. | Good backend dependency coverage without command-line exposure. | Public readiness vocabulary remains broad; future U3 work may need more operator-specific action labels, but this U1 slice should not expand public API. |
| Storage/VFS | `storage_readiness_check` combines backend health with unresolved VFS cache repair pressure. VFS repair pressure comes from unresolved repair targets, not just the latest failure. | Strong. Existing tests verify unresolved target inventory and admin overview redaction. | No deletion work needed in this slice. |
| Network | `AdminNetworkAccessDiagnostics` and `AdminNetworkReadinessDiagnostics` describe exposure mode, external endpoint, trusted proxy policy, origins, and tunnel provider posture using fingerprints/counts rather than raw URLs or tokens. | Strong for system config diagnostics. Overview currently exposes the mapped network readiness check, not the full detail object. | If operators need endpoint detail directly in overview, add it through existing safe DTO fields only after an API contract decision. |
| Backup/restore | `backup_readiness_check` distinguishes in-memory SQLite from durable database configuration and points to the backup/restore runbook. | Minimal but intentional for U1. | No live backup recency, snapshot presence, restore drill, or upgrade readiness fact is exposed. |
| Durable jobs/runtime | Runtime summary exposes active/completed/failed managed tasks and jobs. Admin jobs and incident bundle surfaces include redacted queue pressure. | Good for drilldown and incident bundle. | Overview has no distinct durable-job readiness area; generic non-scan backlog may not degrade operator readiness unless it appears as runtime failure or scan/source-hash pressure. |
| Startup recovery | `ServerStartupReport` records migration, configured libraries, recovered transcode sessions, recovered unfinished jobs, staging cleanup, playback artifact cleanup, metadata raw cache cleanup, lifecycle startup, and watch-folder runtime coverage. | Good startup fact inventory. | Recovery counts are startup facts, not operator readiness blockers unless other runtime/job pressure remains. |
| Addon/runtime supervision | Addon event scheduler and VFS repair/watch-folder runtimes are started through app startup/runtime seams, not ad hoc HTTP work. | Good for control-plane ownership. | Legacy Addon task resource-class compatibility remains. Future removal needs persisted-job evidence. |

## Redaction Review

Operator-facing admin facts are already designed around safe summaries:

* Network endpoint facts use scheme plus host fingerprint fields, not full
  backend URLs or secret query strings.
* Tunnel provider diagnostics expose endpoint-configured and token-present
  booleans plus token environment variable names, not token values.
* Source-fingerprint hash overview and job queue pressure expose counts and
  retry timestamps, not source locators or provider payloads.
* Watch-folder overview diagnostics expose redacted root/tick summaries, not
  raw local paths, source locators, object fingerprints, or credentials.
* Playback readiness reports dependency posture, not FFmpeg command lines.
* Existing admin overview, incident bundle, system config, watch-folder, source
  hash, VFS repair, and network boundary tests assert that sensitive strings do
  not appear in response bodies.

One boundary nuance is worth keeping explicit: `crates/nako-server/src/http/network.rs`
can emit an external-origin response header when trusted proxy policy accepts
the forwarded source. That is transport boundary behavior, not an admin
readiness DTO. Any future readiness projection of that fact should continue to
use fingerprints, counts, and booleans.

## Legacy Compatibility Classification

| Compatibility path | Classification | Evidence | Deletion requirements |
| --- | --- | --- | --- |
| Legacy watch-folder source-key fallback | Keep | `find_existing_watch_folder_candidate` first checks the current `watch_folder:<uri>` key, then falls back to `legacy_watch_folder_source_key`. The characterization test `acquisition_intake_watch_folder_discovery_updates_legacy_source_key_without_duplicate` proves this prevents duplicate candidates for persisted legacy rows and verifies redaction of local URI, path, movie name, and fingerprint. | Do not remove until there is a migration, repair command, or repository-level characterization proving legacy source keys have been normalized or can no longer exist. After that, replace the current compatibility test with a current-key-only duplicate-prevention test and verify watch-folder discovery still redacts source facts. |
| Legacy Addon task resource-class mapping | Remove after test | `runtime_budget_class_for_job_resource_class` maps `addon.task`, `addon.generated_artifact_handoff`, and any `addon.task.*` legacy class to the `addon.task` budget class. The runtime mapping test currently asserts `addon.task.bulk-refresh` compatibility. No current production writer for arbitrary `addon.task.*` was identified in this audit, but persisted durable jobs can outlive code changes. | Search/migrate persisted Addon task jobs, prove current writers only use canonical classes, then replace the compatibility assertion with a rejection or no-match test for `addon.task.*`. Run focused runtime/job scheduler tests before deleting `is_legacy_addon_task_resource_class`. |
| Legacy principal normalization | Remove after test | `require_auth` inserts both `AuthenticatedPrincipal` and the legacy `UserPrincipalId` extension. `get_admin_access_summary` still extracts `Extension<UserPrincipalId>`, and auth tests assert the legacy extension for token and auth-disabled flows. | Migrate the remaining admin access handler to `Extension<AuthenticatedPrincipal>`, add route-level tests for bearer token, user session, and auth-disabled flows, then assert no HTTP handler or middleware test still depends on `Extension<UserPrincipalId>` or `Request::extensions().get::<UserPrincipalId>()`. Only then remove legacy insertion and update auth tests. |

No compatibility path has enough evidence for "remove now" in this task.

## Safe Cleanup Decision

No code cleanup was performed.

Reason: each obsolete-looking path is still protected by either direct handler
usage, direct characterization tests, or durable persisted-state compatibility
risk. Removing any of them in this slice would weaken compatibility without a
failing characterization test or migration evidence.

## Recommended First Code-Bearing Follow-Up

First follow-up: remove legacy principal normalization after migrating the last
HTTP consumer to `AuthenticatedPrincipal`.

Why this should be first:

* It is narrow and does not require public API, schema, generated contract, or
  frontend changes.
* The dependency graph is small: `crates/nako-server/src/http/auth.rs` and
  `crates/nako-server/src/http/admin.rs`.
* The deletion safety criteria are objective and easy to verify with `rg`.
* It reduces future confusion between the canonical authenticated principal and
  the older principal-id-only extension.

Proposed file areas:

* `crates/nako-server/src/http/admin.rs`: change `get_admin_access_summary` to
  consume `Extension<AuthenticatedPrincipal>` and derive the principal ID from
  the canonical principal.
* `crates/nako-server/src/http/auth.rs`: after route migration and tests, stop
  inserting the legacy `UserPrincipalId` extension.
* `crates/nako-server/src/http/tests/system.rs` and auth unit tests: cover admin
  access summary for token, session, and auth-disabled local admin flows.

Deletion safety criteria:

* `rg "Extension\\(principal\\): Extension<UserPrincipalId>" crates/nako-server/src`
  returns no production handler.
* `rg "get::<UserPrincipalId>" crates/nako-server/src/http` returns no
  compatibility assertion that still needs to pass.
* Focused `nako-server` auth/admin access tests pass under `cargo nextest run`
  or the narrowest available equivalent.

Second follow-up after that should be durable-job/operator readiness
characterization: prove whether non-scan job backlog should degrade the admin
overview, then add or intentionally reject a distinct durable-job readiness
check through the admin API contract process.

## Verification

Planned for this documentation-only slice:

* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-16-06-16-backend-readiness-control-plane-audit`
* `git diff --check`

No Rust code changed, so no focused `nextest` or `cargo check` run is required
for this slice.
