# Addon, Automation, and Control Plane Campaign Findings

## Scope

This read-only pass inspected Nako's Addon / Automation / Control Plane
architecture for a hypothetical 10-hour media-server improvement campaign. It
does not recommend reopening the Extism decision. The baseline remains HTTP
Addon Sidecars, scoped Addon Tokens, Library-Scoped Addon Grants, host-owned
Addon Side Effects, Generated Artifacts, durable jobs, and Runtime Supervisor
boundaries.

Primary evidence:

- `CONTEXT.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/adr/0003-http-addons-before-in-process-plugins.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- `docs/adr/0053-application-control-plane-boundary.md`
- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-automation/src/lib.rs`
- `crates/nako-official-addon-catalog/src/lib.rs`
- `crates/nako-server/src/app.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/app/addons/*.rs`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/app/job_runtime.rs`
- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-server/src/app/runtime.rs`
- `crates/nako-server/src/app/webhooks.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/network.rs`

## Baseline Reading

Nako already has strong extension foundations:

- ADR 0003 and ADR 0020 reject native/in-process plugins for now and keep
  Addons as HTTP Sidecars.
- ADR 0015 requires explicit capability scopes, safe error mapping, bounded
  workers, no plaintext secrets, and Generated Artifacts or accepted writes
  instead of implicit canonical mutation.
- ADR 0033 separates Addon Version, Addon Protocol Version, and Rust crate
  package version.
- ADR 0034 separates fine-grained Addon permission units from coarse Addon
  Package / Addon Suite deployment units.
- ADR 0053 makes the Application Control Plane the shared owner of addon
  mediation, runtime supervision, durable work, resource accounting,
  diagnostics, remote access, and API scale contracts.

The main implementation friction is no longer "missing Addon Protocol"; it is
that several useful control-plane capabilities are present as narrow local
modules but not yet shaped as reusable product-depth modules. The best 10-hour
campaign should therefore prefer deepening existing runtime, diagnostics,
install, and delivery modules over adding new schema or public API shapes.

## Ranked Opportunities

### 1. Deepen Addon Task Dispatch Runtime

User-visible value:

- More reliable Bulk Metadata Scrape, scan-triggered Addon Tasks, and direct
  Addon task execution.
- Clearer operator state for queued, claimed, failed, cancelled, continued, and
  retried Addon Tasks.

Problem:

- Addon Task runs already persist a `JobKind::AddonTask` row and an
  Addon-specific task-run row, but direct dispatch still has a specialized
  runtime path inside `task_runtime.rs`.
- The current interface makes callers know too much about dispatch mode,
  task declaration lookup, direct claim, outbound secret resolution, lease
  guard refresh, safe failure codes, and continuation enqueueing.

Evidence:

- `crates/nako-server/src/app/addons/task_runtime.rs:38` creates Addon Task
  runs.
- `crates/nako-server/src/app/addons/task_runtime.rs:93` persists
  `JobKind::AddonTask`.
- `crates/nako-server/src/app/addons/task_runtime.rs:118` branches direct
  dispatch from task creation.
- `crates/nako-server/src/app/addons/task_runtime.rs:462` starts direct
  dispatch with `spawn_direct_addon_task_dispatch`.
- `crates/nako-server/src/app/addons/task_runtime.rs:628` performs the outbound
  task call and has to translate failures into safe Addon Task outcomes.
- `crates/nako-server/src/http/addons.rs:195` exposes runtime task-run claim,
  progress, complete, fail, and cancel routes.
- `crates/nako-server/src/app/tests/startup.rs:824`,
  `crates/nako-server/src/app/tests/startup.rs:924`,
  `crates/nako-server/src/app/tests/startup.rs:1005`, and
  `crates/nako-server/src/app/tests/startup.rs:1082` cover scan-triggered
  Addon task execution and continuation.

Recommendation:

- Extract an `AddonTaskDispatchRuntime` module whose interface accepts a
  persisted Addon Task run and returns a normalized terminal command:
  `complete`, `fail`, `cancel`, or `enqueue_continuation`.
- Keep the current DB/schema/API shape. This should be a refactor-first task.
- Preserve `SidecarClaim` behavior; do not force every Addon Task through host
  direct dispatch.

Risk:

- Medium. It touches Addon Task lease and terminal-state behavior.

Parallelizability:

- High if isolated to Addon Task runtime and tests.
- Serial dependency before broad scan/addon product work because scan-triggered
  Addon Scrape depends on this path.

Likely gates:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server scan_library_enqueues_addon_bulk_metadata_scrape_when_enabled --no-fail-fast`
- `cargo nextest run -p nako-server scan_library_continues_addon_bulk_metadata_scrape_from_next_cursor --no-fail-fast`
- `cargo nextest run -p nako-server scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect --no-fail-fast`
- `cargo nextest run -p nako-server addon_task --no-fail-fast`

### 2. Create One Outbound Addon Call Engine

User-visible value:

- More consistent Addon health checks, Resource Search, Subtitle Search,
  Event Subscription delivery, and Task calls.
- Fewer cases where one Addon path retries, redacts, or classifies errors
  differently from another.

Problem:

- `nako-addon-client` already centralizes HTTP transport but still exposes many
  hand-shaped call helpers with repeated envelope, scope, auth, retry, and
  response-validation behavior.
- Server-side diagnostics, resource search, event runtime, and task runtime all
  map client failures into safe status/error-code vocabularies separately.

Evidence:

- `crates/nako-addon-client/src/lib.rs:224` resource calls.
- `crates/nako-addon-client/src/lib.rs:422` resource search calls.
- `crates/nako-addon-client/src/lib.rs:509` resource link-check calls.
- `crates/nako-addon-client/src/lib.rs:599` subtitle search calls.
- `crates/nako-addon-client/src/lib.rs:662` task calls.
- `crates/nako-addon-client/src/lib.rs:798` event calls.
- `crates/nako-addon-client/src/lib.rs:935` health checks.
- `crates/nako-server/src/app/addons/diagnostics.rs:68` and
  `crates/nako-server/src/app/addons/diagnostics.rs:101` classify diagnostic
  status and safe error codes.
- `crates/nako-server/src/app/addons/event_runtime.rs:805` and
  `crates/nako-server/src/app/addons/event_runtime.rs:827` classify retry and
  safe error codes for events.
- `crates/nako-server/src/app/addons/task_runtime.rs:914` and
  `crates/nako-server/src/app/addons/task_runtime.rs:929` map task-call
  failures.

Recommendation:

- Add a narrow internal call engine in `nako-addon-client` or
  `nako-server/src/app/addons` that owns request envelope construction,
  auth-header selection, retry loop, response validation, safe call facts, and
  redacted error classification.
- Keep public helper functions as adapters initially to avoid broad caller
  churn.

Risk:

- Medium. The interface must not weaken protocol validation or leak raw payloads.

Parallelizability:

- High. Can run in parallel with install-guide/catalog polish.
- Should precede deeper Addon Event and Addon Task changes if those changes
  need shared outbound error semantics.

Likely gates:

- `cargo check -p nako-addon-client --tests`
- `cargo nextest run -p nako-addon-client --no-fail-fast`
- `cargo nextest run -p nako-server addon_resource --no-fail-fast`
- `cargo nextest run -p nako-server addon_runtime_readiness --no-fail-fast`
- `cargo nextest run -p nako-server addon_event --no-fail-fast`

### 3. Unify Webhook Delivery With Addon Event Scheduler Semantics

User-visible value:

- More reliable notifications and external integrations.
- Operators get similar delivery visibility for Webhook Endpoints and Addon
  Event Subscriptions.

Problem:

- Addon Event delivery has a scheduler loop, work-state classification, leases,
  retries, and replay behavior.
- Webhook delivery has durable attempts and retry timestamps in `nako-events`,
  but the app path is still a manual per-event dispatch over enabled endpoints.

Evidence:

- `crates/nako-server/src/app/addons/event_runtime.rs:37` starts the Addon
  Event scheduler under `RuntimeSupervisor`.
- `crates/nako-server/src/app/addons/event_runtime.rs:55` runs a scheduler tick
  over pending outbox events.
- `crates/nako-server/src/app/addons/event_runtime.rs:299` exposes scheduler
  work diagnostics.
- `crates/nako-server/src/app/addons/event_runtime.rs:400` claims and delivers
  one Addon Event Subscription attempt.
- `crates/nako-server/src/app/webhooks.rs:170` delivers Webhooks for one event
  on demand.
- `crates/nako-server/src/app/webhooks.rs:206` uses a local `JoinSet` worker
  fan-out.
- `crates/nako-events/src/lib.rs:102` owns `WebhookDeliveryService::deliver_once`.
- `crates/nako-events/src/lib.rs:164` calculates retry time after failure.
- `crates/nako-server/src/http/webhooks.rs:30` exposes manual event delivery.

Recommendation:

- Start with a read-model and scheduler parity task: add a `WebhookScheduler`
  that uses existing `WebhookDeliveryAttemptRecord` and `next_retry_at` facts.
- Avoid schema changes in the first 10-hour goal unless the existing attempt
  records cannot express "due work" cleanly.
- Do not merge Webhook Endpoint and Addon Event Subscription protocols; only
  share control-plane scheduling and diagnostics concepts.

Risk:

- Medium-high if it changes retry timing. Keep the first implementation as an
  explicit scheduler tick plus tests.

Parallelizability:

- Medium. It can run in parallel with Addon install/catalog work, but should
  not run in parallel with broad Event Outbox schema work.

Likely gates:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-events --no-fail-fast`
- `cargo nextest run -p nako-server webhook --no-fail-fast`
- `cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast`

### 4. Productize Official Addon Install Guide and Source Catalog

User-visible value:

- A self-hosted user can install useful official Addons from Admin surfaces
  without reading internal docs or manually constructing manifests.
- This is a high-value operator-experience slice that does not require Nako to
  manage Docker or process lifecycles.

Problem:

- The official source catalog and install-guide machinery exists, but the next
  useful product step is to make it a polished, tested operator workflow:
  resolve catalog entry, preview lifecycle boundary, render Docker/systemd
  snippets, verify health, verify surfaces, then register/grant.

Evidence:

- `crates/nako-server/src/app/addons/catalog.rs:22` previews install guides.
- `crates/nako-server/src/app/addons/catalog.rs:37` lists catalog sources.
- `crates/nako-server/src/app/addons/catalog.rs:58` lists catalog entries.
- `crates/nako-server/src/app/addons/catalog.rs:87` renders an install guide
  for a registered Addon.
- `crates/nako-server/src/app/addons/catalog.rs:183` reports that Nako does
  not manage package lifecycle.
- `crates/nako-server/src/app/addons/catalog.rs:196` lists built-in official
  descriptors.
- `crates/nako-official-addon-catalog/src/lib.rs:17`,
  `crates/nako-official-addon-catalog/src/lib.rs:261`,
  `crates/nako-official-addon-catalog/src/lib.rs:541`,
  `crates/nako-official-addon-catalog/src/lib.rs:768`,
  `crates/nako-official-addon-catalog/src/lib.rs:970`, and
  `crates/nako-official-addon-catalog/src/lib.rs:1139` define official Addon
  IDs for metadata scraper, resource search, external acquisition runner,
  subtitle provider, DLNA renderer, and Chromecast renderer.
- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md:90`
  defers package signing and source catalog policy to later surfaces.

Recommendation:

- In a 10-hour goal, pick install-guide/source-catalog polish before process
  lifecycle automation.
- Add/verify route-level tests that the Admin response never implies Nako owns
  container lifecycle, logs, package update, or rollback.
- Add one generated "official suite" install-guide preview only if it does not
  collapse per-Addon manifest/grant units.

Risk:

- Low. Mostly Admin read-model and docs/DTO polish.

Parallelizability:

- Very high. This is a good parallel worker lane while runtime workers focus on
  task/event reliability.

Likely gates:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-official-addon-catalog --no-fail-fast`
- `cargo nextest run -p nako-addon-protocol addon_install --no-fail-fast`
- `cargo nextest run -p nako-server addon_install --no-fail-fast`
- `cargo nextest run -p nako-server addon_source_catalog --no-fail-fast`

### 5. Addon Runtime Readiness and Surface Diagnostics as a Single Admin Concept

User-visible value:

- Operators can answer "why is this Addon not usable?" from one diagnostic
  surface: health, manifest mismatch, missing grants, network policy, resource
  declarations, tasks, event subscriptions, hosted pages, and secret references.

Problem:

- `surfaces.rs`, `diagnostics.rs`, `resource_search.rs`, and `routing.rs` each
  expose pieces of readiness. The information is good, but the operator mental
  model is split.

Evidence:

- `crates/nako-server/src/app/addons/surfaces.rs:17` checks health.
- `crates/nako-server/src/app/addons/surfaces.rs:63` checks runtime readiness.
- `crates/nako-server/src/app/addons/surfaces.rs:179` returns Addon surfaces.
- `crates/nako-server/src/app/addons/surfaces.rs:387` classifies network
  policy blockers.
- `crates/nako-server/src/app/addons/routing.rs:11` syncs routing plans.
- `crates/nako-server/src/app/addons/routing.rs:48` builds routing plans from
  tasks and event subscriptions.
- `crates/nako-server/src/app/addons/diagnostics.rs:14` diagnoses resource
  calls.
- `crates/nako-server/src/http/tests/addons.rs:3443` tests runtime readiness
  for network policy blockers without URL leaks.

Recommendation:

- Create an Admin-only "Addon Runtime Readiness Summary" module that composes
  existing checks without changing the Addon Protocol.
- Keep raw diagnostics and one-off resource-call diagnostic routes intact; this
  is a summary/read-model improvement.

Risk:

- Low-medium. The main risk is changing response shape if this enters API
  contracts. Keep additive fields only.

Parallelizability:

- High. Can run beside install-guide/source-catalog polish.

Likely gates:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server addon_runtime_readiness --no-fail-fast`
- `cargo nextest run -p nako-server admin_addon --no-fail-fast`
- `cargo nextest run -p nako-api admin_network --no-fail-fast`

### 6. Generated Artifact Intake Convergence for Addons and Automation

User-visible value:

- Addon-produced suggestions and Automation Provider suggestions feel like one
  reviewable workflow instead of separate internal channels.
- This improves metadata stewardship without allowing direct unreviewed
  canonical mutation.

Problem:

- Automation has a deep Generated Artifact Acceptance Workflow and Metadata
  Authority apply path.
- Addons can submit side effects and generated artifacts through runtime routes,
  but the shape is distributed across Addon runtime, Automation service, and
  metadata application logic.

Evidence:

- `crates/nako-server/src/http/addons.rs:751` exposes Addon Generated Artifact
  submission.
- `crates/nako-addon-protocol/src/lib.rs:12` defines the Addon runtime
  Generated Artifacts path.
- `crates/nako-server/src/app/automation.rs:251` lists Generated Artifact
  proposals.
- `crates/nako-server/src/app/automation.rs:290` plans Generated Artifact
  review.
- `crates/nako-server/src/app/automation.rs:362` plans metadata authority
  apply.
- `crates/nako-server/src/app/automation.rs:736` creates durable bulk apply
  batches.
- `crates/nako-core/src/automation.rs:309` defines `GeneratedArtifactProposal`.
- `crates/nako-core/src/automation.rs:402` defines
  `GeneratedArtifactAcceptancePlan`.
- `crates/nako-core/src/repository/automation.rs:18` owns the automation
  repository contract.

Recommendation:

- Do not attempt a full merge in a 10-hour goal.
- Start with a read-only contract audit and a tiny refactor that makes the
  Addon submission path name the same provenance/readiness concepts as the
  Automation path.
- Any public DTO or schema change should be split into a separate Trellis task.

Risk:

- High. This touches canonical metadata, Provider Mapping apply, Generated
  Artifact recovery, Admin API, and likely Web Admin.

Parallelizability:

- Low for implementation. Good as a read-only research lane.

Likely gates:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server generated_artifact --no-fail-fast`
- `cargo nextest run -p nako-server automation --no-fail-fast`
- `cargo nextest run -p nako-server addon_generated_artifact --no-fail-fast`
- `cargo nextest run -p nako-api admin_automation --no-fail-fast`

### 7. Remote Access Endpoint Diagnostics and Client Discovery Slice

User-visible value:

- Self-hosted operators can expose Nako through reverse proxies or tunnels and
  clients can understand local vs remote endpoints.

Problem:

- Config, preflight, Admin diagnostics, and network boundary logic already
  exist. `CONTROL_PLANE.md` still marks endpoint discovery as not started.

Evidence:

- `docs/architecture/CONTROL_PLANE.md:52` marks endpoint discovery as not
  started.
- `docs/architecture/CONTROL_PLANE.md:290` describes the remote access and
  endpoint discovery lane.
- `crates/nako-server/src/config.rs:124` defines `NetworkAccessConfig`.
- `crates/nako-server/src/config.rs:1297` tests network access policy parsing.
- `crates/nako-server/src/http/admin.rs:2170` renders network diagnostics.
- `crates/nako-server/src/http/network.rs:31` enforces network boundary policy.
- `crates/nako-server/src/http/tests/system.rs:9655` tests origin policy and
  auth ordering.
- `crates/nako-server/src/http/tests/system.rs:9797` tests forwarded host
  trust policy.

Recommendation:

- Add a minimal Admin/Public read model for endpoint hints only after deciding
  whether it belongs to Public Client API or Admin API.
- This is not Addon-specific, but it is one of the best 10-hour user-visible
  control-plane tasks.

Risk:

- Medium. It can affect security and cache/playback-ticket behavior.

Parallelizability:

- Medium-high if scoped to read-only diagnostics.
- Serial dependency if it changes Public Client API.

Likely gates:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server network_boundary --no-fail-fast`
- `cargo nextest run -p nako-server admin_network --no-fail-fast`
- `cargo nextest run -p nako-api admin_network --no-fail-fast`

### 8. API Scale and Cache Contract Audit Tests

User-visible value:

- Large libraries remain usable on TV, mobile, and low-power clients.
- Fewer accidental unbounded list responses and fewer unsafe cache headers.

Problem:

- `CONTROL_PLANE.md` says pagination/cache/N+1 discipline is partial.
- Many app surfaces use `PageRequest`, but several governance summaries still
  loop over `PageRequest::MAX_LIMIT` internally and should be audited with
  large-library tests before more features stack on them.

Evidence:

- `docs/architecture/CONTROL_PLANE.md:53` marks API version/error/page
  contracts as shipped foundation but still needing cursor pagination and
  large-library contracts.
- `docs/architecture/CONTROL_PLANE.md:54` marks HTTP cache/ETag contracts as
  narrow shipped partial.
- `docs/architecture/CONTROL_PLANE.md:55` marks N+1/list projection discipline
  as partial.
- `crates/nako-server/src/app/catalog.rs:69` performs paged internal looping
  for governance summaries.
- `crates/nako-server/src/app/automation.rs:1043` lists Generated Artifact
  proposals at `PageRequest::MAX_LIMIT`.
- `crates/nako-server/src/app/automation.rs:1157` iterates bulk apply planning
  with max-limit pages.
- `crates/nako-server/src/http/catalog.rs:508` applies private artwork cache
  headers.
- `crates/nako-api/src/public_client.rs:70` adapts server pagination to client
  `PageInfo`.

Recommendation:

- Use the 10-hour goal to add targeted scale/regression tests, not to redesign
  pagination.
- Pick two hot surfaces: Public catalog list/detail and Admin Generated
  Artifact proposal/recovery lists.

Risk:

- Low for tests, medium if fixes touch query contracts.

Parallelizability:

- Very high as a read-only/test-first lane.

Likely gates:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server catalog --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact --no-fail-fast`
- `cargo nextest run -p nako-api public_client --no-fail-fast`

## Recommended 10-Hour Parallel Campaign

Use four parallel read-only or low-risk lanes first, then converge into one or
two implementation tasks.

### Hours 0-1: Planner Setup

- Confirm clean `main`, task directory, and current specs.
- Do not open broad API/schema changes.
- Split workers by owned scopes:
  - Worker A: Addon Task Dispatch Runtime.
  - Worker B: Outbound Addon Call Engine.
  - Worker C: Official Addon Install Guide / Source Catalog / Readiness.
  - Worker D: Webhook/Add-on Event delivery parity.
  - Optional Worker E: Remote Access Endpoint diagnostics.
  - Optional Worker F: API scale/cache audit tests.

### Hours 1-3: Parallel Recon and Micro-Design

- Worker A returns an interface sketch and exact task-runtime tests.
- Worker B returns a deletion-test result for repeated outbound call helpers.
- Worker C returns an Admin operator journey and route/test gaps.
- Worker D returns whether Webhook scheduler can use existing attempt records
  without schema changes.
- Optional workers report whether their surfaces are safe for implementation in
  the same 10-hour window.

Stop condition:

- If Worker A and Worker B both need to rewrite the same
  `nako-addon-client` failure model, serialize B before A.
- If Worker D needs schema changes, keep it as a follow-on and do not implement
  it inside this 10-hour goal.

### Hours 3-7: Implementation Candidates

Preferred pair:

1. Addon Task Dispatch Runtime deepening.
2. Official Addon Install Guide / Runtime Readiness polish.

Alternative pair if outbound-call evidence is simpler than expected:

1. Outbound Addon Call Engine.
2. API scale/cache audit tests.

Do not attempt in the same 10-hour goal:

- Addon Manager process lifecycle.
- Generated Artifact intake convergence implementation.
- Public Client endpoint discovery API shape.
- Schema migrations for scheduler parity.

### Hours 7-9: Quality and Integration

- Run focused package tests from each worker.
- Run `cargo check -p nako-server --tests`.
- Run `cargo fmt --all -- --check`.
- Run `git diff --check`.
- Validate the Trellis task.

### Hours 9-10: Docs, Spec, and Follow-On Split

- Update `.trellis/spec/nako-server/backend/directory-structure.md` if a new
  server-side module contract is added.
- Update `docs/architecture/CONTROL_PLANE.md` only if a control-plane baseline
  changes or a follow-on is retired.
- Open follow-on Trellis tasks for high-risk deferred work:
  - Webhook Scheduler parity.
  - Generated Artifact intake convergence.
  - Public endpoint discovery API.
  - Addon Manager lifecycle trust/update design.

## Exact Worker Prompts

### Worker A: Addon Task Dispatch Runtime

Inspect Addon Task dispatch in `crates/nako-server/src/app/addons/task_runtime.rs`,
`crates/nako-core/src/repository/addon_task.rs`, and scan-triggered tests in
`crates/nako-server/src/app/tests/startup.rs`. Do not modify files. Return a
minimal refactor plan that makes task dispatch a deeper module without changing
schema/API. Include exact tests and stop conditions.

### Worker B: Outbound Addon Call Engine

Inspect `crates/nako-addon-client/src/lib.rs` and all server callers in
`crates/nako-server/src/app/addons/*.rs`. Do not modify files. Rank repeated
call-envelope/auth/retry/error mapping patterns and propose a smallest
interface that reduces duplication without changing public Addon Protocol.

### Worker C: Addon Install Guide and Readiness

Inspect `crates/nako-server/src/app/addons/catalog.rs`,
`crates/nako-server/src/app/addons/surfaces.rs`,
`crates/nako-official-addon-catalog/src/lib.rs`, and HTTP tests under
`crates/nako-server/src/http/tests/addons.rs`. Do not modify files. Propose an
Admin operator-flow polish task that improves official Addon installation
without implying Nako manages containers, packages, logs, updates, or rollback.

### Worker D: Webhook and Addon Event Delivery Parity

Inspect `crates/nako-server/src/app/addons/event_runtime.rs`,
`crates/nako-server/src/app/webhooks.rs`, `crates/nako-events/src/lib.rs`, and
webhook/addon-event tests. Do not modify files. Decide whether a scheduler
parity slice can be implemented without schema change. If not, return an ADR or
Trellis follow-on shape instead.

### Optional Worker E: Remote Access Endpoint Diagnostics

Inspect `crates/nako-server/src/config.rs`, `crates/nako-server/src/http/admin.rs`,
`crates/nako-server/src/http/network.rs`, `crates/nako-api/src/admin/network.rs`,
and `docs/architecture/CONTROL_PLANE.md`. Do not modify files. Propose the
smallest endpoint diagnostics/discovery task and classify whether it requires
Public Client API changes.

### Optional Worker F: API Scale and Cache Audit

Inspect `docs/architecture/CONTROL_PLANE.md`, Public catalog routes, Admin
Generated Artifact routes, and pagination/cache tests. Do not modify files.
Return two high-value large-library regression tests and any obvious no-schema
fixes.

## Recommended First Implementation Choice

For a 10-hour goal, start with:

1. `AddonTaskDispatchRuntime` deepening.
2. Official Addon install-guide/readiness polish.

Reason:

- They produce real media-server value for scan-triggered Addon metadata
  workflows and self-hosted operator setup.
- They can run mostly in parallel.
- They do not require schema migration, public API redesign, or addon-manager
  lifecycle authority.
- They respect ADR 0003, ADR 0020, ADR 0034, and ADR 0053.
