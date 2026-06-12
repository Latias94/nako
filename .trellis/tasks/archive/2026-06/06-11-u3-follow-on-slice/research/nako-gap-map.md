# Research: Nako operator gap map

- Query: Identify the most obvious operator gap still open in Nako, and map the shipped surfaces that already exist for a one-day follow-on.
- Scope: mixed
- Date: 2026-06-11

## Findings

### Recommended one-day slice

Redacted incident bundle export is the clearest next operator gap. The control-plane map says safe realtime diagnostics and incident bundles are still future work, and the crash/fault bundle row is explicitly "Not started" (`docs/architecture/CONTROL_PLANE.md:48`, `docs/architecture/CONTROL_PLANE.md:49`, `docs/architecture/CONTROL_PLANE.md:350`). The M1 coverage matrix also keeps incident bundles deferred (`docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:67`). At the same time, the release checklist already defines what a safe support bundle may contain, which means the slice is a productization/wiring gap rather than a missing data-source gap (`docs/deployment/RELEASE_CHECKLIST.md:257`, `docs/deployment/RELEASE_CHECKLIST.md:263`, `docs/deployment/RELEASE_CHECKLIST.md:264`).

The bundle can be assembled from existing safe summaries already exposed through Admin API and Admin Web: overview/system config, playback runtime/support evidence, jobs/queue pressure, storage staging/VFS repair, source fingerprint hash, and source duplicate reconciliation. Those surfaces are already shipped as separate routes and pages.

### Shipped surfaces

- Overview and system/config posture are already shipped as read-only diagnostics: `GET /admin/v1/overview` and `GET /admin/v1/system/config` (`crates/nako-server/src/http/admin.rs:213`, `crates/nako-server/src/http/admin.rs:1461`, `crates/nako-server/src/http/admin.rs:1522`). The DTOs are already present in `nako-api` (`crates/nako-api/src/admin.rs:310`, `crates/nako-api/src/admin.rs:34`) and the Admin Web overview/settings routes are live (`apps/admin-web/src/App.tsx:126`, `apps/admin-web/src/App.tsx:148`, `apps/admin-web/src/App.tsx:240`, `apps/admin-web/src/features/overview/OverviewPage.tsx:166`, `apps/admin-web/src/features/overview/OverviewPage.tsx:211`).
- Playback runtime and support evidence are already shipped: `GET /admin/v1/playback/runtime` and `GET /admin/v1/playback/support` (`crates/nako-server/src/http/admin.rs:3337`, `crates/nako-server/src/http/admin.rs:3358`). The support DTO is redaction-shaped and typed (`crates/nako-api/src/admin_contract.rs:3702`, `crates/nako-api/src/admin_contract.rs:3957`), and the client/Web wiring already exists (`apps/admin-web/src/adminApi/client.ts:641`, `apps/admin-web/src/App.tsx:220`, `apps/admin-web/src/features/items/ItemDetailPage.tsx:172`).
- Jobs visibility is already shipped as a redacted list plus queue pressure summary: `GET /admin/v1/jobs` (`crates/nako-server/src/http/admin.rs:323`, `crates/nako-server/src/http/admin.rs:3167`). The payload is the redacted `AdminJobListItem`/`AdminJobListResponse` shape (`crates/nako-api/src/admin/operations.rs:121`, `crates/nako-api/src/admin/operations.rs:171`, `crates/nako-api/src/admin_contract.rs:3205`, `crates/nako-api/src/admin_contract.rs:3260`), and the Admin Web Jobs page already consumes live job data, cancel, source-hash retry, and VFS repair execute/retry actions (`apps/admin-web/src/features/jobs/JobsPage.tsx:87`, `apps/admin-web/src/features/jobs/JobsPage.tsx:103`, `apps/admin-web/src/features/jobs/JobsPage.tsx:121`, `apps/admin-web/src/features/jobs/JobsPage.tsx:432`).
- Source fingerprint hash is already a full control-plane slice: manual enqueue, retry, overview pressure, scheduler integration, idempotency, and redacted execution are all present (`crates/nako-server/src/http/admin.rs:327`, `crates/nako-server/src/http/admin.rs:331`, `crates/nako-server/src/app/source_hash.rs:95`, `crates/nako-server/src/app/source_hash.rs:134`, `crates/nako-server/src/app/source_hash.rs:157`, `crates/nako-server/src/app/source_hash.rs:168`, `crates/nako-server/src/app/source_hash.rs:189`, `crates/nako-server/src/app/source_hash.rs:289`, `crates/nako-server/src/app/source_hash.rs:337`, `crates/nako-server/src/app/source_hash.rs:420`, `crates/nako-server/src/app/jobs.rs:1107`, `crates/nako-server/src/app/jobs.rs:1178`). The overview DTO already tracks queue pressure, delayed retries, and next retry timing (`crates/nako-api/src/admin_contract.rs:2328`, `crates/nako-api/src/admin_contract.rs:2339`, `apps/admin-web/src/features/overview/OverviewPage.tsx:173`, `apps/admin-web/src/features/overview/OverviewPage.tsx:185`).
- Source duplicate reconciliation is already shipped as a plan/apply flow: routes exist, the app service enforces library ownership, stale checks, fingerprint matching, canonical pair handling, and suggest-only apply semantics (`crates/nako-server/src/http/admin.rs:335`, `crates/nako-server/src/http/admin.rs:339`, `crates/nako-server/src/app/source_duplicate.rs:35`, `crates/nako-server/src/app/source_duplicate.rs:107`, `crates/nako-server/src/app/source_duplicate.rs:169`). The Admin DTOs and generated contract are already in place (`crates/nako-api/src/admin/operations.rs:307`, `crates/nako-api/src/admin/operations.rs:335`, `crates/nako-api/src/admin_contract.rs:531`, `crates/nako-api/src/admin_contract.rs:542`), and the Web data source already exposes the flow (`apps/admin-web/src/adminApi/dataSource.ts:739`, `apps/admin-web/src/App.tsx:198`).
- Storage staging and VFS repair are already a rich read-only/mutation-adjacent operator surface: staging diagnostics, latest-failure refresh, action plan, remediation plan, target preview, refresh, automation plan, enqueue, execute, and retry are all wired (`crates/nako-server/src/http/admin.rs:441`, `crates/nako-server/src/http/admin.rs:1980`, `crates/nako-server/src/app/storage.rs:1039`, `crates/nako-server/src/app/storage.rs:1057`, `crates/nako-server/src/app/storage.rs:1091`, `crates/nako-server/src/app/storage.rs:1119`, `crates/nako-server/src/app/storage.rs:1168`, `crates/nako-server/src/app/storage.rs:1180`, `crates/nako-server/src/app/storage.rs:1225`, `crates/nako-server/src/app/storage.rs:1265`, `crates/nako-server/src/app/storage.rs:1296`). The Admin Web storage page already renders the live staging/repair surface (`apps/admin-web/src/features/storage/StorageStagingPage.tsx:102`, `apps/admin-web/src/features/storage/StorageStagingPage.tsx:112`, `apps/admin-web/src/features/storage/StorageStagingPage.tsx:187`, `apps/admin-web/src/features/storage/StorageStagingPage.tsx:426`, `apps/admin-web/src/features/storage/StorageStagingPage.tsx:551`).

### Residual gaps

- No incident-bundle route, DTO, or Admin Web page exists yet. The repo search only found documentation references, not a product surface.
- Jobs drilldown is still narrow. The roadmap and control-plane docs explicitly keep `admin-jobs-retry-and-drilldown-gap-<job-kind>` as the follow-on when a concrete durable job class lacks safe retry/cancellation/detail (`docs/architecture/CONTROL_PLANE.md:186`, `docs/ROADMAP.md:69`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:78`). In code, `AdminJobDiagnostics` only expands `VfsCacheRepair` today (`crates/nako-api/src/admin/operations.rs:171`), so there is no general job-detail diagnostic model yet.
- Source hash evidence detail or duplicate-suggestion diagnostics are explicitly called out as a narrower follow-on if needed (`docs/architecture/CONTROL_PLANE.md:128`), but that is still a smaller gap than a full incident bundle.

## Files Found

- `.trellis/tasks/06-11-u3-follow-on-slice/prd.md` - task goal, constraints, and acceptance criteria for the bundle slice.
- `.trellis/tasks/06-11-u3-follow-on-slice/research/jellyfin-gap-analysis.md` - Jellyfin operator/controller pattern comparison.
- `.trellis/tasks/06-11-u3-follow-on-slice/research/plex-benchmark.md` - Plex operator journey benchmark and slice recommendation.
- `docs/architecture/CONTROL_PLANE.md` - control-plane maturity map; incident bundles are still future work.
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` - M1 diagnostics/repair coverage and deferred gaps.
- `docs/ROADMAP.md` - M1/M2 operator maturity targets and follow-on routing.
- `docs/deployment/RELEASE_CHECKLIST.md` - support bundle expectations and safe redaction boundary.
- `docs/deployment/REMOTE_ACCESS.md` - support bundle redaction guidance around network/log exposure.
- `crates/nako-server/src/http/admin.rs` - Admin route registration and handler composition.
- `crates/nako-server/src/app/source_hash.rs` - source fingerprint hash app service and overview pressure summary.
- `crates/nako-server/src/app/source_duplicate.rs` - source duplicate reconciliation plan/apply service.
- `crates/nako-server/src/app/storage.rs` - staging/VFS repair app service.
- `crates/nako-server/src/app/jobs.rs` - scan-originated source hash enqueue and job runtime integration.
- `crates/nako-api/src/admin.rs` - overview and server config diagnostics DTOs.
- `crates/nako-api/src/admin/operations.rs` - redacted job list item/diagnostics and duplicate reconciliation DTOs.
- `crates/nako-api/src/admin_contract.rs` - generated Admin contract surface for overview, jobs, playback, storage, and reconciliation.
- `apps/admin-web/src/App.tsx` - Admin route inventory and navigation.
- `apps/admin-web/src/adminApi/client.ts` - Admin HTTP client methods for the shipped surfaces.
- `apps/admin-web/src/adminApi/dataSource.ts` - live/mock data source wiring for Admin Web pages.
- `apps/admin-web/src/features/overview/OverviewPage.tsx` - overview projection of shipped diagnostics.
- `apps/admin-web/src/features/jobs/JobsPage.tsx` - jobs queue and action surface.
- `apps/admin-web/src/features/storage/StorageStagingPage.tsx` - storage staging and VFS repair surface.
- `apps/admin-web/src/features/items/ItemDetailPage.tsx` - playback/support adjacent navigation entry.

## Code Patterns

- `docs/architecture/CONTROL_PLANE.md:48`, `docs/architecture/CONTROL_PLANE.md:49`, `docs/architecture/CONTROL_PLANE.md:350` - safe incident bundles are future work; crash/fault bundles are not started.
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:53`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:57`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:60`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:66`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:67`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:78` - M1 readiness overview, source duplicate repair, jobs visibility, system/config diagnostics, incident bundles, and job drilldown follow-on routing.
- `docs/ROADMAP.md:32`, `docs/ROADMAP.md:69` - M1 operator release target and the job drilldown follow-on condition.
- `docs/deployment/RELEASE_CHECKLIST.md:257`, `docs/deployment/RELEASE_CHECKLIST.md:263`, `docs/deployment/RELEASE_CHECKLIST.md:264` - safe support bundle expectations already exist.
- `crates/nako-server/src/http/admin.rs:213`, `crates/nako-server/src/http/admin.rs:323`, `crates/nako-server/src/http/admin.rs:327`, `crates/nako-server/src/http/admin.rs:331`, `crates/nako-server/src/http/admin.rs:335`, `crates/nako-server/src/http/admin.rs:339`, `crates/nako-server/src/http/admin.rs:441`, `crates/nako-server/src/http/admin.rs:486`, `crates/nako-server/src/http/admin.rs:493`, `crates/nako-server/src/http/admin.rs:497`, `crates/nako-server/src/http/admin.rs:501`, `crates/nako-server/src/http/admin.rs:1461`, `crates/nako-server/src/http/admin.rs:1522`, `crates/nako-server/src/http/admin.rs:1980`, `crates/nako-server/src/http/admin.rs:3167`, `crates/nako-server/src/http/admin.rs:3196`, `crates/nako-server/src/http/admin.rs:3220`, `crates/nako-server/src/http/admin.rs:3243`, `crates/nako-server/src/http/admin.rs:3267`, `crates/nako-server/src/http/admin.rs:3337`, `crates/nako-server/src/http/admin.rs:3358` - route registration and handler coverage for overview, jobs, source hash, duplicate reconciliation, storage staging, system config, playback runtime, and playback support evidence.
- `crates/nako-api/src/admin.rs:310`, `crates/nako-api/src/admin.rs:540` - overview DTO and safe summary serialization.
- `crates/nako-api/src/admin/operations.rs:121`, `crates/nako-api/src/admin/operations.rs:171`, `crates/nako-api/src/admin/operations.rs:307`, `crates/nako-api/src/admin/operations.rs:335` - redacted job list item and duplicate reconciliation DTOs.
- `crates/nako-api/src/admin_contract.rs:2339`, `crates/nako-api/src/admin_contract.rs:3183`, `crates/nako-api/src/admin_contract.rs:3205`, `crates/nako-api/src/admin_contract.rs:3260`, `crates/nako-api/src/admin_contract.rs:3702`, `crates/nako-api/src/admin_contract.rs:3957`, `crates/nako-api/src/admin_contract.rs:4099`, `crates/nako-api/src/admin_contract.rs:4486` - generated Admin contract for overview, jobs, playback support, storage staging, and system config.
- `crates/nako-server/src/app/source_hash.rs:95`, `crates/nako-server/src/app/source_hash.rs:134`, `crates/nako-server/src/app/source_hash.rs:157`, `crates/nako-server/src/app/source_hash.rs:168`, `crates/nako-server/src/app/source_hash.rs:176`, `crates/nako-server/src/app/source_hash.rs:189`, `crates/nako-server/src/app/source_hash.rs:289`, `crates/nako-server/src/app/source_hash.rs:337`, `crates/nako-server/src/app/source_hash.rs:420`, `crates/nako-server/src/app/source_hash.rs:526` - source hash enqueue, retry, overview pressure, execution, and idempotency.
- `crates/nako-server/src/app/source_duplicate.rs:35`, `crates/nako-server/src/app/source_duplicate.rs:107`, `crates/nako-server/src/app/source_duplicate.rs:169` - plan/apply flow and canonical action handling.
- `crates/nako-server/src/app/storage.rs:1039`, `crates/nako-server/src/app/storage.rs:1057`, `crates/nako-server/src/app/storage.rs:1091`, `crates/nako-server/src/app/storage.rs:1105`, `crates/nako-server/src/app/storage.rs:1119`, `crates/nako-server/src/app/storage.rs:1134`, `crates/nako-server/src/app/storage.rs:1168`, `crates/nako-server/src/app/storage.rs:1180`, `crates/nako-server/src/app/storage.rs:1211`, `crates/nako-server/src/app/storage.rs:1225`, `crates/nako-server/src/app/storage.rs:1265`, `crates/nako-server/src/app/storage.rs:1296` - staging/VFS repair diagnostics and commands.
- `crates/nako-server/src/app/jobs.rs:1107`, `crates/nako-server/src/app/jobs.rs:1178` - scan-originated source-hash enqueue after indexing.
- `apps/admin-web/src/App.tsx:126`, `apps/admin-web/src/App.tsx:198`, `apps/admin-web/src/App.tsx:220`, `apps/admin-web/src/App.tsx:228`, `apps/admin-web/src/App.tsx:240`, `apps/admin-web/src/App.tsx:330`, `apps/admin-web/src/App.tsx:339`, `apps/admin-web/src/App.tsx:340`, `apps/admin-web/src/App.tsx:342` - Admin route inventory and nav entries already cover the shipped surfaces.
- `apps/admin-web/src/adminApi/client.ts:144`, `apps/admin-web/src/adminApi/client.ts:601`, `apps/admin-web/src/adminApi/client.ts:641`, `apps/admin-web/src/adminApi/client.ts:692`, `apps/admin-web/src/adminApi/client.ts:799` - Admin HTTP client methods for overview, jobs, playback support, storage staging, and system config.
- `apps/admin-web/src/adminApi/dataSource.ts:213`, `apps/admin-web/src/adminApi/dataSource.ts:233`, `apps/admin-web/src/adminApi/dataSource.ts:313`, `apps/admin-web/src/adminApi/dataSource.ts:320`, `apps/admin-web/src/adminApi/dataSource.ts:524`, `apps/admin-web/src/adminApi/dataSource.ts:530`, `apps/admin-web/src/adminApi/dataSource.ts:739`, `apps/admin-web/src/adminApi/dataSource.ts:745` - live/mock wiring for overview, jobs, duplicate reconciliation, and storage staging.
- `apps/admin-web/src/features/overview/OverviewPage.tsx:166`, `apps/admin-web/src/features/overview/OverviewPage.tsx:173`, `apps/admin-web/src/features/overview/OverviewPage.tsx:185`, `apps/admin-web/src/features/overview/OverviewPage.tsx:193`, `apps/admin-web/src/features/overview/OverviewPage.tsx:211` - overview already renders source-hash, storage, and metadata diagnostics.
- `apps/admin-web/src/features/jobs/JobsPage.tsx:87`, `apps/admin-web/src/features/jobs/JobsPage.tsx:93`, `apps/admin-web/src/features/jobs/JobsPage.tsx:103`, `apps/admin-web/src/features/jobs/JobsPage.tsx:121`, `apps/admin-web/src/features/jobs/JobsPage.tsx:432` - jobs queue, cancel/retry/execute actions, and queue pressure UI.
- `apps/admin-web/src/features/storage/StorageStagingPage.tsx:102`, `apps/admin-web/src/features/storage/StorageStagingPage.tsx:112`, `apps/admin-web/src/features/storage/StorageStagingPage.tsx:187`, `apps/admin-web/src/features/storage/StorageStagingPage.tsx:426`, `apps/admin-web/src/features/storage/StorageStagingPage.tsx:551` - storage staging and VFS repair UI.
- `apps/admin-web/src/features/items/ItemDetailPage.tsx:172` - existing support card navigation makes playback support evidence a natural adjacent surface.

## External References

- Local Jellyfin reference material under `repo-ref/jellyfin` and the task-local `jellyfin-gap-analysis.md`.
- Task-local Plex benchmark notes in `plex-benchmark.md`.
- No internet sources were used for this pass.

## Related Specs

- `.trellis/spec/nako-server/backend/index.md`
- `.trellis/spec/nako-server/backend/http-api-patterns.md`
- `.trellis/spec/nako-server/backend/error-handling.md`
- `.trellis/spec/nako-server/backend/logging-guidelines.md`
- `.trellis/spec/nako-server/backend/quality-guidelines.md`
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
- `.trellis/spec/nako-vfs/backend/index.md`
- `.trellis/spec/admin-web/frontend/index.md`
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`

## Caveats / Not Found

- `python3 ./.trellis/scripts/task.py current --source` returned no active task in session state, so this note was written to the task directory already named in the conversation context.
- I did not find any existing incident-bundle product route or Admin Web page in the repository.
- The conclusion that incident bundle export is the best one-day follow-on is an inference from current docs and code, not a previously recorded roadmap decision.
