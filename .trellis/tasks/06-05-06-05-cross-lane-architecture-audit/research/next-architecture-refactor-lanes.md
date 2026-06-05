# Next Architecture Refactor Lanes

Audit date: 2026-06-05

Scope: decide whether Nako should continue with fearless refactor /
architecture deepening before more parallel product development, and identify
the module hotspots that currently serialize otherwise independent lanes.

Constraints: research only; no production code, commits, pushes, destructive
git commands, or broad formatting.

## Inputs

- `CONTEXT.md`
- `docs/ARCHITECTURE.md`
- `docs/architecture/LANES.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/STATE_ACCESS.md`
- ADR 0016, 0017, 0038, 0044, 0045, 0052, 0053
- `.trellis/spec/guides/index.md`
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
- `.trellis/spec/nako-api/backend/quality-guidelines.md`
- `.trellis/spec/nako-server/backend/directory-structure.md`
- `.trellis/spec/nako-server/backend/http-api-patterns.md`
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
- parent research notes under
  `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/`
- child task PRDs under `.trellis/tasks/06-05-*`

Useful evidence commands run:

```powershell
git status --short
git log --since="2026-05-01" --name-only --pretty=format: -- crates/nako-* apps/admin-web web docs/architecture .trellis/spec
rg --files crates apps/admin-web web
rg -n "ADMIN_ROUTE_SUFFIXES|ClientPlaybackCapabilities|SourceFingerprintHash|AddonResource|NetworkAccessConfig" crates apps/admin-web web docs/architecture .trellis/spec
```

`git status --short` had no output before this research file was written.

## Current Task State

The older parent synthesis recommended the Admin route inventory parity gate as
the first contract-safety task. That gate now appears to be implemented:

- `.trellis/tasks/06-05-admin-route-inventory-parity-gate/prd.md` has all
  acceptance criteria checked.
- `crates/nako-api/src/admin_contract.rs` now exposes
  `AdminContractRoute`, `AdminContractRouteExclusion`,
  `admin_contract_routes()`, `admin_contract_route_exclusions()`, and
  `normalize_admin_route_path()`.
- `crates/nako-server/src/http/tests/admin_route_inventory.rs` contains
  `implemented_admin_routes_are_generated_or_explicitly_excluded`.

The playback capability audit also has a recorded outcome in
`.trellis/tasks/06-05-playback-output-profile-device-capability-audit/prd.md`.
It recommends a Public Client playback capability parity gate before a v2
profile skeleton.

This changes the next decision: do not re-recommend the already completed
Admin route parity gate. The remaining question is which deeper serial
contracts should happen before feature lanes start editing the same surfaces.

## Parallel Development Bottlenecks

### Admin API, Generated Admin Contract, And Admin Web Route State

High-conflict files:

- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/addons.rs`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/mockData.ts`

Evidence:

- `admin_contract.rs`, generated Admin TypeScript contracts, and
  `http/admin.rs` are among the highest-churn Nako files since 2026-05-01.
- `admin_contract.rs` is still a wide static generator module even after route
  inventory parity was added.
- `http/admin.rs` and `http/addons.rs` together own broad `/admin/v1/*` route
  registration.
- `apps/admin-web/src/App.tsx` repeats many
  `validate*Search` / `normalize*Search` route-search helpers, and Admin Web
  pages repeatedly reset `offset` on filter changes.

Why it blocks parallel work:

Almost every future lane wants an Admin diagnostic or route: playback support
facts, remote access drill-down, Addon Manager, VFS cache repair, source hash
triggering, durable jobs, and storage diagnostics. Even with route parity in
place, generator output, DTO redaction, mock fixtures, route state, and tests
still serialize.

### Public Client Playback Capability Contract

High-conflict files:

- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-api/src/sdk.rs`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/http/renderer.rs`
- `crates/nako-playback/src/lib.rs`
- `crates/nako-playback/src/capability.rs`
- `docs/api/HTTP_API.md`

Evidence:

- The playback child audit found that `ClientPlaybackCapabilitiesDto`,
  `BrowserPlaybackCapabilitiesDto`, OpenAPI, Rust client builders, SDK query
  surfaces, server query/body mapping, and HTTP docs are not yet protected by a
  single parity gate.
- `nako-playback` has a deep pure planner, but the next profile wave needs
  `profile_id`, `profile_version`, `device_family`, `player_engine`, direct
  play/remux/transcode rows, subtitle delivery rows, audio output facts, and
  color pipeline facts.

Why it blocks parallel work:

HEVC/AV1, hardware tone mapping, image subtitle burn-in, browser/mobile/TV
profiles, renderer capability reporting, and Admin support evidence all want
to touch capability facts. If they start independently, they will invent
incompatible inputs or drift generated client contracts.

### Durable Jobs, Disk Scan Scheduling, And Source Fingerprint Hash

High-conflict files:

- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-server/src/app/job_runtime.rs`
- `crates/nako-server/src/app/runtime.rs`
- `crates/nako-server/src/app/source_hash.rs`
- `crates/nako-library/src/source_hash.rs`
- `crates/nako-core/src/job.rs`
- `crates/nako-core/src/repository/*`
- `crates/nako-db/src/**`

Evidence:

- Source Fingerprint hash now has execution, durable job input, summary JSON,
  internal enqueue, queued execution planning, single-job execution, scheduler
  integration, and evidence persistence.
- `jobs.rs` now schedules both `JobKind::LibraryScan` and
  `JobKind::SourceFingerprintHash` under the disk-scan budget.
- `.trellis/spec/nako-server/backend/directory-structure.md` contains detailed
  source hash scheduler contracts, meaning this surface is already important
  enough to preserve as project guidance.

Why it blocks parallel work:

The next source hash step is policy, not mechanics: scan-originated enqueue,
Admin manual trigger, retry/requeue, automatic scheduling, and Source Duplicate
Relationship mutation all cross scan, source identity, durable jobs, storage
budgets, Admin API, and redaction.

### Playback Runtime Identity, HLS Artifact Authority, And Output Profiles

High-conflict files:

- `crates/nako-playback/src/lib.rs`
- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/values.rs`
- `crates/nako-transcode/src/artifact.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-transcode/src/ffmpeg/hls/*`
- `crates/nako-server/src/app/playback/*`
- `crates/nako-server/src/http/playback.rs`

Evidence:

- `nako-playback` and `nako-transcode` are not shallow in the deletion-test
  sense. Deleting them would re-spread planner, requirement, pipeline,
  artifact, and FFmpeg command logic across server flows.
- `artifact.rs` is a valid hotspot: it owns HLS rendition records, burn-in
  plans, media rendition identity, request variant identity, artifact manifest
  reconstruction, cleanup matching, and many tests.
- `profile.rs` owns deferred HEVC/AV1 output policy and persisted profile
  identity.

Why it blocks parallel work:

HLS artifact identity and `TranscodeProfile` identity are shared by HEVC/AV1,
hardware tone mapping, subtitle execution, adaptive HLS, seek restart, and
player support evidence. Any identity change should be serial-first.

### Addon Host-Owned Resource Flows

High-conflict files:

- `crates/nako-server/src/app/addons/resource_search.rs`
- `crates/nako-server/src/app/addons/subtitles.rs`
- `crates/nako-server/src/app/addons/external_acquisition.rs`
- `crates/nako-server/src/app/addons/task_runtime.rs`
- `crates/nako-server/src/app/addons/event_runtime.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/features/addons/AddonsPage.tsx`

Evidence:

- Resource Search, subtitle import, and external acquisition each have
  selection or selected-reference concepts, grant checks, redaction, safe
  error codes, and host-owned apply/materialization handoff.
- `nako-addon-protocol/src/lib.rs`, `nako-addon-client/src/lib.rs`, and
  `nako-official-addon-catalog/src/lib.rs` are large, but their churn is lower
  and their current responsibility is mostly wire contract / adapter / catalog
  material. Size alone is not enough evidence for a protocol split.

Why it blocks parallel work:

Adding another Addon Resource flow before a server-local pattern exists will
duplicate selection TTL, selected reference identity, apply-plan shape, safe
  error taxonomy, and grant/redaction placement.

### Remote Access And Network Policy

High-conflict files:

- `crates/nako-server/src/config.rs`
- `crates/nako-server/src/config/preflight.rs`
- `crates/nako-server/src/http/network.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-api/src/admin/network.rs`
- `docs/deployment/**`
- `deploy/**`
- `scripts/release-gate.*`

Evidence:

- `NetworkAccessConfig` owns exposure mode, `external_base_url`, trusted proxy
  headers/sources, allowed origins, and tunnel providers.
- `config/preflight.rs` classifies network access readiness for config-check.
- `http/admin.rs` separately maps network readiness into Admin DTOs.
- `http/network.rs` owns trusted forwarded header enforcement.

Why it blocks parallel work:

Docs/config fixtures are safe to parallelize, but endpoint discovery, tunnel
health, trusted proxy expansion, and Admin network drill-down would all share
config, Admin DTOs, auth/CORS/header behavior, and client endpoint contracts.

## Candidate Classification

### Strong: Admin Contract Generator Deepening

Current status:

The route parity gate is done. The next bottleneck is that the Admin TypeScript
contract generator is still a wide static module and every Admin-facing lane
must touch it.

Problem:

`admin_contract.rs` now gives good route inventory leverage, but its Interface
is still shallow for DTO additions: a caller must understand Rust DTO source,
static TypeScript contract body, route suffix table, expected-name assertions,
two generated output copies, Admin Web mocks, and redaction tests.

Recommended deepening:

Run a design-first cleanup that keeps the existing generator but splits contract
authoring by Admin domain or adds typed per-domain generator fragments. Do not
jump directly to a new schema-generation stack unless a short spike proves the
dependency and build cost.

Serial or parallel:

Serial-first. No other task should regenerate Admin contracts while this runs.

Writable scope if opened:

- `crates/nako-api/src/admin_contract.rs`
- possible internal `crates/nako-api/src/admin_contract/*` modules if the
  crate layout is adjusted deliberately
- generated files only via the generator:
  `apps/admin-web/src/adminApi/generated/contract.ts` and
  `web/src/api/admin/generated/contract.ts`
- focused spec/task docs under `.trellis/`

Validation:

```powershell
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast
cargo check -p nako-api -p nako-server --tests
npm run generate:admin-api --prefix apps/admin-web
git diff --check
```

Risk:

High merge churn and easy generated-artifact drift. Keep behavior and wire
shape unchanged in the first cleanup.

### Strong: Public Client Playback Capability Contract Parity Gate

Current status:

The playback output/device capability audit is complete enough to name this as
the next playback contract-safety task.

Problem:

Current flat playback capability facts cross Public Client DTOs, OpenAPI,
Rust client builders, generated SDK query surfaces, server query/body mapping,
renderer mapping, and docs. Future profile fields would amplify any drift.

Recommended deepening:

Add a parity gate for the current capability fields before adding v2 profile
fields. The second task should add an additive optional v2 skeleton that maps
absent profile data to `legacy_default` behavior without changing planner
decisions.

Serial or parallel:

Serial-first for contract changes. Docs/research can continue in parallel, but
HEVC/AV1, hardware tone mapping, image subtitle burn-in, and Admin support
evidence should wait.

Writable scope if opened:

- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-api/src/sdk.rs`
- `crates/nako-client/src/**`
- `crates/nako-client-core/src/**`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/http/renderer.rs`
- `docs/api/HTTP_API.md`
- generated SDK artifacts only through existing generators

Validation:

```powershell
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-client -p nako-client-core --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server renderer --no-fail-fast
cargo check -p nako-client-protocol -p nako-api -p nako-server --tests
git diff --check
```

Risk:

Very high cross-client drift risk. Do not expose FFmpeg, hardware, resource
pressure, or operator policy facts through Public Client capability DTOs.

### Strong: Source Fingerprint Hash Triggering And Reconciliation Policy

Current status:

Execution mechanics are mature. Trigger and reconciliation semantics are not.

Problem:

Persisted Source Fingerprint evidence can now exist, but Nako still needs to
decide whether and when it should enqueue work from scan, allow Admin manual
enqueue, retry/requeue work, or create Source Duplicate Relationship
suggestions. `CONTEXT.md` is explicit: Source Fingerprint is evidence, not
Source identity.

Recommended deepening:

Finish the architecture policy task before implementation. The first
implementation should probably be trigger-only or suggestion-only, not
automatic duplicate mutation.

Serial or parallel:

The audit can run in parallel with Addon and remote docs work. Implementation
is serial with scan scheduling, broad durable job migration, Admin Jobs/DTO
changes, and source identity repository changes.

Writable scope if opened:

- audit docs under `.trellis/tasks/06-05-source-hash-triggering-reconciliation-policy/`
- later implementation in `crates/nako-library/src/source_hash.rs`
- `crates/nako-server/src/app/source_hash.rs`
- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-core/src/job.rs`
- `crates/nako-core/src/repository/*`
- `crates/nako-db/src/**`
- Admin DTO/contract files only with a contract owner

Validation:

```powershell
cargo nextest run -p nako-library source_hash --no-fail-fast
cargo nextest run -p nako-server source_hash --no-fail-fast
cargo nextest run -p nako-server schedule_queued_library_scans --no-fail-fast
cargo check -p nako-core -p nako-library -p nako-api -p nako-server --tests
cargo nextest run -p nako-db --no-fail-fast
git diff --check
```

Risk:

Expensive full hashing, hidden background work, redaction leaks in job input or
summary JSON, and surprising automatic Source Duplicate Relationship mutation.

### Strong: Host-Owned Addon Resource Flow Pattern

Current status:

The Addon Protocol foundation is strong. The pressure is server-side
host-owned resource flow repetition.

Problem:

Resource Search, subtitle import, and external acquisition each have their own
selection/session/apply/redaction pattern. The next Addon Resource would likely
copy that shape again.

Recommended deepening:

Define a server-local pattern for selection session storage, selected reference
identity, apply-plan/apply-result shape, safe error code taxonomy, grant check
placement, and side-effect handoff. Keep protocol crates permissive and avoid
moving host policy into `nako-addon-protocol`.

Serial or parallel:

The audit can run in parallel. Implementation should serialize with any new
Addon Resource flow, Addon Manager API work, or Admin Addon generated contract
work.

Writable scope if opened:

- `.trellis/tasks/06-05-addon-resource-flow-pattern-audit/research/*`
- later server-local implementation under `crates/nako-server/src/app/addons/*`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-api/src/admin/addons*.rs` and `admin_contract.rs` only if DTOs
  are intentionally changed
- no mechanical split of `nako-addon-protocol`, `nako-addon-client`, or
  `nako-official-addon-catalog`

Validation:

```powershell
cargo nextest run -p nako-server addons --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests
npm run test --prefix apps/admin-web -- Addons
git diff --check
```

Risk:

Over-abstracting too early, weakening host-owned side-effect authority, or
creating a protocol-level constraint that future Addons must carry forever.

### Worth Exploring: Network Policy Classifier Deepening

Problem:

`config/preflight.rs` and `http/admin.rs` classify similar exposure, endpoint,
trusted proxy, origin, and tunnel facts into different output models. This is
manageable today but will become drift-prone if endpoint discovery, tunnel
health checks, or richer proxy states are added.

Recommended deepening:

Extract a pure internal classifier only when a new network readiness state is
about to land. The cookbook/config fixture task does not need this refactor.

Serial or parallel:

Docs and fixture work can run in parallel. Classifier implementation is
serial with trusted proxy/header behavior, auth/CORS middleware, Admin network
DTOs, and endpoint discovery.

Writable scope if opened:

- `crates/nako-server/src/config/preflight.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/network.rs`
- `crates/nako-api/src/admin/network.rs`
- `docs/deployment/**` only for docs-only work

Validation:

```powershell
cargo nextest run -p nako-server network --no-fail-fast
cargo nextest run -p nako-server config_preflight --no-fail-fast
cargo nextest run -p nako-api network --no-fail-fast
cargo check -p nako-api -p nako-server --tests
git diff --check
```

Risk:

Trusted proxy semantics are security-sensitive. Do not broaden from narrow
`X-Forwarded-*` handling to RFC `Forwarded`, multi-hop, or provider-specific
logic without a dedicated contract decision.

### Worth Exploring: Admin Web Route Search Helper

Problem:

`apps/admin-web/src/App.tsx` repeats route search validation and normalization
for many pages. This is not a correctness failure, but it makes each Admin Web
page feature touch shared route state and tests.

Recommended deepening:

Extract a small helper for paged route search, common string/enum/integer
normalization, and "filter changes reset offset" tests. Keep TanStack Router
as owner of URL state and do not add a form stack.

Serial or parallel:

Can be done after the current Admin contract owner is idle. Avoid doing it
while a large Admin page feature is actively changing `App.tsx`.

Writable scope if opened:

- `apps/admin-web/src/App.tsx`
- possible route-search helper under `apps/admin-web/src/*`
- `apps/admin-web/src/App.test.tsx`
- no generated contract edits by hand

Validation:

```powershell
npm run check --prefix apps/admin-web
npm run test --prefix apps/admin-web
npm run build --prefix apps/admin-web
npm run verify --prefix apps/admin-web
git diff --check
```

Risk:

Frontend validation app churn. The helper should remove repeated behavior
without changing route URLs, mock fallback, or data-source contracts.

### Worth Exploring: Separate Admin Client From Public Client Bridges

Problem:

Admin Web's Admin API client still carries Public Client bridge methods and
hand-written Public Client response types, while Media Web already uses
`@nako/sdk`.

Recommended deepening:

Move Public Client reads used by Admin management views behind a separate
Public Client data adapter backed by generated SDK surfaces, or make the
bridge explicit outside `AdminApiClient`.

Serial or parallel:

Worth doing only if web-product and client-surface work will run together.
Serialize with Public Client SDK changes.

Writable scope if opened:

- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/types.ts`
- `apps/admin-web/src/surfaces/media/*`
- related tests

Validation:

```powershell
npm run check --prefix apps/admin-web
npm run test --prefix apps/admin-web
npm run build --prefix apps/admin-web
cargo nextest run -p nako-api --no-fail-fast
git diff --check
```

Risk:

Can accidentally blur Admin API and Public Client API semantics in the other
direction. Keep **Admin API** and **Public Client API** vocabulary explicit.

### Worth Exploring Later: Playback Artifact Identity Cleanup

Problem:

`nako-transcode/src/artifact.rs` is large and owns several identity and
manifest concerns. It is a real future conflict surface, but not the first
thing to change before the capability contract gate.

Recommended deepening:

Piggyback cleanup on the first selected playback implementation that needs HLS
artifact or profile identity changes. Candidate split points are HLS request
variant identity parsing, artifact manifest allow-list reconstruction, and
test helper extraction.

Serial or parallel:

Serial with all playback implementation work. Do not run beside HEVC/AV1,
hardware tone mapping, subtitle execution, or HLS seek identity work.

Writable scope if opened:

- `crates/nako-transcode/src/artifact.rs`
- possible focused internal modules under `crates/nako-transcode/src/*`
- `crates/nako-transcode/src/profile.rs` only if identity is intentionally
  touched
- affected server playback tests if behavior is preserved through app flows

Validation:

```powershell
cargo nextest run -p nako-transcode artifact --no-fail-fast
cargo nextest run -p nako-transcode profile --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo check -p nako-playback -p nako-transcode -p nako-server --tests
git diff --check
```

Risk:

Persisted request keys and artifact manifests are compatibility-sensitive.
Behavior-preserving refactor first; do not change identity grammar casually.

### Speculative: Disk Scan Job Executor Registry

Problem:

`jobs.rs` now has repeated scheduling/execution shape for library scan and
source fingerprint hash under `disk.scan`. It looks like an abstraction
candidate, but there are only two disk-scan variants today.

Recommendation:

Defer. Revisit only when a third or fourth disk-scan job kind joins the same
budget path. Until then, a registry would likely be a hypothetical seam.

Validation if later opened:

```powershell
cargo nextest run -p nako-server schedule_queued_library_scans --no-fail-fast
cargo nextest run -p nako-server source_hash --no-fail-fast
cargo nextest run -p nako-server startup --no-fail-fast
cargo check -p nako-server --tests
git diff --check
```

Risk:

Generic scheduler abstractions can hide durable lease, resource-class,
redaction, and exact-claim invariants that are currently explicit in specs.

### Speculative: Mechanical Protocol/Client/Catalog Splits

Large files:

- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-official-addon-catalog/src/lib.rs`

Recommendation:

Defer. They are large, but the current pressure is not mixed persistence,
server policy, or routing. The real pressure is server-side host-owned Addon
resource flow and Admin surface ownership.

Validation if later opened:

```powershell
cargo nextest run -p nako-addon-protocol --no-fail-fast
cargo nextest run -p nako-addon-client --no-fail-fast
cargo nextest run -p nako-official-addon-catalog --no-fail-fast
cargo check -p nako-addon-protocol -p nako-addon-client -p nako-official-addon-catalog --tests
git diff --check
```

Risk:

Mechanical splitting creates churn without improving locality for the modules
that currently block product work.

## Serial-First Work

Run these serially before broad feature work touches the same surfaces:

1. Admin contract generator deepening, if more Admin DTO/diagnostic lanes are
   about to run.
2. Public Client playback capability parity gate, before HEVC/AV1, hardware
   tone mapping, image subtitle burn-in, or device profile work.
3. Source Fingerprint hash trigger/reconciliation policy, before Admin manual
   trigger or automatic Source Duplicate Relationship mutation.
4. Any implementation that changes `TranscodeProfile` identity,
   `PlaybackTargetProfile` identity, HLS request variant identity, or
   `HlsArtifactManifest` reconstruction.
5. Trusted proxy/header or endpoint discovery implementation, because it
   crosses auth, CORS, client endpoint contracts, and Admin diagnostics.
6. Broad durable-job scheduler migration, because it shares `jobs.rs`,
   `job_runtime.rs`, runtime budgets, and redacted durable errors.

## Parallel-Friendly Work

These can proceed in parallel if they avoid the shared contract files:

- Remote access cookbook and config-check fixtures under docs/deploy/scripts,
  with no endpoint discovery API and no tunnel runtime.
- Addon resource flow pattern audit as docs/research.
- Source hash trigger/reconciliation policy audit as docs/research.
- Public playback capability contract research that does not edit DTOs yet.
- PostgreSQL runtime harness / verification tasks that do not add migrations
  or alter shared repository contracts.
- VFS cache non-destructive remediation planning if Admin DTO scope is owned
  or idle.

## Refactors To Defer Until A Feature Triggers Them

- Playback artifact identity cleanup: defer until a selected playback feature
  needs identity changes.
- Network policy classifier: defer until endpoint discovery, tunnel health, or
  richer proxy states are accepted.
- Disk-scan executor registry: defer until more than two disk-scan job kinds
  need the same shape.
- Mechanical Addon protocol/client/catalog splits: defer until the protocol
  Interface itself becomes the source of drift.
- Admin diagnostics module sizing review: do one domain at a time when that
  domain is changing anyway.
- A new `nako-control-plane` crate: ADR 0053 does not require it; introduce it
  only after multiple real callers prove the seam.

## Recommended Queue

1. If Admin-heavy lanes are next, run Admin contract generator deepening first.
   If Admin work is light, keep the completed route parity gate as sufficient
   and move on.
2. Run `public-client-playback-capability-contract-parity-gate` before any
   playback execution expansion.
3. Continue the open architecture audits for host-owned Addon resource flow and
   Source Fingerprint hash triggering/reconciliation.
4. Run remote access cookbook/config fixtures in parallel because they are
   low-conflict and operator-visible.
5. Pick one implementation lane after those contracts are clearer; do not open
   a global fearless refactor campaign.

## Bottom Line

Nako should not continue with a broad fearless refactor campaign before product
work. The current architecture has several deep modules that are earning their
interfaces. The right move is selective serial-first deepening of shared
contracts:

- Admin contract generator ownership,
- Public Client playback capability parity,
- source hash triggering/reconciliation policy,
- host-owned Addon resource flow pattern.

Everything else should either run as docs/research in parallel or wait until a
concrete feature touches the files.
