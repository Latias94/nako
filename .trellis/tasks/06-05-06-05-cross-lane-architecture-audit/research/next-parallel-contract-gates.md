# Next Parallel Contract Gates

Date: 2026-06-05

## Scope

This note audits the shared contract surfaces that can serialize the next
parallel development wave. It does not recommend production code changes in
this task.

Primary inputs:

- Parent audit notes under
  `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/`.
- Completed child task:
  `.trellis/tasks/06-05-admin-route-inventory-parity-gate/`.
- Completed child task:
  `.trellis/tasks/06-05-playback-output-profile-device-capability-audit/`.
- `.trellis/spec/nako-api/backend/*`,
  `.trellis/spec/nako-client-protocol/backend/index.md`,
  `.trellis/spec/nako-server/backend/*`,
  `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`,
  `.trellis/spec/nako-addon-protocol/backend/index.md`,
  `.trellis/spec/nako-core/backend/index.md`,
  `.trellis/spec/nako-playback/backend/*`,
  `.trellis/spec/nako-transcode/backend/*`.
- `CONTEXT.md`, `docs/architecture/LANES.md`,
  `docs/architecture/PLAYBACK.md`, and
  `docs/architecture/CONTROL_PLANE.md`.

## Executive Summary

The next parallel wave should not let every worker touch shared contracts
opportunistically. Nako has enough cross-lane maturity that the main risk is
contract drift, not missing foundations.

Two recent tasks change the queue shape:

1. The Admin route inventory parity gate is complete and should now be a
   standing serial gate for every Admin route, Admin DTO, generated Admin
   contract, and Admin Web route/client change.
2. The playback output/profile audit is complete, but it found a missing next
   serial gate: Public Client playback capability parity across protocol DTOs,
   OpenAPI/SDK generation, Rust clients, server query/body mapping, and HTTP
   docs.

Recommended serial gates before broad implementation:

1. Keep one Admin contract owner for any `/admin/v1/*`, Admin DTO, and generated
   Admin TypeScript work.
2. Run a Public Client playback capability parity gate before profile v2,
   HEVC/AV1 execution, hardware tone mapping, image subtitle execution, or
   renderer/device profile work.
3. Require one owner for generated artifacts and route inventories in a given
   wave. Other lanes can implement behind stable contracts or stay docs-only.
4. Require one owner for shared identity/access/config/manifest shape changes.
   These are not safe side edits for feature lanes.
5. Let parallel workers proceed only when their writable files are disjoint and
   their stop conditions do not point back into these shared surfaces.

## Shared Contract Surfaces

| Surface | Primary owner files | Collision mode | Parallel rule |
| --- | --- | --- | --- |
| Public Client API | `crates/nako-client-protocol/src/*`, `crates/nako-api/src/public_client.rs`, `crates/nako-api/src/openapi.rs`, `crates/nako-api/src/sdk.rs`, `crates/nako-client-core/src/*`, `crates/nako-client/src/lib.rs`, `docs/api/HTTP_API.md`, generated SDK outputs | Route/DTO/query field drift, SDK/docs mismatch, unsafe Admin concept leaking into Client Application contract | One Public Client contract owner per wave. Feature lanes must not add public route strings or DTO fields independently. |
| Admin API | `crates/nako-api/src/admin.rs`, `crates/nako-api/src/admin/*`, `crates/nako-server/src/http/admin.rs`, `crates/nako-server/src/http/addons.rs` | Admin route implemented but not generated/excluded, generated route not implemented, DTO redaction drift | One Admin contract owner whenever any `/admin/v1/*` route or Admin DTO changes. |
| Generated Admin contract | `crates/nako-api/src/admin_contract.rs`, `apps/admin-web/src/adminApi/generated/contract.ts`, `web/src/api/admin/generated/contract.ts` | Hand-edited or stale generated TypeScript, route key drift, duplicate placeholder syntax | Generated files are artifacts from `nako-api`; only the contract owner regenerates them. |
| Admin Web route/client state | `apps/admin-web/src/App.tsx`, `apps/admin-web/src/adminApi/client.ts`, `apps/admin-web/src/adminApi/dataSource.ts`, `apps/admin-web/src/adminApi/mockData.ts`, Admin Web route tests | URL search, mock fallback, data-source mapping, and route constant drift | Safe only after the Admin contract owner declares the route/DTO stable. |
| Shared DTOs | `nako-api` Admin/Public DTOs, `nako-client-protocol` DTOs, `nako-addon-protocol` wire records, `nako-core` domain records | Same concept represented differently across layers, or raw domain records exposed as wire responses | Assign by audience first: Public Client, Admin, Addon Protocol, or core domain. Do not let feature workers duplicate DTO shapes. |
| Shared identity/access | `crates/nako-core/src/id.rs`, `crates/nako-core/src/identity.rs`, identity repositories, access checks in server HTTP/app layers | New ID/access value without DB/API/server mapping, principal or locator leaks into Public Client DTOs | Serial gate required for new IDs, roles, principals, Library Access semantics, or source duplicate identity behavior. |
| Shared config/settings | `crates/nako-server/src/config.rs`, `crates/nako-server/src/config/preflight.rs`, `crates/nako-core/src/settings.rs`, settings repositories, Admin settings DTOs, deployment examples | Config TOML/preflight/Admin readiness drift, secret leaks, runtime settings versus startup config confusion | One config/settings owner when shape changes. Docs-only cookbook work can run separately if it does not change structs or Admin DTOs. |
| Addon manifest/protocol | `crates/nako-addon-protocol/src/lib.rs`, `crates/nako-official-addon-catalog/src/*`, `crates/nako-reference-addon/src/*`, server Addon registration/routes | Manifest shape or scope changes without protocol versioning, grants inferred from declarations, official catalog drift | One Addon Protocol/manifest owner for wire shape, scopes, runtime routes, or official manifest changes. |
| Runtime/artifact manifests | `crates/nako-transcode/src/artifact.rs`, `crates/nako-transcode/src/profile.rs`, `crates/nako-server/src/app/playback/*`, VFS staging manifest code | HLS/staging artifact identity and reconstruction drift while playback/storage lanes both edit flow files | Do not run HLS identity, VFS staging, and storage circuit-breaker changes in parallel without one playback/storage planner. |
| Durable jobs/resource policy | `crates/nako-core/src/job.rs`, job repositories, `crates/nako-server/src/app/jobs.rs`, runtime/resource policy files | Hidden background work, scheduler/resource-class drift, queue starvation/regression | Source hash triggering, broad scheduler migration, and Addon task/event runtime policy must serialize. |
| Terminology/baselines | `CONTEXT.md`, ADRs, `docs/architecture/*.md`, `.trellis/spec/*` | Product language or baseline changes that make active workers implement different contracts | Planner owns durable terminology/ADR/spec changes during a parallel wave. |

## Effect Of Recent Gates

### Admin Route Inventory Parity Gate

Status: complete on 2026-06-05 in
`.trellis/tasks/06-05-admin-route-inventory-parity-gate/task.json`.

Effective constraint:

- Every implemented `/admin/v1/*` route must now be generated or explicitly
  excluded with a reason.
- Generated Admin routes must map back to implemented server routes.
- Placeholder syntax must normalize between Axum route registration and
  generated route constants.
- Exclusions are live contract facts, not comments; stale exclusions should
  fail the gate.
- Generated and excluded Admin routes must stay out of Public Client route
  inventories.

Practical consequence:

- Admin-facing playback diagnostics, Addon Manager, network diagnostics,
  VFS/source-hash diagnostics, settings mutations, and Admin Web pages should
  not each regenerate contracts independently.
- One contract owner should take `admin_contract.rs` and generated Admin
  TypeScript for a wave; feature workers either wait, avoid the generated
  contract, or coordinate through that owner.

### Playback Output/Profile Device Capability Audit

Status: complete on 2026-06-05 in
`.trellis/tasks/06-05-playback-output-profile-device-capability-audit/task.json`.

Effective constraint:

- Do not start HEVC/AV1 output execution, hardware tone-map execution, image
  subtitle execution, or profile v2 work until current playback capability
  fields are parity-checked across Public Client surfaces.
- Public Client capability DTOs describe client/player facts only. They must
  not carry FFmpeg encoder names, GPU device paths, hardware probe facts,
  operator fallback policy, or runtime resource pressure.
- Admin diagnostics can expose redaction-safe support/effective profile
  evidence, but that is separate from Public Client request capability.
- `PlaybackTargetProfile::identity`, `TranscodeProfile` identity, HLS request
  variant identity, and HLS artifact manifests are high-conflict files.

Practical consequence:

- The next serial playback contract task is
  `public-client-playback-capability-contract-parity-gate`.
- Only after that gate should a worker add the additive
  `playback-output-profile-v2-skeleton-contract-only` fields.

## Required Serial Gates

### Gate 1: Admin Route And Generated Contract Owner

Trigger:

- Add, rename, delete, or expose any `/admin/v1/*` route.
- Add or change an Admin DTO consumed by Admin Web or `web`.
- Change `crates/nako-api/src/admin_contract.rs`.
- Regenerate Admin TypeScript contracts.

Writable owner set:

- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`
- focused route inventory tests
- route/DTO files explicitly listed in the task

Validation commands:

```powershell
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast
cargo check -p nako-api -p nako-server --tests
npm run generate:admin-api --prefix apps/admin-web
cargo fmt --all -- --check
git diff --check
```

Stop conditions:

- An implemented Admin route is neither generated nor explicitly excluded.
- A generated Admin route has no implemented server route.
- An exclusion has no reason, points at a removed route, or duplicates a
  generated route.
- Any Public Client inventory or generated Public SDK contains `/admin` or an
  Admin-only governance shape.
- Generated TypeScript contracts drift and no contract owner owns the output.
- Another active worker is already editing `admin_contract.rs` or generated
  Admin contract files.

### Gate 2: Public Client Playback Capability Parity

Trigger:

- Add, rename, delete, or reinterpret a playback capability field.
- Change browser ticket capability body, playback/remux/HLS query params,
  renderer media capabilities, generated client builders, or HTTP docs.
- Begin profile v2, HEVC/AV1, hardware tone-map, image subtitle execution, or
  renderer/device profile work.

Writable owner set:

- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-client-protocol/src/lib.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-api/src/sdk.rs`
- `crates/nako-client-core/src/*`
- `crates/nako-client/src/lib.rs`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/http/renderer.rs`
- generated SDK outputs and `docs/api/HTTP_API.md`

Validation commands:

```powershell
cargo nextest run -p nako-client-protocol public_route_inventory --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-client-core -p nako-client --no-fail-fast
cargo check -p nako-client-protocol -p nako-client-core -p nako-client --tests
cargo check -p nako-api -p nako-server --tests
git diff --check
```

If SDK or OpenAPI outputs are touched, also run the matching generator command
from `crates/nako-api/examples/*` and compare committed outputs according to
the task scope.

Stop conditions:

- The server accepts a capability query/body field that no generated client or
  docs can send.
- `nako-client-core`, generated SDKs, OpenAPI, or HTTP docs lag behind
  protocol DTOs.
- Public Client DTOs include Admin/runtime-only facts such as FFmpeg command
  facts, GPU/device paths, hardware reports, resource pressure, operator
  fallback policy, bearer tokens, raw source locators, or principal IDs.
- A field can change planner output but is missing from
  `PlaybackTargetProfile::identity` or planner tests.
- Another active worker is editing the same playback capability DTOs,
  generated SDK surfaces, or HTTP mapping.

### Gate 3: Public/Admin DTO Audience And Redaction

Trigger:

- Add any wire DTO field derived from storage, VFS, playback, Addons,
  Generated Artifacts, jobs, network config, provider data, or source identity.
- Move data between Public Client API and Admin API.

Writable owner set:

- The audience-specific DTO module plus route/test files named in the task.
- Generated artifacts only if the gate owner also owns Gate 1 or Gate 2.

Validation commands:

```powershell
cargo nextest run -p nako-api --no-fail-fast
cargo check -p nako-api -p nako-server --tests
```

For Admin Web consumers:

```powershell
npm run check --prefix apps/admin-web
npm run test --prefix apps/admin-web
```

Stop conditions:

- The task cannot answer whether the field is Public Client, Admin, Addon
  Protocol, or core domain.
- The DTO exposes local path, Source Locator, Source Fingerprint, raw hash,
  etag, provider payload, prompt, token, credential, FFmpeg argv/stderr, device
  path, raw URL, or raw backend error text.
- A domain/database record is returned directly through HTTP.
- A generated contract changes but no generated-contract owner is active.

### Gate 4: Shared Identity And Access

Trigger:

- Add or rename strong IDs in `nako-core`.
- Change User, Role, Library Access, session auth, principal, invitation,
  source duplicate identity, or source identity semantics.
- Expose a new identity in Public Client or Admin contracts.

Writable owner set:

- `crates/nako-core/src/id.rs`
- `crates/nako-core/src/identity.rs`
- identity/access repository traits and DB adapters
- HTTP/app access checks explicitly in scope
- audience-specific DTO mapping only through Gate 1 or Gate 2 when needed

Validation commands:

```powershell
cargo nextest run -p nako-core --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests
git diff --check
```

Stop conditions:

- A new ID requires schema or repository changes that are not owned by the
  task.
- Public Client DTOs expose server principal IDs, raw source locators, or
  source identity internals.
- Source Fingerprint evidence is treated as Media Source identity rather than
  duplicate evidence.
- Role/Library Access changes are made without server access tests and DTO
  redaction review.

### Gate 5: Shared Config And Runtime Settings

Trigger:

- Change TOML config shape, preflight behavior, network exposure settings,
  tunnel provider declarations, auth config, transcode/playback budgets, or
  runtime Admin settings documents.
- Change Admin settings DTOs or Admin Web settings pages.

Writable owner set:

- `crates/nako-server/src/config.rs`
- `crates/nako-server/src/config/preflight.rs`
- `crates/nako-core/src/settings.rs`
- settings repositories and Admin settings DTOs
- deployment examples only if the task owns docs/config consistency

Validation commands:

```powershell
cargo nextest run -p nako-server config_preflight --no-fail-fast
cargo check -p nako-server --tests
```

If Admin settings DTOs or generated contracts change:

```powershell
cargo nextest run -p nako-api admin_contract --no-fail-fast
npm run generate:admin-api --prefix apps/admin-web
npm run check --prefix apps/admin-web
npm run test --prefix apps/admin-web
```

Stop conditions:

- Config shape changes and Admin settings mutations are mixed without one
  owner.
- A preflight/Admin diagnostic can echo database URLs, tunnel URLs, proxy
  origins, env secret values, local paths, raw headers, or credentials.
- Remote access endpoint discovery is attempted as a docs/config side effect.
- Trusted proxy/header behavior changes while auth or trace middleware is owned
  by another worker.

### Gate 6: Addon Manifest And Protocol

Trigger:

- Change Addon manifest, protocol version, runtime route constants, resource,
  task, event, hosted page, permission/scope, side effect, install descriptor,
  or health wire shape.
- Change official Addon catalog manifest facts.
- Add Addon Manager behavior that consumes or mutates manifest/grant/token
  contracts.

Writable owner set:

- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/*`
- `crates/nako-official-addon-catalog/src/*`
- `crates/nako-reference-addon/src/*`
- server Addon DTO/routes only if coordinated with Gate 1

Validation commands:

```powershell
cargo nextest run -p nako-addon-protocol --no-fail-fast
cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api --tests
```

For official/reference manifest work:

```powershell
cargo nextest run -p nako-official-addon-catalog -p nako-reference-addon --no-fail-fast
```

Stop conditions:

- A breaking manifest or protocol change is made without a new Addon Protocol
  Version decision.
- Manifest declarations are treated as accepted grants.
- Addon Package or Addon Suite becomes the permission unit instead of Addon.
- Hosted pages receive Nako admin credentials.
- Server Addon route/generated Admin contract changes are needed but no Admin
  contract owner is active.

### Gate 7: Runtime, Artifact Manifest, And Durable Job Policy

Trigger:

- Change HLS artifact manifest identity, transcode profile identity, staging
  manifest lease behavior, durable job kinds/resource classes, scheduler
  policy, or hidden background work.

Writable owner set:

- Playback/transcode artifact and identity files for playback work.
- VFS/staging files for storage work.
- Job/runtime/resource files for control-plane work.
- Do not mix these owner sets unless the task is explicitly a cross-lane gate.

Validation commands:

```powershell
cargo check -p nako-playback -p nako-transcode -p nako-server --tests
cargo nextest run -p nako-server <focused_playback_or_job_filter> --no-fail-fast
git diff --check
```

Use the package-specific nextest filters from the owning Trellis spec for the
exact lane.

Stop conditions:

- Raw `tokio::spawn` is added for scan, metadata, playback, addon, webhook, or
  artifact workflows that need durable state, retry, cancellation, diagnostics,
  or resource policy.
- HLS request identity or artifact reconstruction changes while another worker
  edits playback profile/output capability.
- Source hash triggering runs in parallel with broad durable-job scheduler
  migration.
- VFS staging or circuit-breaker changes land in the same playback flow files
  as HLS lifecycle/admission changes.

## Parallel Worker Write Boundaries

Safe or mostly safe parallel workers:

| Worker | Allowed writable files | Must avoid |
| --- | --- | --- |
| Remote access cookbook/docs | `docs/deployment/**`, `deploy/**`, maybe release checklist docs | `crates/nako-server/src/config.rs`, Admin network DTOs, Public Client endpoint discovery, generated contracts |
| Remote access config-fixture release gate | docs/examples plus `scripts/release-gate.*` if explicitly assigned | Config shape, Admin DTOs, tunnel runtime, client endpoint contracts |
| Addon host-owned resource flow audit | task research docs, maybe future architecture/spec docs only | `nako-addon-protocol`, `nako-api`, generated Admin contracts, server Addon routes until owner exists |
| Addon task/event execution policy audit | task research docs and ADR/spec draft only | durable job scheduler code, Addon manifest wire shape, generated Admin contracts |
| Source hash triggering/reconciliation policy audit | task research docs and architecture map notes only | `jobs.rs`, source identity repositories, Admin DTOs, source duplicate schema/repository mutation |
| Playback architecture map reconciliation | `docs/architecture/PLAYBACK.md` and maybe `docs/architecture/LANES.md` if assigned | playback DTOs, generated SDKs, `nako-playback`/`nako-transcode` identity files |
| Storage/source identity PostgreSQL harness | focused DB adapter tests and harness docs if no migrations | identity semantics, source duplicate policy, Admin DTOs, source hash triggering |
| Admin Web route search helper | `apps/admin-web/src/App.tsx`, route helper/test files only after Admin contract scope is free | generated contract output, Admin DTOs, Public Client SDK changes |

Serial or unique-owner workers:

| Worker | Unique owner files |
| --- | --- |
| Admin contract owner | `crates/nako-api/src/admin_contract.rs`, generated Admin TypeScript contracts, Admin route inventory tests, route suffix/exclusion table |
| Public playback capability parity owner | `nako-client-protocol` playback DTOs, `nako-api` OpenAPI/SDK/public mapping, `nako-client-core`, `nako-client`, server playback/renderer mapping, `docs/api/HTTP_API.md`, generated SDK outputs |
| Shared identity/access owner | `nako-core/src/id.rs`, `nako-core/src/identity.rs`, identity repositories, access-check mapping, related DTO mapping |
| Shared config/settings owner | `nako-server/src/config.rs`, `config/preflight.rs`, `nako-core/src/settings.rs`, Admin settings DTOs/contracts, settings pages |
| Addon protocol/manifest owner | `nako-addon-protocol`, official/reference addon manifests, Addon client protocol mapping, protocol docs |
| Runtime/durable job owner | job kind/resource policy, scheduler/runtime files, broad resource budget policy |

## Recommended Next Queue

1. Serial first or standing owner:
   `public-client-playback-capability-contract-parity-gate`.
   This is now the highest missing contract gate after Admin route parity.
2. Parallel docs/ops:
   remote access cookbook and config-check fixtures, provided they do not
   change config structs or Admin DTOs.
3. Parallel architecture audits:
   host-owned Addon resource flow, Addon task/event policy, and source hash
   triggering/reconciliation policy.
4. Optional low-conflict doc reconciliation:
   playback seek/status architecture-map reconciliation.
5. After Gate 2:
   `playback-output-profile-v2-skeleton-contract-only`, with one playback
   contract owner and no HEVC/AV1 or hardware tone-map execution yet.

## Global Stop Conditions For The Wave

Stop and return to planner coordination when:

- A worker needs to edit a file listed under a serial or unique-owner worker but
  was not assigned that owner role.
- Two active workers need the same generated artifact, route inventory, schema,
  identity, config, protocol manifest, or runtime identity file.
- A task requires a schema migration, Addon Protocol Version change, Public
  Client versioning decision, or ADR update not listed in its PRD.
- A redaction question cannot be answered from existing specs.
- A feature attempts to make an Admin concept public, or to make a Public
  Client fact carry operator/runtime policy.
- A docs-only task discovers it must change production Rust/TypeScript to stay
  truthful.
- Any required focused gate fails and the fix would cross the assigned writable
  boundary.

## Bottom Line

Parallel work is safe only if contract ownership is explicit. The completed
Admin route parity gate reduces Admin drift, but it does not remove the need
for a single Admin contract owner. The completed playback capability audit
identifies the next serial gate: Public Client playback capability parity. Once
those owners are assigned, docs/ops and architecture-audit workers can proceed
in parallel with disjoint writable files.
