# Addon / Automation / Events / Metadata-Library Boundary Review

Read set:

- `.trellis/tasks/06-04-architecture-boundary-refactor-review/prd.md`
- `CONTEXT.md`
- `AGENTS.md`
- `docs/adr/0003-http-addons-before-in-process-plugins.md`
- `docs/adr/0004-ai-as-external-automation-first.md`
- `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `.trellis/spec/nako-addon-protocol/backend/index.md`
- `.trellis/spec/nako-addon-client/backend/index.md`
- `.trellis/spec/nako-automation/backend/index.md`
- `.trellis/spec/nako-events/backend/index.md`
- `.trellis/spec/nako-metadata/backend/index.md`
- `.trellis/spec/nako-library/backend/index.md`

Scope reviewed:

- `crates/nako-addon-protocol`
- `crates/nako-addon-client`
- `crates/nako-automation`
- `crates/nako-events`
- `crates/nako-metadata`
- `crates/nako-library`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/app/addons/*`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/app/metadata_application.rs`
- `crates/nako-server/src/app/metadata_scan.rs`
- `crates/nako-server/src/app/webhooks.rs`
- related Admin and Addon runtime routes in `crates/nako-server/src/http/*` and `crates/nako-api/src/*`

## Top Opportunities

### 1. Move Generated Artifact metadata application into a deep metadata/automation Module

Files:

- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/app/metadata_application.rs`
- `crates/nako-metadata/src/candidate_review.rs`
- `crates/nako-metadata/src/confirmation.rs`
- `crates/nako-core/src/automation.rs`
- `crates/nako-api/src/admin/automation.rs`
- `crates/nako-server/src/app/tests/automation.rs`

Problem:

`AutomationAppService` currently owns several unrelated Interfaces behind one server-layer Module: Automation Provider CRUD, provider job enqueueing, Generated Artifact review, metadata apply planning, provider mapping proposal parsing, bulk apply batch creation, bulk execution, recovery listing, and metadata merge/application. The server file is doing both Admin Adapter work and core Acceptance Workflow work.

The deepest reusable behavior is the Acceptance Workflow for Generated Artifact metadata suggestions: plan before apply, stale target detection, metadata field lock handling, Provider Mapping planning, idempotent outcome persistence, and catalog projection. That behavior is not specific to HTTP Admin routes, and it is not only automation execution. Addon `metadata_write` already uses `MetadataApplication`, but it has a separate payload parser and commit route. This creates a shallow Seam: callers still need to know too much about artifact status, payload shape, metadata merge, provider mapping, catalog projection, and idempotency details.

Proposed refactor:

Create a deep Module for metadata authority application. A conservative first step is to move `MetadataApplication` from `nako-server` into `nako-metadata` behind repository traits. Then move Generated Artifact metadata planning/parsing/apply logic into a `nako-metadata` service such as `GeneratedArtifactMetadataApplicationService<R>`, while keeping Automation Provider job execution in `nako-automation`.

The server `AutomationAppService` would become an Admin Adapter and orchestration shell:

- provider configuration and job enqueueing stay in `AutomationAppService` or `nako-automation`;
- Generated Artifact review/apply calls delegate to the deep metadata application Module;
- DTO mapping remains in `nako-api`;
- persistence stays behind `nako-core` repository traits.

Deletion/deepening angle:

- Delete server-local duplicate metadata patch parsing, provider subject proposal parsing, field summary construction, and apply plan assembly from `automation.rs`.
- Deepen the metadata application Interface so callers pass an artifact or side-effect command and receive a plan/result, instead of assembling merge, locks, provider mappings, and catalog commits themselves.
- Increase Locality: metadata authority rules live with Candidate Review and Hierarchy Confirmation rather than in a generic server app file.
- Increase Leverage: Addon, Automation Provider, and future Generated Artifact sources can share one plan/apply path.

Test impact:

- Move most Generated Artifact apply plan and stale/idempotency tests from `crates/nako-server/src/app/tests/automation.rs` into `nako-metadata` service tests using `nako-db` as a dev dependency.
- Keep server route tests thin: route -> service -> DTO mapping.
- Add explicit cross-source tests showing Addon Side Effect and Generated Artifact application use the same metadata lock semantics.

Risk/ADR conflicts:

- No ADR conflict if the Module preserves ADR 0004 and ADR 0015: generated output still becomes canonical only through an Acceptance Workflow.
- Watch dependency depth. `nako-metadata` already depends on `nako-catalog` and `nako-core`, so moving `MetadataApplication` there is plausible. Moving too much Automation Provider job execution into `nako-metadata` would be the wrong direction.
- Public API shape should not change in this refactor.

Suggested workflow scale:

- Medium refactor task first: move `MetadataApplication` and Generated Artifact metadata apply planning into `nako-metadata`.
- Follow-up task: route Addon `metadata_write` through the same Interface if the first task exposes a clean command shape.

### 2. Extract a host-owned Library File Write Module out of Addon internals

Files:

- `crates/nako-server/src/app/addons/library_file_write.rs`
- `crates/nako-server/src/app/addons/subtitles.rs`
- `crates/nako-server/src/app/addons/side_effect_apply.rs`
- `crates/nako-server/src/app/subtitle_sidecar.rs`
- `crates/nako-server/src/app/nfo.rs`
- `crates/nako-nfo`
- `crates/nako-vfs`
- `docs/adr/0051-host-owned-subtitle-import-chain.md`

Problem:

The current `LibraryFileWriteRuntime` is host-owned behavior placed under `app/addons/`. It is already used for two different flows:

- Addon Side Effect `library_file_write` for NFO export;
- Admin Addon subtitle import apply for subtitle sidecar writes.

That means the Implementation is deeper than the folder name implies. The Interface currently looks addon-specific, but the behavior is a Nako-owned Library File Write policy: target resolution, storage backend selection, write permits, NFO export writability, subtitle sidecar path derivation, atomic/backup write policy, and report generation.

This naming and placement weakens Locality. Future Library File Write callers will either import an Addon-private Module or duplicate storage/NFO/subtitle write rules elsewhere.

Proposed refactor:

Extract a server-level host-owned Module, for example `crates/nako-server/src/app/library_file_write.rs`, with a public crate-local Interface centered on Nako domain commands:

- `LibraryFileWriteService`;
- `NfoSidecarWriteCommand` or `NfoExportWriteCommand`;
- `SubtitleSidecarWriteCommand`;
- `LibraryFileWriteReport`.

Keep a small Addon Adapter under `app/addons/` that translates `AddonLibraryFileWritePayload` and `AddonSideEffectRecord` into the host-owned command. Keep subtitle import as another Adapter to the same Module.

Deletion/deepening angle:

- Delete the misleading Addon ownership from general Library File Write runtime code.
- Delete or shrink ad hoc subtitle import write helpers in `subtitles.rs`.
- Deepen the Interface around "Nako owns Library File Write" instead of "Addon asks for this file role".
- Improve Leverage for NFO Export, subtitle import, artwork sidecar, and future Addon Side Effects without giving Addon Sidecars direct storage authority.

Test impact:

- Move file-write behavior tests to a dedicated server app test module focused on `LibraryFileWriteService`.
- Keep Addon Side Effect tests focused on authorization, idempotency, and Adapter mapping.
- Keep subtitle import tests focused on candidate selection and plan/apply idempotency.

Risk/ADR conflicts:

- This aligns with ADR 0020 and ADR 0051: Addon Sidecars do not receive raw filesystem authority; Nako owns Library File Write.
- The main risk is accidentally widening the Interface so Addon code can choose raw paths. The command must keep target IDs, roles, policies, and backend resolution host-owned.

Suggested workflow scale:

- Small-to-medium refactor task. No schema or public API change needed.

### 3. Deepen Addon event delivery around the durable outbox Seam

Files:

- `crates/nako-events/src/lib.rs`
- `crates/nako-server/src/app/webhooks.rs`
- `crates/nako-server/src/app/addons/event_runtime.rs`
- `crates/nako-core/src/addon_event.rs`
- `crates/nako-core/src/repository/addon_event.rs`
- `crates/nako-core/src/repository/jobs.rs`
- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-server/src/http/addons.rs`

Problem:

`nako-events` has a deep `WebhookDeliveryService` for webhook envelope creation, signing, attempt persistence, status mapping, and retry delay. Addon Event Subscription delivery uses the same durable event outbox but implements most of its scheduler, work status, claim, lease, retry, filter, attempt status, and dispatch summary behavior inside `app/addons/event_runtime.rs`.

The two flows should not be collapsed into one generic "HTTP delivery" Interface because Addon Event Subscription needs Addon Manifest validation, Addon Token/auth mode, accepted grants, routing plans, declaration filters, and Addon Protocol envelopes. But the current split leaves outbox delivery mechanics duplicated at the server layer and makes the durable outbox Seam less explicit than ADR 0014 intends.

Proposed refactor:

Create an outbox-delivery runtime Module that is generic over a target Adapter:

- target discovery Adapter: webhooks or Addon Event Subscriptions;
- attempt claim/persist Adapter: webhook attempts or addon event attempts;
- envelope/call Adapter: signed webhook HTTP or Addon Protocol event call;
- retry policy/status Adapter: target-specific max attempts and safe errors.

This could live in `nako-events` if the Interface can stay dependency-light, or in `nako-server/src/app/events.rs` if Addon registration dependencies make a pure crate awkward. The key is to make Addon Event Subscription delivery a target Adapter, not a parallel runtime hidden in `app/addons/event_runtime.rs`.

Deletion/deepening angle:

- Delete duplicated JoinSet dispatch summary and retry-status mechanics from webhook and addon flows.
- Deepen the outbox Interface: event delivery callers ask "dispatch this outbox event to this target family" and do not manually assemble concurrency, attempt sorting, and retry status.
- Preserve distinct Adapters for webhook and Addon Protocol calls.

Test impact:

- Add adapter-level tests for Addon Event Subscription filtering, missing grants, routing-plan deferral, and retryable/non-retryable Addon client failures.
- Add shared runtime tests for attempt ordering, exhausted attempts, in-flight lease behavior, and idempotent replay.
- Keep HTTP route tests as route inventory and DTO checks.

Risk/ADR conflicts:

- No conflict with ADR 0014; this refactor strengthens the durable event boundary.
- No conflict with ADR 0020 if Addon auth/grants remain in the Addon Adapter.
- Risk: over-generalizing away security distinctions. The shared Module should own mechanics, not authority.

Suggested workflow scale:

- Medium-to-large refactor task. Start with read-only extraction of status/retry calculation and target Adapter shape before moving dispatch execution.

### 4. Add an Addon outbound call engine inside `nako-addon-client`

Files:

- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-server/src/app/addons/diagnostics.rs`
- `crates/nako-server/src/app/addons/resource_search.rs`
- `crates/nako-server/src/app/addons/subtitles.rs`
- `crates/nako-server/src/app/addons/task_runtime.rs`
- `crates/nako-server/src/app/addons/event_runtime.rs`
- `crates/nako-server/src/app/addons/external_acquisition.rs`

Problem:

`nako-addon-client` is already the right crate for outbound Addon Sidecar calls, but its `lib.rs` is a broad Module with many repeated Implementations: manifest validation, grant checks, timeout/default resolution, auth header creation, protocol headers, `x-nako-attempt`, retryable HTTP status handling, response envelope parsing, typed schema checks, and redaction rules.

The Interface surface is large and growing: generic resource call, resource search, link check, subtitle, task, event, health, runtime side-effect submission, metadata write, artwork write, and external acquisition materialization. New Addon Resources currently invite copy/paste of call setup rather than attaching to a deep call engine.

Proposed refactor:

Keep public functions stable but introduce internal Modules:

- `transport`: `AddonTransport`, `ReqwestAddonTransport`, HTTP request/response;
- `call`: `AddonOutboundCall`, auth/header builder, retry loop, response parser;
- `resource`: generic resource call plus typed resource adapters;
- `task`: Addon Task call adapter;
- `event`: Addon Event Subscription call adapter;
- `runtime`: Nako runtime client for side effects, generated artifacts, materialization;
- `error`: redaction-safe `AddonClientError` and retry classification.

The important deepening is not just file splitting. It is making one internal call engine own the invariant: no token material in body/errors, correct protocol headers, consistent attempt behavior, and typed schema validation hooks.

Deletion/deepening angle:

- Delete duplicated auth/header/timeout/retry code across resource, task, event, and runtime paths.
- Reduce the Interface new resource authors must understand: they provide declaration lookup, payload schema, response schema, and response parser.
- Increase Locality for redaction and retry policy.

Test impact:

- Keep existing public tests but move most header/retry/redaction assertions to call-engine tests.
- Add a single "new typed resource adapter" fixture test to prove schema hooks are sufficient.
- Run `cargo nextest run -p nako-addon-client --no-fail-fast`.

Risk/ADR conflicts:

- No ADR conflict. This preserves ADR 0003/0015/0020 by strengthening the HTTP Addon Protocol Adapter.
- Risk is accidental wire drift. Keep public functions and serialized envelopes unchanged; use golden request-body tests around resource_search, subtitle, task, event, and runtime side effects.

Suggested workflow scale:

- Medium refactor task. Good first fearless-refactor candidate because it is mostly internal and heavily testable.

### 5. Make scan-time metadata acquisition an explicit scan adjunct Interface

Files:

- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-server/src/app/metadata_scan.rs`
- `crates/nako-server/src/app/addons/scan_metadata.rs`
- `crates/nako-library/src/scan.rs`
- `crates/nako-library/src/index.rs`
- `crates/nako-library/src/probe.rs`
- `crates/nako-library/src/local_inference/*`
- `crates/nako-nfo`

Problem:

`nako-library` correctly owns VFS scan, ingestion, probe orchestration, and Local Inference without depending on Addon or metadata provider crates. The server then appends scan-time metadata acquisition through `MetadataScanAcquisitionService`, which directly coordinates NFO Import and Addon Bulk Metadata Scrape task creation.

That is a reasonable dependency direction, but the current Interface is still a shallow orchestration helper: the library scan job needs to know that scan metadata consists of NFO Import plus Addon scrape; `AddonAppService` exposes scan-specific request/summary types; Addon continuation logic lives in task runtime; and tests for scan-time Addon writeback sit at startup/system level.

Proposed refactor:

Keep `nako-library` free of Addon dependencies, but introduce an explicit scan adjunct Interface in server composition:

- `LibraryScanAdjunct` or `ScanMetadataAcquisitionPort`;
- Adapter 1: NFO Import adjunct;
- Adapter 2: Addon Bulk Metadata Scrape adjunct;
- each adjunct receives `LibraryScanContext` with job ID, library, cancellation, and scan/probe summaries as needed;
- the job runner executes registered adjuncts after index/probe and aggregates typed summaries.

This makes scan-time metadata acquisition a composable Interface instead of hard-coded branching in `metadata_scan.rs`.

Deletion/deepening angle:

- Delete scan-specific public exports from `AddonAppService` where possible; expose an Addon Bulk Metadata Scrape Adapter instead.
- Deepen the library scan job Interface: it runs scan, probe, and adjuncts without knowing each adjunct's internal workflow.
- Improve Locality for future adjuncts such as Managed Artwork discovery, Generated Artifact proposal creation, or Storage Backend Health diagnostics.

Test impact:

- Add unit/service tests for the scan adjunct runner with fake adjuncts and cancellation.
- Keep end-to-end startup tests for the real Addon Bulk Metadata Scrape path.
- Add focused tests that Addon writeback continuation remains an Addon Task concern, not a library scan concern.

Risk/ADR conflicts:

- No ADR conflict if Addon calls still go through Addon Task jobs and resource budgets.
- Risk: over-abstracting too early. This should only be done if another scan adjunct is imminent or if Addon/NFO scan metadata keeps growing.

Suggested workflow scale:

- Small design spike or medium refactor task, depending on whether the first slice only extracts an Interface around current NFO/Addons or also moves tests.

## Priority Ranking

1. Add an Addon outbound call engine inside `nako-addon-client`.
   Best first fearless-refactor target: internal, test-heavy, high deletion potential, low public API risk.

2. Extract a host-owned Library File Write Module out of Addon internals.
   Strong domain correction and aligns with ADR 0020/0051; small enough to complete safely.

3. Move Generated Artifact metadata application into a deep metadata/automation Module.
   Highest long-term Leverage for Acceptance Workflow and canonical metadata governance, but larger blast radius.

4. Deepen Addon event delivery around the durable outbox Seam.
   Important control-plane cleanup, but needs careful Adapter design to avoid flattening Addon-specific authority.

5. Make scan-time metadata acquisition an explicit scan adjunct Interface.
   Useful when scan metadata grows; defer if only NFO plus Addon scrape remains stable.
