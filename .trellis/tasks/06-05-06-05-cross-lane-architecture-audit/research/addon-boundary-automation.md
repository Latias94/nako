# Addon Boundary And Automation Audit

## Scope

Reviewed Addon vocabulary, control-plane architecture, lane ownership, ADRs
0003/0015/0020/0050/0051/0053, and implementation boundaries across:

- `crates/nako-addon-protocol`
- `crates/nako-addon-client`
- `crates/nako-official-addon-catalog`
- `crates/nako-automation`
- `crates/nako-server/src/app/addons*`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/http/automation.rs`
- `apps/admin-web/src/features/addons/AddonsPage.tsx`
- `web/src/features/admin/admin-plugins.tsx`

This is an audit-only note. No production implementation change is recommended
as part of this file.

## Current State

- **The extension model is already defined as Addon Sidecar plus Nako-owned
  control plane.** `CONTEXT.md:455` through `CONTEXT.md:491` separates Addon
  Version, Addon Protocol Version, Addon Package/Suite, Addon Token,
  Library-Scoped Addon Grant, Addon Side Effect, Addon Task, Addon Hosted Page,
  and future Addon Manager concerns.
- **ADR direction is consistent.** ADR 0003 chooses HTTP Addons before
  in-process plugins (`docs/adr/0003-http-addons-before-in-process-plugins.md:25`),
  ADR 0015 requires capability-scoped HTTP addons and bounded workers
  (`docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md:30`,
  `:57`), ADR 0020 allows strong Addon side effects only through Nako-owned
  APIs and scoped tokens (`docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md:39`),
  and ADR 0053 keeps managed sidecars out-of-process even if Nako later manages
  installation or process lifecycle (`docs/adr/0053-application-control-plane-boundary.md:60`).
- **Protocol surface is broad but still contract-shaped.** `nako-addon-protocol`
  owns manifest, resources, hosted pages, event subscriptions, tasks, scopes,
  permissions, resource search, external acquisition, subtitle search, side
  effects, and validation helpers (`crates/nako-addon-protocol/src/lib.rs:134`,
  `:274`, `:384`, `:453`, `:483`, `:553`, `:622`, `:959`, `:1019`, `:1167`,
  `:1880`, `:1940`, `:2244`, `:2304`, `:2473`, `:2521`, `:2577`). This is a
  large file, but the contents are wire contract and validation, not server
  persistence or routing policy.
- **Server Addon foundation is not a stub.** Admin routes cover registration,
  catalog, health, runtime readiness, surfaces, routing plans, event attempts,
  event delivery/replay, task runs, install guide, manager plan, diagnostics,
  resource search, subtitle import, tokens, and grants
  (`crates/nako-server/src/http/addons.rs:40` through `:170`). Runtime routes
  expose Addon token access check, side effects, generated artifacts,
  acquisition candidates, external acquisition materialization, and sidecar
  task claim/progress/complete/fail/cancel (`crates/nako-server/src/http/addons.rs:174`
  through `:208`).
- **Persistence exists for the important control-plane entities.**
  `nako-core` exposes repository contracts for registrations, token lifecycle,
  grants, routing plans, Addon task runs, and event delivery attempts
  (`crates/nako-core/src/repository/addon.rs:11`, `:40`, `:50`, `:56`, `:58`,
  `:70`; `crates/nako-core/src/repository/addon_task.rs:12`;
  `crates/nako-core/src/repository/addon_event.rs:13`). SQLite and PostgreSQL
  backends implement those contracts.
- **Automation is correctly separated from Addon authority.** `nako-automation`
  queues `JobKind::Automation` with resource class `automation.external_api`
  (`crates/nako-automation/src/lib.rs:105` through `:108`), persists provider
  output as automation artifacts (`crates/nako-automation/src/lib.rs:151`), and
  rejects direct canonical metadata mutation (`crates/nako-automation/src/lib.rs:227`).
  Server-side automation then routes accepted Generated Artifacts through
  acceptance and metadata-apply planning rather than treating the provider as a
  trusted metadata writer (`crates/nako-server/src/app/automation.rs:293`,
  `:365`, `:895`, `:991`, `:1629`).
- **Official catalog is descriptor/catalog material, not lifecycle management.**
  It already covers metadata scraper, resource search, external acquisition
  runner, subtitle provider, renderer adapters, and notification bridge shapes
  (`crates/nako-official-addon-catalog/src/lib.rs:76`, `:306`, `:571`, `:792`,
  `:990`, `:1160`, `:1340`). It should not be treated as package signing,
  process supervision, or a complete marketplace.
- **Admin Web has two Addon surfaces.** The new Admin Web route is broad but
  mostly diagnostic/read-oriented, showing registry, health, surface counts,
  tokens, grants, and install boundary state
  (`apps/admin-web/src/features/addons/AddonsPage.tsx:141`, `:251`, `:281`,
  `:312`, `:370`). The older `web` Admin Plugins page has live status mutation
  and catalog cards, but Grants and Hosted pages are still disabled
  (`web/src/features/admin/admin-plugins.tsx:44`, `:85`, `:238`, `:244`, `:248`).

## Findings

### P1: Host-Owned Resource Flow Rules Are Repeating

Resource Search and Subtitle now both implement in-memory selection sessions
(`crates/nako-server/src/app/addons/resource_search.rs:64`,
`crates/nako-server/src/app/addons/subtitles.rs:79`), grant checks, safe error
codes, and host-owned selection/apply handoffs. External acquisition has a
similar host-owned materialization boundary
(`crates/nako-server/src/app/addons/external_acquisition.rs:113`, `:167`,
`:218`, `:408`).

This repetition is not yet a bug, but it is the highest-value architecture
pressure in this lane. ADR 0050 and ADR 0051 explicitly require host-owned
selected-link/subtitle target derivation and Nako-owned writes
(`docs/adr/0050-acquisition-resource-action-boundaries.md:31`,
`:41`, `:44`; `docs/adr/0051-host-owned-subtitle-import-chain.md:24`,
`:33`, `:41`). As more resource types arrive, each flow should not invent its
own session TTL, selected-reference shape, safe diagnostic taxonomy, redaction
policy, and apply-plan vocabulary.

Recommended next task: design a shared "host-owned Addon resource flow" app
pattern before adding another resource-specific product flow. Keep it inside
server/app boundaries at first; do not push this policy into
`nako-addon-protocol`.

### P1: Addon Task Execution Has Two Modes That Need One Policy Language

Nako supports sidecar-claim task runs and direct dispatch paths. Task runtime
validates declaration scopes, normalizes safe error codes, and maps client
failures to safe host diagnostics (`crates/nako-server/src/app/addons/task_runtime.rs:56`,
`:653`, `:747`, `:855`, `:899`, `:915`). Event runtime separately filters by
grants, creates persisted delivery attempts, and records safe failure state
(`crates/nako-server/src/app/addons/event_runtime.rs:199`, `:216`, `:365`,
`:429`, `:609`, `:641`, `:818`, `:837`).

This is healthy control-plane depth, but direct dispatch and claim-based
execution should converge on one vocabulary for resource class, retry,
cancellation, trace identity, redacted output, and operator diagnostics. ADR
0053 warns against each runtime inventing its own queue, retry, trace, or
resource policy (`docs/adr/0053-application-control-plane-boundary.md:89`,
`:94`).

Recommended next task: audit and specify Addon task/event execution policy
across direct dispatch, durable job task runs, and event delivery attempts.
Implementation can then be split into small follow-ons.

### P1: Addon Manager Is A Product Layer Gap, Not A Protocol Gap

The first Addon Manager should focus on registry, permissions, token rotation,
health check, and install guide behavior, not Docker/process lifecycle
(`CONTEXT.md:490`, `:491`; `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md:54`,
`:85`). Control-plane docs keep process lifecycle deferred
(`docs/architecture/CONTROL_PLANE.md:355`, `:357`, `:369`) and warn that a
managed sidecar is still not trusted in-process code
(`docs/architecture/CONTROL_PLANE.md:393`, `:395`, `:396`).

The server already exposes install-guide and manager-plan routes
(`crates/nako-server/src/http/addons.rs:113`, `:117`). The missing piece is a
coherent operator workflow that connects catalog entry, manifest registration,
grant review, token issuance/rotation, health, install guide, and runtime
readiness.

Recommended next task: an Addon Manager product PRD and thin implementation
slice. Avoid container/process control until signing, package inventory, update
policy, log redaction, and sidecar trust model are explicitly decided.

### P2: Admin Surfaces Can Drift

There are two Addon management surfaces with different strengths:

- `apps/admin-web` is strong on diagnostics and boundary display, but does not
  appear to be the full lifecycle surface yet.
- `web` has live status mutation, but important actions such as Grants and
  Hosted pages are disabled.

This is a parallelization risk because future Addon Manager work will likely
touch API DTOs, generated contracts, Admin route state, and old/new Admin UI
at the same time.

Recommended next task: choose the primary Admin surface for Addon Manager work
before implementation. If both must remain, define a temporary contract owner
and acceptance test shape for route parity.

### P2: Diagnostics And Redaction Are Correct In Spirit But Scattered

Diagnostics modules already avoid leaking raw sidecar/provider detail:
resource diagnostics map missing grants to safe codes
(`crates/nako-server/src/app/addons/diagnostics.rs:49`, `:61`, `:111`),
resource search redacts source refs (`crates/nako-server/src/app/addons/resource_search.rs:531`,
`:615`, `:625`), and external acquisition materialization redacts source URIs
(`crates/nako-server/src/app/addons/external_acquisition.rs:151`). This matches
the control-plane requirement to avoid raw paths, tokens, credentials, provider
payloads, and secrets in diagnostics (`docs/architecture/CONTROL_PLANE.md:150`,
`:174`, `:263`, `:383`).

The risk is not immediate leakage in the reviewed paths; the risk is taxonomy
drift as each resource flow adds its own `safe_error_code` and redaction
helpers.

Recommended next task: make a small server-side diagnostics/redaction policy
module or spec for Addon resource flows after the host-owned flow design is
accepted.

## Candidate Next Tasks

### 1. Host-Owned Addon Resource Flow Pattern

**Type**: architecture audit plus bounded refactor.

Define reusable server-side conventions for:

- selection session storage and expiry;
- selected reference identity;
- apply-plan and apply-result structure;
- safe error code taxonomy;
- source-ref redaction;
- permission/grant check placement;
- relationship between read-only discovery and explicit side-effect authority.

Parallel safety:

- Serializes with new resource search/subtitle/external acquisition product
  work.
- Can run in parallel with remote access or playback work if `nako-api` DTO
  changes are avoided or coordinated.

### 2. Addon Task/Event Execution Policy Convergence

**Type**: architecture audit before implementation.

Unify policy language for Addon direct dispatch, sidecar-claim task runs, and
event delivery attempts. The goal is not one runtime implementation; the goal
is shared semantics for durability, retry, resource class, cancellation,
redacted diagnostics, trace context, and operator-visible state.

Parallel safety:

- Serializes with broad durable-job scheduler migration.
- Safe with Admin UI-only work if API shapes are stable.

### 3. Addon Manager First Product Slice

**Type**: product/architecture PRD, then bounded implementation.

Build the first manager around registry, catalog resolution, install guide,
grant review, token issue/rotate/revoke, health, runtime readiness, and hosted
surface links. Do not add Docker socket access, process launch, auto-update, or
package signing in this slice.

Parallel safety:

- Serializes with Admin Addon route and generated contract work.
- Can run after Admin surface ownership is decided.

### 4. Hosted Page Safe Surface Contract

**Type**: product/security decision before implementation.

Clarify how Addon Hosted Pages are surfaced without receiving Nako admin
credentials (`CONTEXT.md:481`; `CONTEXT.md:731`). Decide URL launch mechanics,
allowed context, token exchange or no-token policy, CSP/frame policy, and
operator warnings.

Parallel safety:

- Serializes with Addon Manager UI work.
- Mostly independent from Automation and durable job internals.

### 5. Automation/Addons Generated Artifact Governance Check

**Type**: verification/spec task.

Automation is currently on the right boundary: external providers produce
artifacts, and canonical mutation goes through acceptance/apply. Addon
generated-artifact submission should be periodically checked against the same
authority model so Addons do not become a shortcut around Metadata Authority or
Acceptance Workflow.

Parallel safety:

- Serializes with metadata authority/apply changes.
- Safe with Addon Manager registry/token UI work.

## Fearless Refactor Candidates

No high-confidence production deletion or crate split is recommended now.

Do **not** prioritize a mechanical split of:

- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-official-addon-catalog/src/lib.rs`

Those files are large, but the observed problem is not that protocol, client,
or catalog boundaries are mixed with server persistence. The real complexity is
the server-side product/control-plane layer where host-owned resource flows,
diagnostics, grants, tasks, events, and Admin surfaces meet.

Good future refactor target:

- Extract a server-local host-owned resource-flow pattern once one more flow or
  a dedicated Addon Resource Flow task proves the common shape. This should
  reduce duplication across resource search, subtitle import, and external
  acquisition without moving policy into the permissive protocol crate.

## Product Decisions Blocking Implementation

- Which Admin surface is authoritative for Addon Manager: `apps/admin-web` or
  legacy `web`?
- Does the first Addon Manager stop at install guides and health checks, or is
  any package inventory included?
- Are Hosted Pages opened as external links only, framed surfaces, or a
  mediated route with explicit no-admin-token guarantees?
- What is the stable policy for converting read-only resource discovery into
  external acquisition actions?
- Will official Addons be distributed as individual Addon Packages or grouped
  Addon Suites while preserving per-Addon grants and audit?

## Recommended Priority

1. **Architecture audit**: host-owned Addon resource flow pattern.
2. **Architecture audit**: Addon task/event execution policy convergence.
3. **Product PRD**: Addon Manager first slice without process lifecycle.
4. **Security/product decision**: Hosted Page safe surface contract.
5. **Verification/spec**: keep Automation and Addon Generated Artifact paths
   aligned with Acceptance Workflow and Metadata Authority.

Overall recommendation: use a mixed plan. This lane is not blocked on missing
foundation and is not ready for broad fearless refactor. The next high-value
work is boundary deepening plus a small productized Addon Manager slice after
the Admin surface owner is chosen.
