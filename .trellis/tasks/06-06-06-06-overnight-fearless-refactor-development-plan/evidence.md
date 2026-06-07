# Evidence

## 2026-06-07 M1 / Storage Planning

- Created and committed the continuous development plan:
  `1ce7cd0f chore(task): plan overnight fearless refactor development`.
- Recorded completed M1 evidence gates in roadmap/goal docs:
  `f8eac5fe docs(roadmap): record completed M1 evidence gates`.
- Recorded OpenDAL adapter-spike decision instead of adding a production
  dependency:
  `c035a813 docs(storage): record OpenDAL adapter spike decision`.

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
