# Control Plane API/DB Boundary Review

## Scope Notes

This review focused on the Admin API / server app-service / repository / DB
Adapter / generated contract path. The main sample slice was recent VFS cache
repair work because it crosses `nako-api`, `nako-server`, `nako-core`,
`nako-db`, Admin Web generated contracts, and the storage control plane.

The recommendations do not propose public compatibility preservation for weak
pre-compatibility shapes. They use the ADR 0019 / ADR 0027 / ADR 0053 direction:
thin HTTP handlers, focused app-service Modules, redacted Admin API
diagnostics, bounded list surfaces, and control-plane behavior that is not
rebuilt per feature.

## Top Opportunities

### 1. Deepen VFS cache repair target querying behind a repair-target Module

#### Files

- `crates/nako-server/src/app/storage.rs:459`
- `crates/nako-server/src/app/storage.rs:504`
- `crates/nako-server/src/app/storage.rs:617`
- `crates/nako-server/src/app/storage.rs:874`
- `crates/nako-core/src/repository/vfs.rs:11`
- `crates/nako-core/src/repository/vfs.rs:33`
- `crates/nako-db/src/sqlite/vfs_cache.rs:232`
- `crates/nako-db/src/postgres/vfs_staging.rs:259`
- `crates/nako-db/src/facade.rs:2754`
- `crates/nako-db/src/contract_tests.rs:7229`

#### Problem

The current `VfsCacheRepository` Interface exposes raw cache failure records,
while the product concept needed by the Admin API is an unresolved, redaction
safe repair target. `StorageDiagnosticsAppService` therefore has to page through
raw failures, call `vfs_cache_failure_resolved_by_cache`, rebuild
`VfsCacheRepairDiagnostic`, generate `target_ref`, and then re-scan the same
failure list to resolve a preview. That makes the app-service Implementation
own both repository access mechanics and Admin repair-target semantics.

The Interface is shallow: callers ask for failures but really need repair
targets. The deletion test says if `list_vfs_cache_failures(PageRequest)` were
deleted, the target inventory logic would not disappear; it would need to
reappear as a better query or deeper Module.

#### Proposed refactor

Introduce a focused VFS cache repair target Module at the repository/app-service
Seam. The external Interface should speak in repair-target terms:

- list unresolved VFS cache repair targets with bounded pagination;
- resolve one opaque target handle to the current unresolved target;
- keep raw cache URI, Source Locator, backend identity, etag, Source
  Fingerprint, and raw error text inside the Implementation.

There are two viable depths:

- Smaller task: keep DB rows unchanged, but create an internal
  `VfsCacheRepairTargetQuery` Module inside `nako-server::app::storage` that
  owns the paging loop, resolved filtering, HMAC target refs, and preview
  lookup. `StorageDiagnosticsAppService` delegates to it.
- Deeper task: add repository methods such as
  `list_unresolved_vfs_cache_repair_targets(page, now/cache evidence)` only if
  resolved filtering can be expressed without leaking app storage-backend
  knowledge into `nako-db`.

#### Deletion/deepening angle

Delete the direct repair-target pagination loops from
`StorageDiagnosticsAppService`. Deepen the repair target Interface so callers
receive Admin-ready target facts rather than recomputing unresolved state from
raw failures.

The best first move is an internal deep Module, not a new crate. There is only
one real Adapter today, and ADR 0019 prefers internal module deepening before
adding more seams.

#### Test impact

- Existing DB contract tests for raw failure ordering still matter.
- Add app-service tests around the new repair-target Module for pagination
  after resolved filtering, stale/unknown target refs, and HMAC opacity.
- Existing HTTP tests for `/admin/v1/storage/vfs-cache/repair/targets` and
  target preview should become thinner and assert only route mapping/redaction.

#### Risk/ADR conflicts

No ADR conflict. This strengthens ADR 0053 by keeping bounded Admin list
surfaces and redaction inside one Locality. A DB-level unresolved-target query
could conflict with server storage authority if it tries to decide resolution
from SQL alone; avoid that unless the repository contract gains explicit cache
freshness evidence.

#### Suggested workflow scale

Small-to-medium Trellis implementation task. No schema migration required for
the internal Module version. Broaden to a storage/repository task only if the
query is moved into `nako-core` / `nako-db`.

### 2. Split Storage diagnostics into Storage Health, Staging Pressure, and VFS Cache Repair Modules

#### Files

- `crates/nako-server/src/app/storage.rs:36`
- `crates/nako-server/src/app/storage.rs:336`
- `crates/nako-server/src/app/storage.rs:650`
- `crates/nako-server/src/app/storage.rs:688`
- `crates/nako-server/src/app/storage.rs:722`
- `crates/nako-server/src/app/storage.rs:970`
- `crates/nako-server/src/app/storage.rs:1015`
- `.trellis/spec/nako-server/backend/directory-structure.md`
- `.trellis/spec/nako-server/backend/database-guidelines.md`

#### Problem

`StorageDiagnosticsAppService` currently groups several different control-plane
concepts under one broad Module: Storage Backend Health, Storage Circuit
Breaker reset, staging manifest pressure, staging cleanup pressure, staging
budget policy, process cached backend count, VFS cache summary, VFS cache repair
action plan, and VFS cache repair target refs.

The Interface is becoming wide enough that unrelated workflows share the same
state and helper namespace. For example, staging pressure and VFS cache repair
both use bounded repository paging, but they have different product language,
different redaction risks, and different tests. Locality is weak: a future
Storage Backend Health change can accidentally touch VFS cache repair helpers
because both live in the same app-service file and Module.

#### Proposed refactor

Keep `StorageDiagnosticsAppService` as a facade-style handle returned by
`NakoApp::storage()`, but move the Implementation into focused internal
Modules:

- `storage/health.rs`: Storage Backend Health and Storage Circuit Breaker
  diagnostics/reset;
- `storage/staging_pressure.rs`: staging manifest pressure, cleanup pressure,
  budget policy slices, and scan admission helpers;
- `storage/vfs_cache_repair.rs`: VFS cache summary repair preview, action plan,
  targets, HMAC handles, and refresh action.

`StorageBackendRegistry` can remain the shared Adapter for configured backends,
but each focused Module should expose a small Interface and own its own tests.

#### Deletion/deepening angle

Delete the single 2.8k-line mixed storage Implementation shape. Deepen each
Module around one operator workflow. The facade keeps call-site churn small
while improving Locality behind the Seam.

#### Test impact

- Move pure pressure/status/ref tests next to the relevant internal Module.
- Preserve route tests through the existing Admin API.
- Run `cargo check -p nako-server --tests` and focused storage filters.
- If no public DTO changes occur, `nako-api` gates are not required.

#### Risk/ADR conflicts

No ADR conflict. This directly follows ADR 0019's thin composition root and
focused app-service direction. Main risk is accidental visibility churn in
tests; keep `pub(crate)` exports minimal from `app.rs`.

#### Suggested workflow scale

Medium fearless-refactor task. It can be done without schema/API changes if the
public app-service handle remains stable.

### 3. Make Admin route inventory the single source for Router wiring and generated contract paths

#### Files

- `crates/nako-api/src/admin_contract.rs:3`
- `crates/nako-api/src/admin_contract.rs:203`
- `crates/nako-api/src/admin_contract.rs:3537`
- `crates/nako-server/src/http/admin.rs:160`
- `crates/nako-server/src/http/admin.rs:366`
- `apps/admin-web/src/adminApi/generated/contract.ts:74`
- `web/src/api/admin/generated/contract.ts:74`

#### Problem

The Admin API path list lives in at least two places: Axum route wiring in
`http/admin.rs` and `ADMIN_ROUTE_SUFFIXES` in `admin_contract.rs`. Recent VFS
cache routes also needed executable-action constants in `http/admin.rs` for
route key/path guidance. The tests verify drift after the fact, but the Interface
for adding a route remains shallow: every caller must remember the route path,
route key, generated contract entry, and any executable-action metadata.

This weakens Leverage. The Module does not yet let a new Admin route be declared
once and reused for handler wiring, generated contracts, route-key references,
and redaction tests.

#### Proposed refactor

Create an Admin route inventory Module that is small enough for `nako-api` to
own route keys and paths, while `nako-server` owns handler registration.

Possible shape:

- `nako_api::admin_routes` owns typed route keys and canonical `/admin/v1/*`
  path templates;
- `admin_contract.rs` generates `NAKO_ADMIN_ROUTES` from that typed inventory;
- `nako-server::http::admin` imports route constants instead of duplicating
  strings for route wiring and executable-action DTOs;
- route inventory tests assert every canonical route has server registration
  and generated contract output.

Do not put Axum handlers in `nako-api`. The Seam is route metadata, not HTTP
Implementation.

#### Deletion/deepening angle

Delete duplicated route path strings and local executable-action path constants.
Deepen the route inventory Interface so the Admin API path/key pair becomes a
reusable control-plane contract, not an after-the-fact generated artifact.

#### Test impact

- Existing `admin_contract_includes_route_constants` remains, but should test a
  typed inventory instead of a local array.
- Add a server route inventory test if feasible to catch missing handler wiring.
- Regenerated Admin Web contract files should be unchanged except ordering if
  the inventory preserves current order.

#### Risk/ADR conflicts

No ADR conflict. ADR 0027 wants a separate versioned Admin API and generated
Admin contract; this makes that Seam deeper. The risk is over-coupling
`nako-api` to Axum if handler details leak into the inventory. Keep only route
metadata in `nako-api`.

#### Suggested workflow scale

Medium Trellis task. It touches `nako-api`, `nako-server`, both generated
contracts, and tests, but should not need DB or runtime changes.

### 4. Replace monolithic string-built Admin TypeScript contract body with module-scoped contract fragments

#### Files

- `crates/nako-api/src/admin_contract.rs:203`
- `crates/nako-api/src/admin_contract.rs:226`
- `crates/nako-api/src/admin_contract.rs:3537`
- `crates/nako-api/src/admin/storage.rs:1`
- `crates/nako-api/src/admin/automation.rs:1`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`

#### Problem

`admin_contract.rs` has a typed route suffix list but a very large raw
`CONTRACT_BODY` string. The Rust DTOs are split into Admin modules, while the
TypeScript contract generation is centralized as a single string literal and a
large test that checks for expected names. This makes the generated contract
Module shallow: adding a DTO means manually synchronizing Rust DTOs, a giant
TypeScript string, generated files, and string-presence tests.

The Interface is high-friction because maintainers must understand the whole
Admin contract body to add a local storage or Generated Artifact DTO. Locality
is poor: storage DTO changes and Addon DTO changes share one generated body
file.

#### Proposed refactor

Split the Admin TypeScript contract generator into module-scoped fragments
that match `nako-api::admin` modules:

- route inventory fragment;
- shared pagination/version/error fragment;
- `storage` fragment;
- `automation` / Generated Artifact and Acceptance Workflow fragment;
- `playback`, `access`, `addon`, and other existing fragments.

Each fragment can remain hand-authored TypeScript if full serde-to-TypeScript
generation is not desired yet. The deepening comes from Locality: `storage.rs`
DTO changes update a `storage_contract_fragment()` and storage-specific tests,
not a 3k-line body.

#### Deletion/deepening angle

Delete the single monolithic `CONTRACT_BODY` Interface. Deepen contract
generation by giving each Admin domain Module its own contract fragment and
focused expected-type tests.

#### Test impact

- Keep `admin_web_generated_contract_matches_generator_output`.
- Replace the single giant expected-name list with module-specific tests.
- Existing generated contract files can remain byte-for-byte stable if fragment
  ordering is deterministic.

#### Risk/ADR conflicts

No ADR conflict. This preserves ADR 0027 and the spec rule that Admin Web
contracts are generated from `nako-api`. It does not require adding a new code
generation dependency.

#### Suggested workflow scale

Medium cleanup task. It is a pure API-contract generator refactor if output is
kept stable; run focused `nako-api admin_contract` plus generated artifact drift
tests.

### 5. Split Generated Artifact provider config from Acceptance Workflow persistence and app orchestration

#### Files

- `crates/nako-core/src/repository/automation.rs:18`
- `crates/nako-server/src/app/automation.rs:95`
- `crates/nako-server/src/app/automation.rs:290`
- `crates/nako-server/src/app/automation.rs:362`
- `crates/nako-server/src/app/automation.rs:736`
- `crates/nako-server/src/app/automation.rs:1346`
- `crates/nako-server/src/app/automation.rs:1628`
- `crates/nako-db/src/sqlite/automation.rs:276`
- `crates/nako-db/src/sqlite/automation.rs:520`
- `crates/nako-db/src/sqlite/automation.rs:640`
- `crates/nako-db/src/postgres/addons_automation.rs:1024`
- `crates/nako-db/src/postgres/addons_automation.rs:1268`

#### Problem

`AutomationRepository` combines Automation Provider configuration, Automation
Artifact rows, Generated Artifact proposals, metadata apply outcomes, recovery
entries, bulk apply batches, and batch status transitions. The app-service
Implementation then also owns provider normalization, Acceptance Workflow
planning, metadata patch parsing, durable job creation, runtime execution,
idempotency keys, error redaction, and outcome persistence.

This is a broad control-plane Module with a large Interface. It reduces
Locality for the Nako concepts that matter: Automation Provider and Generated
Artifact Acceptance Workflow are related but not the same Module. Changes to
provider config should not require mentally loading bulk metadata apply batch
state transitions.

#### Proposed refactor

Split along domain and control-plane seams:

- `AutomationProviderRepository`: provider config and enabled-provider lists;
- `AutomationArtifactRepository`: raw automation artifact creation/status/list;
- `GeneratedArtifactAcceptanceRepository`: proposal views, apply outcomes,
  recovery entries, bulk apply batches, and state transitions.

In `nako-server`, split `AutomationAppService` internally:

- provider administration Module;
- Generated Artifact proposal/readiness query Module;
- Acceptance Workflow planner Module;
- bulk apply durable-job runner Module.

The public `NakoApp::automation()` handle can remain a facade while the
Implementation deepens.

#### Deletion/deepening angle

Delete the one-repository-does-everything Interface and the one app-service
Implementation carrying both provider config and Acceptance Workflow runtime.
Deepening improves Leverage because tests can cross the same focused Seam that
future Admin API handlers use.

#### Test impact

- Repository contract tests must be split by trait family while preserving
  SQLite/Postgres parity.
- App tests should move planning/idempotency/job-runner assertions to the
  focused Acceptance Workflow Module.
- Admin DTO mapping tests in `nako-api::admin::automation` remain useful and
  should not drive repository shape.

#### Risk/ADR conflicts

No direct ADR conflict, but this is larger than the storage candidates. ADR
0053 supports it because Generated Artifact bulk apply is durable control-plane
work and should not be hidden in a one-off feature helper. Risk is high because
it touches repository traits, SQLite/Postgres Adapters, facade dispatch,
durable job execution, and Admin API handlers.

#### Suggested workflow scale

Large fearless-refactor workstream or architecture lane. Do not combine with
Admin route inventory or storage refactors. Start with repository trait split
and contract-test reorganization, then app-service Module extraction.

## Priority Ranking

1. **VFS cache repair target query Module**: highest immediate Leverage because
   it fixes a fresh cross-layer pattern before more storage repair operations
   copy it.
2. **Storage diagnostics internal Module split**: strong Locality win with
   limited public-surface risk; pairs well after the VFS repair query Module.
3. **Admin route inventory single-source Module**: important before Admin Web
   route count grows further; medium cross-crate surface.
4. **Admin TypeScript contract fragment split**: valuable maintainability
   cleanup, best done after route inventory or alongside a contract-focused
   task with stable generated output.
5. **Generated Artifact Acceptance Workflow repository/app-service split**:
   architecturally high-value but large. Treat as a dedicated fearless-refactor
   lane after smaller control-plane seams prove the pattern.
