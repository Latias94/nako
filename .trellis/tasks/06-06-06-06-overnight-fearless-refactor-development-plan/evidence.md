# Evidence

## 2026-06-07 M1 / Storage Planning

- Created and committed the continuous development plan:
  `1ce7cd0f chore(task): plan overnight fearless refactor development`.
- Recorded completed M1 evidence gates in roadmap/goal docs:
  `f8eac5fe docs(roadmap): record completed M1 evidence gates`.
- Recorded OpenDAL adapter-spike decision instead of adding a production
  dependency:
  `c035a813 docs(storage): record OpenDAL adapter spike decision`.

## 2026-06-07 Source Fingerprint Hash Trace Correlation

What changed:

- Added optional normalized `request_id` to `SourceFingerprintHashJobInput`
  with safe serde round-tripping.
- Propagated HTTP `x-request-id` into Admin enqueue and scan-originated
  source-hash jobs, then restored it into durable job trace context during
  execution.
- Added trace-aware enqueue wrappers in the source-hash app service and
  execution-span instrumentation for the durable source-hash path.
- Updated the source-hash policy and architecture notes to record the
  correlation contract for trace-safe durable input.

Boundaries:

- No production dependency, schema migration, or API route shape changed.
- `request_id` is correlation metadata only; unsafe values are rejected and
  not echoed.
- Source-hash execution still validates durable input and source bindings
  before reloading the current Media Source.

Validation:

- `cargo fmt --all`
  - Result: passed.
- `cargo check -p nako-library --tests`
  - Result: passed.
- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-library source_hash --no-fail-fast`
  - Result: passed, 17 tests run.
- `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`
  - Result: passed, 33 tests run.

## 2026-06-07 VFS Byte Range Refactor

Commits:

- `07743077 refactor(vfs): centralize byte range validation`
- `4d1ec9d1 fix(vfs): validate open-ended WebDAV range length`
- `0f58cc0e fix(vfs): reject invalid WebDAV range syntax`

What changed:

- Moved shared byte-range boundary validation onto `ByteRange`.
- Removed duplicated local/WebDAV range validation helpers.
- Fixed WebDAV open-ended range reads so `bytes=1-` expects `object_len - 1`
  bytes rather than the whole object length.
- Added syntax validation before WebDAV constructs a `Range` header, so
  zero-length and overflowing ranges reject before sending invalid remote
  requests when object metadata lacks length.

Validation:

- `cargo nextest run -p nako-vfs byte_range --no-fail-fast`
  - Result: passed, 9 tests run after syntax-validation slice.
- `cargo nextest run -p nako-vfs webdav_backend_reads_open_ended_byte_ranges_with_resolved_length --no-fail-fast`
  - Result: passed, 1 test run.
- `cargo nextest run -p nako-vfs --no-fail-fast`
  - Result after open-ended fix: 58 tests passed.
  - Result after syntax-validation fix: 59 tests passed.
- `cargo check -p nako-vfs --tests`
  - Result: passed after both VFS range fixes.

## 2026-06-07 VFS Cache Repair Durable Enqueue

What changed:

- Added `JobKind::VfsCacheRepair` and the persisted
  `storage.vfs.cache_repair` resource class.
- Added `VfsCacheRepairJobInput` as the durable repair input contract, derived
  from existing `VfsCacheFailure` facts using source scheme, operation,
  failure timestamp/count, URI digest, and stored failure authority.
- Added internal `StorageDiagnosticsAppService::enqueue_vfs_cache_repair_target`
  for opaque unresolved repair targets that recommend `refresh_cache`.
- Kept enqueue non-mutating: no backend stat/list refresh, no purge/delete,
  no invalidation, no backend configuration mutation, and no library file write.
- Made queued/running enqueue idempotent by validated input equality, including
  duplicates beyond the first paginated durable job page; terminal jobs do not
  block future enqueue.
- Updated `docs/architecture/STORAGE_VFS.md`,
  `docs/architecture/CONTROL_PLANE.md`, and Trellis code-specs with the shipped
  contract and follow-on boundaries.

Boundaries:

- No Admin/Public API route or DTO was added.
- No durable repair executor, scheduler loop, retry/requeue route, schema
  migration, cache purge/delete/invalidation, backend configuration mutation,
  or automated repair worker was added.
- Raw `StorageUri`, local paths, backend URLs, credentials, raw backend errors,
  etags, fingerprints, and cache payloads remain outside durable job input.

Validation:

- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-server vfs_cache_repair_target_enqueue --no-fail-fast`
  - Result: passed, 4 tests run.
- `cargo nextest run -p nako-core vfs_cache_repair --no-fail-fast`
  - Result: passed, 5 tests run.
- `cargo nextest run -p nako-server runtime_job_resource_class_mapping_maps_known_jobs_to_budget_classes --no-fail-fast`
  - Result: passed, 1 test run.

## 2026-06-07 VFS Cache Repair Internal Executor

What changed:

- Added internal `StorageDiagnosticsAppService::execute_vfs_cache_repair_job`
  for one explicit `JobKind::VfsCacheRepair` job id.
- Added a claimed-job helper for future scheduler integration without
  re-claiming already leased work.
- The executor claims through `DurableJobRuntime`, validates kind/resource/input
  and bindings, reloads the current unresolved cache failure by
  `VfsCacheRepairJobInput::matches_failure`, and then reuses the existing
  selected-target refresh authority.
- Added `VfsCacheRepairJobSummary` with only action, source scheme, operation,
  classification, failure class, failed-at timestamp, failure count, and
  refreshed cache state.
- Stale durable input that no longer matches an unresolved failure fails without
  a backend call and persists only a safe durable job error.
- Updated `docs/architecture/STORAGE_VFS.md`,
  `docs/architecture/CONTROL_PLANE.md`, and Trellis code-specs with the shipped
  internal executor boundary.

Boundaries:

- No Admin/Public API route or DTO was added.
- No automatic scheduler loop, retry/requeue route, schema migration,
  purge/delete/invalidation, backend configuration mutation, library file
  write, or automated repair worker was added.
- Raw `StorageUri`, local paths, backend URLs, credentials, raw backend errors,
  etags, fingerprints, cache payloads, and job input JSON remain outside job
  summary JSON and persisted execution errors.

Validation:

- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-server vfs_cache_repair_job_executor --no-fail-fast`
  - Result: passed, 2 tests run.

## 2026-06-07 VFS Cache Repair Durable Executor

What changed:

- Added internal `StorageDiagnosticsAppService::execute_vfs_cache_repair_job`
  and `execute_claimed_vfs_cache_repair_job` for one explicit
  `JobKind::VfsCacheRepair` job.
- The executor claims through `DurableJobRuntime` with exact job kind/resource
  filters, validates safe durable input, and reselects the current unresolved
  repair target through `VfsCacheRepairJobInput::matches_failure`.
- Backend-touching refresh reuses the existing selected-target refresh
  authority, preserving stored failure authority, ambiguous-backend rejection,
  and `refresh_cache` recommendation checks.
- Added `VfsCacheRepairJobSummary` with only action, scheme, operation,
  classification, failure class, failure timestamp/count, and refreshed cache
  state.
- Storage execution errors are persisted with only the source scheme and a
  redacted target marker.
- Updated storage/control-plane architecture docs and Trellis code-specs to
  mark internal single-job execution shipped while keeping scheduler/API work
  as follow-on scope.

Boundaries:

- No Admin/Public API route or DTO was added.
- No durable scheduler loop, retry/requeue route, schema migration,
  purge/delete/invalidation, backend configuration mutation, library file
  write, or automated repair worker was added.
- Raw `StorageUri`, local paths, backend URLs, credentials, raw backend errors,
  etags, fingerprints, cache payloads, and job input JSON remain outside
  durable summary/error surfaces.

Validation:

- `cargo fmt --all -- --check`
  - Result: passed.
- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-server vfs_cache_repair_job_executor --no-fail-fast`
  - Result: passed, 3 tests run.
- `cargo nextest run -p nako-server runtime_job_resource_class_mapping_maps_known_jobs_to_budget_classes --no-fail-fast`
  - Result: passed, 1 test run.

## 2026-06-07 VFS Cache Repair Admin Manual Commands

What changed:

- Added Admin-only manual enqueue route:
  `POST /admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs`.
- Added Admin-only explicit execution route:
  `POST /admin/v1/storage/vfs-cache/repair/jobs/{job_id}/execute`.
- Added `nako-api` Admin DTOs for VFS cache repair enqueue requests,
  enqueue outcome responses, safe repair job summaries, and execution
  responses.
- Updated generated Admin TypeScript contracts for `apps/admin-web` and `web`.
- The enqueue route accepts only an opaque selected-target `target_ref` and
  optional job priority, then delegates to
  `StorageDiagnosticsAppService::enqueue_vfs_cache_repair_target`.
- The execute route accepts only an explicit durable `JobId`, then delegates to
  `StorageDiagnosticsAppService::execute_vfs_cache_repair_job`.
- Responses expose only generic `AdminJobListItem` facts, enqueue outcome, and
  redaction-safe repair summary fields.
- Updated storage/control-plane architecture docs and Trellis server/API specs
  with the shipped Admin manual command boundary.

Boundaries:

- No automatic scheduler loop was added.
- No retry/requeue route was added.
- No schema migration was added.
- No purge/delete/invalidation, backend configuration mutation, library file
  write, or automated repair worker was added.
- Routes do not accept raw `StorageUri`, local path, backend URL, URI digest,
  job input JSON, cache payload, etag, fingerprint, credential, or raw backend
  error material.

Validation:

- `cargo fmt --all -- --check`
  - Result: passed.
- `cargo check -p nako-api -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - Result: passed, 8 tests run.
- `cargo nextest run -p nako-api admin_vfs_cache_repair_job_commands --no-fail-fast`
  - Result: passed, 1 test run.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  - Result: passed, 1 test run.
- `cargo nextest run -p nako-server admin_v1_vfs_cache --no-fail-fast`
  - Result: passed, 11 tests run.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-overnight-fearless-refactor-development-plan`
  - Result: passed.
- `git diff --check`
  - Result: passed.

## 2026-06-07 VFS Cache Repair Scheduler Integration

What changed:

- Added `JobKind::VfsCacheRepair` to the existing disk-scan scheduler candidate
  set through a filtered durable queue window for
  `storage.vfs.cache_repair`.
- The scheduler exact-claims the selected VFS cache repair job once, then
  passes the `LeasedJob` to
  `StorageDiagnosticsAppService::execute_claimed_vfs_cache_repair_job`.
- The supervised background task uses the existing `disk.scan` runtime budget
  and keeps the scan permit alive until repair execution finishes or fails.
- Successful scheduler execution persists only the redaction-safe
  `VfsCacheRepairJobSummary`; storage failures persist only the existing
  redacted durable error shape.
- Updated `docs/architecture/STORAGE_VFS.md` to mark disk-scan scheduler
  integration shipped while leaving retry/requeue, purge/delete/invalidation,
  backend mutation, and automated repair policy as follow-ons.

Boundaries:

- No Admin/Public API route, public DTO, schema migration, config shape, or
  production dependency changed.
- No retry/requeue route, purge/delete/invalidation behavior, backend
  configuration mutation, library file write, or automated repair policy was
  added.
- Raw `StorageUri`, local paths, backend URLs, credentials, raw backend errors,
  etags, fingerprints, cache payloads, and durable input JSON remain outside
  scheduler summaries, errors, diagnostics, and logs.

Validation:

- `cargo fmt --package nako-server -- --check`
  - Result: passed.
- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-server vfs_cache_repair_scheduler --no-fail-fast`
  - Result: passed, 4 tests run.
- `cargo nextest run -p nako-server vfs_cache_repair --no-fail-fast`
  - Result: passed, 24 tests run.
- `cargo nextest run -p nako-server vfs_cache_repair_job_executor --no-fail-fast`
  - Result: passed, 3 tests run.
- `cargo nextest run -p nako-server runtime_job_resource_class_mapping_maps_known_jobs_to_budget_classes --no-fail-fast`
  - Result: passed, 1 test run.
- `cargo nextest run -p nako-server source_fingerprint_hash_scheduler --no-fail-fast`
  - Result: passed, 4 tests run.
- `cargo nextest run -p nako-server job_scheduler --no-fail-fast`
  - Result: passed, 2 tests run.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-overnight-fearless-refactor-development-plan`
  - Result: passed.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.

## 2026-06-07 VFS Cache Repair Internal Retry Seam

What changed:

- Added an internal
  `StorageDiagnosticsAppService::retry_vfs_cache_repair_job` command that
  creates a new queued durable retry from a failed `JobKind::VfsCacheRepair`
  job while preserving the failed source job as audit history.
- The retry command validates the failed job kind, resource class, durable
  input, library/source bindings, failed status, and a current unresolved
  `refresh_cache` target before delegating row creation to
  `JobRepository::enqueue_job_retry`.
- Delayed retry timestamps are parsed as RFC3339 and persisted as canonical UTC
  RFC3339 through a shared `app::job_retry` helper also used by source
  fingerprint hash retry.
- Due VFS cache repair retries continue through the existing disk-scan
  scheduler and executor path; future retries remain queued and unclaimable
  until due.
- Updated `docs/architecture/STORAGE_VFS.md`,
  `docs/architecture/CONTROL_PLANE.md`, and
  `.trellis/spec/nako-server/backend/quality-guidelines.md` to mark only the
  internal retry seam shipped while keeping Admin retry/requeue routes,
  purge/delete/invalidation, backend mutation, library file writes, and
  automated repair policy as follow-ons.

Boundaries:

- No Admin/Public API route, public DTO, schema migration, config shape,
  production dependency, cache purge/delete/invalidation behavior, backend
  configuration mutation, library file write, or automated repair policy was
  added.
- The retry seam does not accept raw `StorageUri`, target refs, local paths,
  backend URLs, URI digests, durable input JSON, cache payloads, etags,
  fingerprints, credentials, or raw backend error material from callers.
- Failed source jobs remain failed; retries are new queued jobs linked by
  `retry_of_job_id`.

Validation:

- `cargo fmt --package nako-server -- --check`
  - Result: passed.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-overnight-fearless-refactor-development-plan`
  - Result: passed.
- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-server vfs_cache_repair_retry --no-fail-fast`
  - Result: passed, 3 tests run.
- `cargo nextest run -p nako-server source_fingerprint_hash_retry --no-fail-fast`
  - Result: passed, 11 tests run.
- `cargo nextest run -p nako-server vfs_cache_repair --no-fail-fast`
  - Result: passed, 27 tests run.
- Independent check agent `Leibniz`
  - Result: no blocking or non-blocking issues; recommended committing the
    internal VFS cache repair retry seam.

## 2026-06-07 VFS Cache Repair Admin Retry Route

What changed:

- Added the Admin manual retry route
  `POST /admin/v1/storage/vfs-cache/repair/jobs/{job_id}/retry` with route key
  `storageVfsCacheRepairJobRetry`.
- Added `AdminVfsCacheRepairRetryRequest { max_attempts, next_attempt_at }` to
  the Admin API contract and regenerated both Admin TypeScript contract copies.
- The HTTP handler is a thin boundary: it parses the durable `JobId` and retry
  request body, delegates to
  `StorageDiagnosticsAppService::retry_vfs_cache_repair_job`, and returns
  `202 Accepted` with only `AdminJobListItem` safe job facts.
- Added route tests proving successful retry creation, invalid retry states
  without retry rows, non-admin rejection, response redaction, route inventory
  parity, and API contract parity.
- Updated architecture and Trellis specs to mark Admin manual retry as shipped
  while leaving purge/delete/invalidation, backend configuration mutation,
  library file writes, broader operator diagnostics, and automated repair
  policy as follow-ons.

Boundaries:

- No generic durable job retry route was added.
- No cache purge/delete/invalidation behavior, backend configuration mutation,
  library file write, automated repair policy, schema migration, config shape,
  or production dependency changed.
- Retry responses expose no retry linkage, attempt counters, durable
  `input_json`, durable `summary_json`, raw durable errors, URI/path/token,
  etag, fingerprint, URI digest, backend URL, credential, or cache payload
  material.

Validation:

- `cargo fmt --package nako-api --package nako-server -- --check`
  - Result: passed.
- `cargo check -p nako-api -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - Result: passed, 8 tests run.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  - Result: passed, 1 test run.
- `cargo nextest run -p nako-server admin_v1_vfs_cache --no-fail-fast`
  - Result: passed, 13 tests run.
- `cargo nextest run -p nako-server vfs_cache_repair_retry --no-fail-fast`
  - Result: passed, 5 tests run.
- `cargo nextest run -p nako-api admin_vfs_cache_repair --no-fail-fast`
  - Result: passed, 5 tests run.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-overnight-fearless-refactor-development-plan`
  - Result: passed.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.

## 2026-06-07 PostgreSQL Storage/Source Runtime Parity Refresh

What changed:

- No production code, schema, migration, API, or harness command surface was
  changed.
- Refreshed current-HEAD PostgreSQL runtime evidence after the VFS cache repair
  durable enqueue, executor, scheduler, and retry/Admin command slices.
- Exercised the existing focused PostgreSQL contract harness suites for the
  storage runtime and source identity query paths named in the M2 reliability
  follow-on pool.
- Verified the local temporary PostgreSQL 17 cluster starts, runs the focused
  ignored contracts, stops, and leaves no listener on port `55432`.
- Updated `docs/architecture/STORAGE_VFS.md` so the focused PostgreSQL
  storage/source parity evidence is marked refreshed while broader runtime
  parity remains a follow-on.

Validation:

- Current commit: `57ff4413`.
- Local tooling:
  - `initdb.exe`, `pg_ctl.exe`, and `createdb.exe` were available from
    `F:\MySoftware\PostgreSQL\17\bin`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite storage-runtime -RequireTooling`
  - Result: passed.
  - PostgreSQL ignored contracts: 4 tests run, 4 passed, 176 skipped.
  - Covered storage backend health, VFS staging listing/failure/summary,
    attribution variants, reservation budget, and lease preservation.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite source-identity -RequireTooling`
  - Result: passed.
  - PostgreSQL ignored contracts: 6 tests run, 6 passed, 174 skipped.
  - Covered library-scoped Media Source identity, scan source-unit writes,
    Source Duplicate Relationship upsert/pair lookup/fingerprint matching, and
    VFS staging attribution/budget contracts.
- `Get-NetTCPConnection -LocalPort 55432 -ErrorAction SilentlyContinue`
  - Result: no listener reported after harness cleanup.

## 2026-06-07 PostgreSQL Storage/Source Parity Harness Alias

What changed:

- Added a first-class `storage-source-parity` PostgreSQL harness suite in both
  `scripts/postgres-contract-harness.ps1` and
  `scripts/postgres-contract-harness.sh`.
- The new suite is a combined M2 storage-VFS reliability entry point that runs
  the existing `storage-runtime` and `source-identity` filters in one harness
  pass.
- Updated `docs/workstreams/self-hosted-release-readiness/DESIGN.md` and
  `docs/workstreams/self-hosted-release-readiness/EVIDENCE_AND_GATES.md` so the
  combined suite is discoverable.
- Updated `.trellis/spec/nako-db/backend/quality-guidelines.md` so the suite
  list and harness-selection guidance stay in sync with the scripts.

Boundaries:

- No production Rust code, schema, migration, API route, or generated contract
  changed.
- Existing `managed-artwork`, `storage-runtime`, `source-identity`, and
  `all-contracts` suite behavior stayed intact.

Validation:

- `bash -n scripts/postgres-contract-harness.sh`
  - Result: passed, though the WSL shim printed its usual environment noise.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite storage-source-parity -RequireTooling`
  - Result: passed.
  - PostgreSQL ignored contracts: 8 tests run, 8 passed, 172 skipped.
  - Covered storage backend health, VFS staging listing/failure/summary,
    library-scoped Media Source identity, scan source-unit writes, source
    duplicate lookup, and VFS staging attribution/reservation contracts.

## 2026-06-07 VFS Cache Repair Admin Web Seam

What changed:

- Added typed Admin Web client methods for the VFS cache repair read and
  command routes:
  - `GET /admin/v1/storage/vfs-cache/repair/action-plan`
  - `GET /admin/v1/storage/vfs-cache/repair/remediation-plan`
  - `GET /admin/v1/storage/vfs-cache/repair/targets`
  - `GET /admin/v1/storage/vfs-cache/repair/targets/{target_ref}/preview`
  - `POST /admin/v1/storage/vfs-cache/repair/refresh-cache`
  - `POST /admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache`
  - `POST /admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs`
  - `POST /admin/v1/storage/vfs-cache/repair/jobs/{job_id}/execute`
  - `POST /admin/v1/storage/vfs-cache/repair/jobs/{job_id}/retry`
- Exported the generated VFS cache repair response/request types through the
  Admin Web API type facade.
- Added deterministic contract-level mock fixtures for action plans,
  remediation plans, repair targets, target previews, refresh responses,
  enqueue responses, execution summaries, queued jobs, and retry jobs.
- Updated the Storage Staging mock summary so `vfs_cache.repair` carries the
  same redaction-safe repair diagnostic used by the route fixtures.
- Added `AdminDataSource` read-model fallback methods for action plan,
  remediation plan, targets, and target preview.
- Added `AdminDataSource` mutation methods for latest refresh, target refresh,
  enqueue, execute, and retry. Mutations delegate directly to the live client
  and do not fabricate mock success.
- Added data-source regression tests for route/query generation, `{target_ref}`
  and `{job_id}` URL encoding, command request bodies, read-model fallback, and
  mutation failure propagation.
- Corrected mock executable-action route metadata to match the backend
  contract: latest action-plan uses
  `storageVfsCacheRepairRefreshCache`, while remediation groups keep the
  target-scoped `{target_ref}` route template.

Boundaries:

- No generated Admin API contract files were edited by hand.
- No backend route, schema, migration, public API, page/UI, config shape,
  production dependency, cache purge/delete/invalidation behavior, backend
  configuration mutation, library file write, or automated repair policy was
  changed.
- Fixtures and test assertions expose only Admin-safe job and repair facts; no
  raw `StorageUri`, local path, backend URL, credentials, raw backend errors,
  etags, fingerprints, URI digest, durable input JSON, or cache payload
  material was added.

Validation:

- `cargo fmt --all`
  - Result: passed.
- `cargo check -p nako-library --tests`
  - Result: passed.
- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-library source_hash --no-fail-fast`
  - Result: passed, 17 tests run.
- `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`
  - Result: passed, 33 tests run.
- `npm run check --prefix apps/admin-web`
  - Result: passed.
- `npm run test --prefix apps/admin-web -- src/adminApi/dataSource.test.ts`
  - Result: passed, 33 tests run.
- `npm run test --prefix apps/admin-web -- src/adminApi/client.test.ts`
  - Result: passed, 20 tests run.
- `npm run test --prefix apps/admin-web`
  - Result: passed, 7 test files and 180 tests run.
- `npm run build --prefix apps/admin-web`
  - Result: passed; Vite reported the existing chunk-size warning.
- `git diff --check -- apps/admin-web/src/adminApi/client.ts apps/admin-web/src/adminApi/types.ts apps/admin-web/src/adminApi/mockData.ts apps/admin-web/src/adminApi/dataSource.ts apps/admin-web/src/adminApi/dataSource.test.ts`
  - Result: passed with only Git LF/CRLF working-copy warnings.
- Independent check agent `Darwin`
  - Result: found missing remediation-plan seam and a fixture route-key drift;
    both were fixed before commit.
- Independent check agent `Godel`
  - Result: no findings after the remediation-plan seam and executable-action
    fixture fixes.

## 2026-06-07 VFS Cache Repair Storage Staging UI Actions

What changed:

- Added a Storage Staging route panel for VFS cache repair operator context.
- The panel reads the VFS cache repair action plan, remediation plan, and
  target list through `AdminDataSource` page seams and preserves deterministic
  mock fallback when those read methods are unavailable.
- Added bounded live-only actions for:
  - latest VFS cache refresh;
  - enqueueing the first returned `refresh_cache` repair target with normal
    priority.
- Added visible success/error notices for the repair mutations and invalidated
  the route-local Storage Staging query group after successful commands.
- The route-level refresh action now reloads both Storage Staging and VFS cache
  repair read models.
- The repair panel shows its own source label so hybrid staging-live/
  repair-fallback states are visible.
- Mutation disabled notices now wait for read loading to finish and include the
  specific missing live data, route, or enqueueable-target reason.
- Rendered repair action groups, target rows, readiness, boundary facts,
  classification counts, retryability, and safe repair messages without raw
  storage locators or backend paths.
- Added English and zh-Hans `storage.repair.*` route copy.
- Added App route tests covering repair context rendering, mock fallback and
  disabled mutation state, and live refresh/enqueue command calls.

Boundaries:

- No generated Admin API contract files, backend route, schema, migration,
  public API, config shape, production dependency, cache purge/delete/
  invalidation behavior, backend configuration mutation, library file write, or
  automated repair policy was changed.
- The page only exposes redaction-safe Admin repair facts already available via
  typed Admin Web fixtures and data-source methods.
- Mutations remain disabled unless the relevant read models are live and the
  corresponding command method exists.

Validation:

- `npm run check --prefix apps/admin-web`
  - Result: passed.
- `npm run test --prefix apps/admin-web -- App.test.tsx`
  - Result: passed, 102 tests run.
- `npm run test --prefix apps/admin-web`
  - Result: passed, 7 test files and 183 tests run.
- `npm run build --prefix apps/admin-web`
  - Result: passed; Vite reported the existing chunk-size warning.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.
- Independent implementation review agent `Ramanujan`
  - Result: improved route refresh coverage, repair source labeling,
    enqueue-target selection, and disabled reason handling; reported missing
    new i18n keys before validation, which were fixed before this evidence was
    finalized.

## 2026-06-07 M1 RC Closeout Audit

Conclusion:

- Product-Operator M1 is RC-ready except publication on current HEAD
  `5c64f2a6`.
- No named M1 blocker was found in the authoritative closeout docs reviewed
  for this audit.

Evidence reviewed:

- `docs/GOALS.md` records the roadmap/goal/lane reconciliation as completed
  and says follow-on M1 release-candidate evidence passed `release-fast`,
  `playback`, `container`, `postgres`, and `workspace`.
- `docs/ROADMAP.md` routes new M1 implementation only from release-ladder or
  Admin coverage-matrix blockers, and no unconditional M1 candidate remains.
- `docs/architecture/LANES.md` keeps operations-release and adjacent M1 lanes
  idle unless a concrete failed ladder mode or coverage-matrix opening
  condition appears.
- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md` documents all runner modes:
  `docs`, `smoke`, `fast`, `release-fast`, `playback`, `container`,
  `postgres`, `workspace`, and `all`.
- Archived evidence records:
  - `release-fast` passed in
    `.trellis/tasks/archive/2026-06/06-06-m1-release-fast-evidence-run/`;
  - `playback` passed in
    `.trellis/tasks/archive/2026-06/06-06-m1-playback-evidence-run/`;
  - `container` passed in
    `.trellis/tasks/archive/2026-06/06-06-m1-container-evidence-run/`;
  - `postgres` passed in
    `.trellis/tasks/archive/2026-06/06-06-m1-postgres-evidence-run/`,
    including a separate `all-contracts` PostgreSQL harness pass;
  - `workspace` passed after the timing-gate repair in
    `.trellis/tasks/archive/2026-06/06-06-m1-workspace-evidence-run/`;
  - Product-Operator smoke/fast evidence is recorded in
    `.trellis/tasks/archive/2026-06/06-06-m1-operator-journey-smoke/` and
    `.trellis/tasks/archive/2026-06/06-06-m1-release-ladder-runner/`.

Ladder policy:

- `scripts/m1-release-ladder.ps1 -Mode all` was not rerun in this closeout
  audit because it is expensive and environment-dependent.
- The script proves `Mode all` is an orchestrated sequence of `fast`,
  `release-fast`, `playback`, `container`, `postgres`, and `workspace`, with
  repeated redaction inventory skipped only for the later expensive delegated
  gates.
- For an actual RC publication run, use `Mode all` without
  `-SkipRedactionInventory`; skipped environment-dependent gates must be
  recorded as skipped, not passed, per the evidence matrix.

Packaging dry-run shape:

- Ran
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -WhatIf`
  on current HEAD.
- Result: passed. The script reported package id
  `nako-server-v0.1.0-alpha.2-x86_64-pc-windows-msvc-5c64f2a68ac7` and
  stopped before building or writing release artifacts.
- `scripts/package-release.ps1` declares `SupportsShouldProcess`; when
  `$WhatIfPreference` is set, it exits after reporting that it would
  build/copy release files, write the manifest, archive, and `SHA256SUMS`.
- No release build, tag, publish, archive, manifest, checksum, or generated
  contract was created by this audit.

Known non-blocking publication gaps:

- Actual artifact build/package publication was intentionally not run.
- Crates/image publishing, tags, and release artifacts remain manual
  publication steps outside this closeout audit.
- No live-browser manual playback session proof was added beyond the existing
  smoke, Admin Web, playback, and workspace gate evidence.

## 2026-06-07 OpenDAL Adapter Decision Spike Closeout

Exit signal:

- Decision: defer `storage-opendal-adapter-first-slice`.
- Do not add OpenDAL as a production dependency in the current M1/M2 wave.
- Keep Nako `StorageBackend` as the product boundary for storage authority,
  Source Locator redaction, source fingerprint evidence, cache repair
  authority, storage health, deterministic staging, and Admin-safe diagnostics.
- If a future task opens `storage-opendal-adapter-first-slice`, the first slice
  must be a feature-gated or test-only adapter harness behind `StorageBackend`.
  It must not replace Nako's storage product model, widen WebDAV capability, or
  change production config/API/schema/dependency shape before tests prove the
  boundary is safe.

Evidence reviewed:

- `research/opendal-storage-layer.md` records OpenDAL 0.57.0 as a credible
  Rust storage operator with local filesystem, WebDAV, S3-compatible storage,
  retry, timeout, tracing/metrics, throttle, and capability layers.
- `docs/architecture/STORAGE_VFS.md` defines Nako-owned VFS semantics:
  `StorageUri`, Source Locator redaction, Source Fingerprint evidence, storage
  health, VFS cache repair authority, deterministic staging, range/stream
  behavior, and redaction-safe Admin diagnostics.
- `crates/nako-vfs/src/lib.rs` shows `StorageBackend` is a domain contract,
  not a generic object-store facade: it includes storage capabilities, object
  metadata, byte-range reads, streaming reads, staging reports, local mutation
  planning, default unsupported mutation reports, and cache repair facts.
- `crates/nako-vfs/src/local.rs` keeps local path authority, escape
  prevention, atomic write/backup/restore/cleanup behavior, link planning, and
  local staging under Nako control.
- `crates/nako-vfs/src/webdav.rs` keeps endpoint validation, credential
  redaction, bounded retry/timeout behavior, PROPFIND parsing, range reads,
  streaming reads, deterministic staging, and intentionally read-only product
  behavior under Nako control.

Alternatives considered:

- Continue hand-written local/WebDAV backends for now.
  - Pros: preserves known M1 behavior, avoids dependency churn, keeps current
    redaction/capability/range/cache-repair semantics directly testable.
  - Cons: backend breadth remains slower to add; repeated remote adapter
    plumbing can accumulate.
  - Decision: chosen for the current wave.
- Replace `nako-vfs` backend implementations directly with OpenDAL.
  - Pros: could reduce bespoke filesystem/WebDAV/S3 operation code.
  - Cons: would blur Nako-owned storage semantics, risks widening WebDAV
    write/delete/copy/rename behavior, and would force error/range/runtime
    policy remapping before product value is proven.
  - Decision: rejected for M1/M2.
- Add a narrow OpenDAL adapter behind `StorageBackend`.
  - Pros: may prove future backend breadth, especially S3-compatible storage,
    while preserving Nako's product boundary.
  - Cons: still needs explicit mapping for redaction, capability narrowing,
    range/stream behavior, storage failure classes, and runtime policy; adding
    the dependency before a committed backend target is premature.
  - Decision: deferred. Reopen only as
    `storage-opendal-adapter-first-slice` with a feature-gated/test-only
    harness and no production behavior change.

Risk and verification requirements for any future first slice:

- Redaction: adapter tests must prove raw `StorageUri`, local paths, backend
  URLs, credentials, headers, raw provider errors, etags, fingerprints, URI
  digests, durable input JSON, and cache payloads do not cross API,
  diagnostic, log, or durable job summary boundaries.
- Capability narrowing: tests must prove OpenDAL-advertised write/delete/copy/
  rename support cannot widen Nako's backend capabilities or WebDAV read-only
  product policy.
- Range reads and streaming: tests must prove bounded ranges, open-ended
  ranges, invalid range syntax, full-object streams, unknown-length objects,
  and non-seekable backends map to existing `ByteRange`, `read_range`, and
  `stream_range` behavior without forcing whole-object loads where streaming
  is required.
- Error mapping: tests must map OpenDAL failures into Nako storage failure
  classes and cache repair diagnostics without leaking raw backend messages or
  turning permission/security failures into retry loops.
- Runtime policy: retry/timeout/throttle/tracing layers must remain subordinate
  to Nako's runtime budgets, storage health/circuit-breaker policy, scan/probe/
  playback admission, and deterministic staging cleanup.

Validation policy:

- This closeout is docs-only. No Rust/TypeScript code, generated contract,
  dependency, `Cargo.toml`, or `Cargo.lock` change is required or allowed.
- Rust tests are intentionally skipped because no code changed.

## 2026-06-07 Storage Staging Purpose/State Diagnostics

What changed:

- Added redaction-safe `purpose_state_summaries` to Admin Storage Staging
  diagnostics.
- Each summary groups staging manifest records by `purpose` and `state` and
  exposes only aggregate facts:
  - `record_count`;
  - `used_manifest_bytes`;
  - `active_leases`;
  - `unknown_size_records`.
- The aggregation is folded into the existing staging manifest pressure scan,
  so the Admin staging request does not add a second full manifest pagination
  pass for this summary.
- Updated `nako-api` Admin DTOs and `admin_contract.rs`, then regenerated both
  Admin TypeScript contract outputs:
  - `apps/admin-web/src/adminApi/generated/contract.ts`;
  - `web/src/api/admin/generated/contract.ts`.
- Added a Storage Staging page purpose/state summary table with English and
  zh-Hans copy, deterministic mock data, and route test coverage.
- Added backend route assertions proving the summary is computed from the full
  manifest set, not just the filtered page records.

Boundaries:

- This is a read-only diagnostics slice.
- No schema migration, public API route, production dependency, OpenDAL
  dependency, backend configuration mutation, cache purge/delete/invalidation,
  library file write, or automated repair policy was added.
- The new payload does not expose raw paths, Source Locators, `source_uri`,
  etags, fingerprints, raw backend errors, credentials, tokens, or durable input
  JSON.
- `web/src/api/admin/generated/contract.ts` was regenerated because the
  `nako-api` Admin contract tests compare both generated Admin contract copies.

Validation:

- `npm run generate:admin-api --prefix apps/admin-web`
  - Result: passed.
- `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
  - Result: passed.
- `cargo fmt --all`
  - Result: completed formatting.
- `cargo fmt --all -- --check`
  - Result: passed.
- `cargo check -p nako-api --tests`
  - Result: passed.
- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo check -p nako-api -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - Result: passed, 8 tests run.
- `cargo nextest run -p nako-server admin_v1_storage_staging_lists_filters_and_redacts_paths --no-fail-fast`
  - Result: passed, 1 test run.
- `npm run check --prefix apps/admin-web`
  - Result: passed.
- `npm run test --prefix apps/admin-web -- App.test.tsx`
  - Result: passed, 102 tests run.
- `npm run test --prefix apps/admin-web`
  - Result: passed, 7 test files and 183 tests run.
- `npm run build --prefix apps/admin-web`
  - Result: passed; Vite reported the existing chunk-size warning.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-overnight-fearless-refactor-development-plan`
  - Result: passed.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.
- Independent check agent `Kepler`
  - Result: no blocking findings; confirmed the slice is read-only,
    redaction-safe, contract-synchronized, and reuses the existing manifest
    pressure scan instead of adding a second full scan.

Residual risk:

- Full workspace Rust nextest was not run. Focused API contract, server route,
  Admin Web typecheck/test/build, formatting, and whitespace gates passed.

## 2026-06-07 Admin Web VFS Cache Repair Job Commands

What changed:

- Added live-only row actions to the Jobs route for VFS cache repair jobs.
- Queued `vfs_cache_repair` jobs with resource class
  `storage.vfs.cache_repair` can now call the existing
  `executeVfsCacheRepairJob` Admin Web data-source command.
- Failed `vfs_cache_repair` jobs with resource class
  `storage.vfs.cache_repair` can now call the existing
  `retryVfsCacheRepairJob` Admin Web data-source command.
- Non-repair jobs render an explicit no-action label, and repair jobs in other
  states render a no-state-action label instead of an unsafe command.
- Mock/fallback Jobs data disables repair job commands and shows a visible
  live-data requirement notice.
- Success and error notices expose only safe job id/status facts and invalidate
  the route-local Jobs query group after successful commands.
- Added English and zh-Hans copy for the new Jobs actions column, command
  labels, live-data requirement, success notices, and errors.
- Added App route tests for live execute/retry delegation, disabled mock
  fallback commands, localized action-column copy, and the existing VFS repair
  filter copy after the new row action aria labels were introduced.

Boundaries:

- This is a frontend-only operator command reachability slice over existing
  Admin Web data-source methods.
- No backend route, API DTO, generated contract, schema migration, production
  dependency, runtime behavior, cache purge/delete/invalidation behavior,
  backend configuration mutation, library file write, or automated repair
  policy changed.
- The Jobs route still does not render durable input JSON, summary JSON,
  retry linkage, raw durable errors, raw `StorageUri`, local paths, backend
  URLs, credentials, URI digests, etags, fingerprints, or cache payloads.
- Commands remain unavailable unless the Jobs read model is live and the
  corresponding data-source method exists.

Spec update review:

- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md` already covers
  this pattern: route/page data goes through `AdminDataSource`, mutations are
  live-only, visible errors are rendered for unavailable commands, route tests
  assert command payloads, localized copy, and mock fallback behavior.
- No new reusable convention or gotcha was discovered, so no code-spec update
  was needed for this slice.

Validation:

- `npm run check --prefix apps/admin-web`
  - Result: passed.
- `npm run test --prefix apps/admin-web -- App.test.tsx`
  - Result: passed, 103 tests run.
- `npm run test --prefix apps/admin-web`
  - Result: passed, 7 test files and 184 tests run.
- `npm run build --prefix apps/admin-web`
  - Result: passed; Vite reported the existing chunk-size warning.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-overnight-fearless-refactor-development-plan`
  - Result: passed.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.
- Independent check agent `Einstein`
  - Result: no blocking findings; confirmed the route still delegates through
    `AdminDataSource`, execute/retry are live-only, execute and retry response
    shapes are handled separately, and the page does not render durable input,
    summary, raw error, URI/path, credential, or cache payload fields.

Residual risk:

- Full workspace Rust nextest was not run because this slice only changes
  Admin Web route command reachability and localized copy over existing Admin
  API contracts.

## 2026-06-07 Admin Web VFS Cache Repair Job Filter

What changed:

- Added a Jobs page quick filter for VFS cache repair jobs.
- The button applies:
  - `kind=vfs_cache_repair`;
  - `resource_class=storage.vfs.cache_repair`;
  - `offset=0`;
  - `source_id=undefined`.
- The filter preserves any existing `library_id` filter through the route-owned
  partial search update pattern.
- Added English and zh-Hans Jobs filter copy.
- Added route test coverage for URL search params, data-source payload, active
  button state, and localized copy.
- Updated deterministic Admin Web mock Jobs data so the VFS cache repair job is
  visible in local/mock mode.
- Corrected the mock queued VFS cache repair job `resource_class` from
  `storage.vfs_cache.repair` to `storage.vfs.cache_repair`.

Boundaries:

- This is a frontend-only operator navigation slice.
- No backend DTO, Admin API route, generated contract, schema migration,
  production dependency, runtime behavior, repair execution, cache mutation, or
  deletion behavior changed.
- The filter only narrows existing Jobs reads; it does not enqueue, retry,
  cancel, repair, purge, or inspect raw job input.

Spec update review:

- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md` already covers
  this pattern: URL-owned route filters, filter changes resetting `offset`,
  i18n copy, mock fallback data, and route tests asserting URL params plus
  data-source payloads.
- No new reusable convention or gotcha was discovered, so no code-spec update
  was needed for this slice.

Validation:

- `npm run check --prefix apps/admin-web`
  - Result: passed.
- `npm run test --prefix apps/admin-web -- App.test.tsx`
  - Result: passed, 102 tests run.
- `npm run test --prefix apps/admin-web`
  - Result: passed, 7 test files and 183 tests run.
- `npm run build --prefix apps/admin-web`
  - Result: passed; Vite reported the existing chunk-size warning.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-overnight-fearless-refactor-development-plan`
  - Result: passed.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.
- Independent check agent `Parfit`
  - Result: no blocking findings; confirmed the slice is frontend-only,
    redaction-safe, and does not require backend/API/generated/schema/dependency
    changes.

Residual risk:

- Full workspace Rust nextest was not run because this slice only changes
  Admin Web route/filter behavior and mock data.

## 2026-06-07 Storage Staging Cleanup Candidate Purpose/State Diagnostics

What changed:

- Added redaction-safe `cleanup_purpose_state_summaries` to Admin Storage
  Staging diagnostics.
- Each summary groups cleanup candidate staging manifest records by `purpose`
  and `state` and exposes only aggregate facts:
  - `record_count`;
  - `cleanup_candidate_bytes`;
  - `active_leases`;
  - `unknown_size_records`.
- The aggregation is folded into the existing cleanup candidate pressure scan,
  so the Admin staging request does not add a second cleanup candidate
  pagination pass for this summary.
- Updated `nako-api` Admin DTOs and `admin_contract.rs`, then regenerated both
  Admin TypeScript contract outputs:
  - `apps/admin-web/src/adminApi/generated/contract.ts`;
  - `web/src/api/admin/generated/contract.ts`.
- Added a Storage Staging page cleanup candidate purpose/state summary table
  with English and zh-Hans copy, deterministic mock data, and route test
  coverage.
- Added backend route assertions proving cleanup candidate summaries come only
  from cleanup candidates, not from all manifest records.

Boundaries:

- This is a read-only diagnostics slice.
- No schema migration, public API route, production dependency, OpenDAL
  dependency, backend configuration mutation, cache purge/delete/invalidation,
  library file write, or automated repair policy was added.
- The new payload does not expose raw paths, Source Locators, `source_uri`,
  etags, fingerprints, raw backend errors, credentials, tokens, or durable input
  JSON.
- `web/src/api/admin/generated/contract.ts` was regenerated because the
  `nako-api` Admin contract tests compare both generated Admin contract copies.

Validation:

- `npm run generate:admin-api --prefix apps/admin-web`
  - Result: passed.
- `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
  - Result: passed.
- `cargo fmt --all`
  - Result: completed formatting.
- `cargo fmt --all -- --check`
  - Result: passed.
- `cargo check -p nako-api -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - Result: passed, 8 tests run.
- `cargo nextest run -p nako-server admin_v1_storage_staging_lists_filters_and_redacts_paths --no-fail-fast`
  - Result: passed, 1 test run.
- `npm run check --prefix apps/admin-web`
  - Result: passed.
- `npm run test --prefix apps/admin-web -- App.test.tsx`
  - Result: passed, 102 tests run.
- `npm run test --prefix apps/admin-web`
  - Result: passed, 7 test files and 183 tests run.
- `npm run build --prefix apps/admin-web`
  - Result: passed; Vite reported the existing chunk-size warning.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-overnight-fearless-refactor-development-plan`
  - Result: passed.
- `git diff --check`
  - Result: passed with only Git LF/CRLF working-copy warnings.
- Independent check agent `Volta`
  - Result: no blocking findings; confirmed the slice is read-only,
    redaction-safe, contract-synchronized, and reuses the existing cleanup
    candidate pressure scan instead of adding a second full scan.

Residual risk:

- Full workspace Rust nextest was not run. Focused API contract, server route,
  Admin Web typecheck/test/build, formatting, and whitespace gates passed.
