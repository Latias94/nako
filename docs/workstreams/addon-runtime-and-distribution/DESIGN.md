# Addon Runtime And Distribution Design

Status: Complete
Last updated: 2026-05-22

## Why This Lane Exists

Taru has already chosen the correct Addon architecture: Addons are HTTP
**Addon Sidecars** speaking the **Addon Protocol**, not Native Plugins and not
Jellyfin Plugin Compatibility. The implementation now has manually registered
Addon Sidecars, Addon Tokens, Library-Scoped Addon Grants, Addon Side Effects,
Canonical Metadata protected writes, Addon Artwork Candidate proposals, NFO
Library File Write boundaries, Admin Addon lifecycle/health/surface
diagnostics, Network Access Boundary policy, and AI Generated Artifact review
semantics.

The remaining product risk is distribution. Operators need a repeatable way to
understand what an addon requires, how to run it, whether it is healthy, and
which Taru-owned queues or side-effect APIs it may use. If Taru jumps straight
to an Addon Manager, Native Plugin ABI, marketplace, or process supervisor, it
will blur trust boundaries before the package/runtime contract is stable.

The first-principles boundary is therefore: **Taru can describe, validate, and
diagnose sidecar addon packages and runtime declarations, but Addon code still
runs outside Taru and all strong effects still pass through Taru-owned APIs.**

## Relevant Authority

- Glossary and policy:
  - `CONTEXT.md` — Addon, Addon Protocol, Addon Sidecar, Addon Manager,
    Addon Install Guide, Addon Task, Addon Event Subscription, Library File
    Write, Generated Artifact.
- ADRs:
  - `docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
- Completed boundaries:
  - `docs/workstreams/addons-automation`
  - `docs/workstreams/addon-architecture-deepening`
  - `docs/workstreams/admin-addon-operations-mvp`
  - `docs/workstreams/downloads-watch-folder-intake`
  - `docs/workstreams/network-access-boundary`
  - `docs/workstreams/ai-assisted-library-ops`
- Related code:
  - `crates/taru-addon-protocol/src/lib.rs`
  - `crates/taru-addon-client`
  - `crates/taru-core/src/addon.rs`
  - `crates/taru-core/src/repository/addon.rs`
  - `crates/taru-db`
  - `crates/taru-server/src/app/addon*`
  - `crates/taru-server/src/http/addons.rs`
  - `crates/taru-api/src/admin.rs`

## Problem

Manual Addon registration proves the protocol and side-effect model, but it
does not yet answer product-critical distribution/runtime questions:

- how Taru validates an addon package or install descriptor before an operator
  runs it;
- how install guidance is generated without embedding credentials, admin
  tokens, local paths, or host-specific secrets;
- how package manifests declare tasks, event subscriptions, hosted pages,
  configuration schema, Secret References, generated artifacts, and library
  side effects in a way that is safe to expose in Admin diagnostics;
- how Taru distinguishes sidecar runtime health, protocol mismatch, grant
  mismatch, network readiness, and missing Secret Reference configuration;
- how Addon Tasks and Addon Event Subscriptions enter existing job/outbox
  boundaries instead of hidden schedulers;
- how Addon-produced Generated Artifacts and acquisition outputs route through
  existing proposal/intake/acceptance workflows;
- how to make distribution useful without becoming an Addon Manager or Native
  Plugin runtime.

## Target State

When this lane closes:

- Taru has a stable Addon Package / Install Guide boundary for sidecar addons.
- Addon manifest/package validation reports install/runtime requirements,
  missing configuration, required grants, declared tasks/events/resources, and
  network readiness without exposing secrets or host paths.
- Admin-only diagnostics let an operator understand package/install/runtime
  readiness and redacted failure categories.
- Runtime checks can prove a sidecar is reachable, protocol-compatible, grant
  compatible, and safe to call before tasks/events/resources are enabled.
- Declared Addon Tasks and Event Subscriptions are routed to Taru-owned job and
  outbox/side-effect boundaries or are explicitly blocked as deferred runtime
  behavior.
- Addon-produced acquisition candidates and Generated Artifacts consume
  `downloads-watch-folder-intake` and `ai-assisted-library-ops` boundaries
  rather than writing Canonical Metadata, NFO sidecars, Media Sources, or
  library files directly.
- `taru-addon-protocol` remains permissive and dependency-light; AGPL server
  code does not leak into protocol crates.
- Public Client API and `taru-client-protocol` remain unchanged unless a
  dedicated client-contract lane is opened.

## In Scope

- Addon Runtime / Distribution workstream docs and task ledger.
- Addon package/install descriptor vocabulary and validation.
- Addon Install Guide generation or preview that is safe to show to operators.
- Admin-only package/runtime readiness diagnostics and typed Admin Web support.
- Runtime health/readiness checks for sidecar reachability, protocol version,
  manifest compatibility, grants, Secret Reference presence, and network
  policy blockers.
- Declaration routing for Addon Tasks and Addon Event Subscriptions into
  existing Taru-owned job/outbox/side-effect boundaries, with explicit blockers
  where runtime execution remains deferred.
- Generated Artifact and Acquisition Intake handoff from Addons through
  existing proposal/intake/acceptance workflows.
- Tests proving no direct library writes, no autonomous metadata writes, no
  admin-token leakage, and no Public Client protocol churn.

## Out Of Scope

- Native Plugin ABI, WASI runtime, embedded JavaScript runtime, or Jellyfin
  Plugin Compatibility.
- Addon Manager automatic discovery, marketplace hosting, package download,
  package signing trust root, update/rollback orchestration, removal, or
  process/container supervision.
- Passing administrator bearer tokens to Addon Sidecars.
- Direct filesystem, Source Locator, database, or storage credentials for
  Addons.
- Direct Canonical Metadata, NFO sidecar, Media Source, Managed Import, or
  library-file writes outside Taru-owned APIs.
- Public Client API/SDK changes.
- Concrete protocol downloader adapters unless routed through a split
  downloader lane.
- Local AI model runtime, embeddings/vector DB, or provider-specific AI
  adapters.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| HTTP Addon Sidecars remain the extension runtime. | High | ADR 0003, ADR 0020, Addon Architecture Deepening | Reopen extension-model ADRs before changing runtime. |
| Distribution must start as validation/install guidance, not process supervision. | High | Admin Addon Operations MVP explicitly deferred Addon Manager automation | If operators need managed processes first, split an Addon Manager lane. |
| Addon package/protocol crates should stay permissive and dependency-light. | High | ADR 0022 and current `taru-addon-protocol` crate boundary | If package helpers need transport/runtime dependencies, split helper crates. |
| Addon Tasks and Event Subscriptions should reuse job/outbox boundaries. | High | ADR 0014/0015 and durable job/outbox workstreams | If a new scheduler is needed, split task-runtime ownership separately. |
| Addon outputs should use existing proposal/intake workflows. | High | DWI and AILO closeouts | If an Addon needs a new effect class, define a Protected Write or side-effect lane. |

## Architecture Direction

Keep the runtime layered:

```text
taru-addon-protocol
  Own permissive package/manifest/install descriptor wire types and validation
  primitives that Addon authors can reuse without server internals.

taru-addon-client
  Own optional sidecar HTTP checks and protocol helpers when transport
  behavior is useful outside server internals.

taru-core / taru-db
  Own durable package/install/runtime readiness records only when runtime state
  must be audited or retried.

taru-server::app
  Own package validation, install-guide generation, runtime readiness,
  declaration routing, and side-effect/proposal/intake handoff.

taru-api::admin / taru-server::http::admin
  Own Admin-only diagnostics and controls. Public Client API stays untouched.
```

Use vertical slices. Do not introduce a package store, process supervisor, or
marketplace until the validation/readiness contract proves useful.

## Closeout Condition

This lane can close when:

- package/install descriptor semantics are explicit and tested;
- Admin diagnostics expose package/runtime readiness without leaking secrets,
  admin tokens, raw URLs with credentials, local paths, Source Locators, or raw
  sidecar payloads;
- sidecar runtime checks distinguish reachability, protocol mismatch, manifest
  mismatch, grant/config gaps, and network policy blockers;
- declared Addon Tasks/Event Subscriptions are either routed through Taru-owned
  boundaries or blocked with explicit deferred reasons;
- Addon-produced Generated Artifacts and acquisition outputs are proven to use
  existing AILO/DWI workflows without autonomous writes;
- Public Client API and `taru-client-protocol` remain unchanged;
- Addon Manager, package signing, marketplace, process supervision, Native
  Plugin ABI, downloader protocols, and local AI runtime are split follow-ons.

## Closeout — 2026-05-22

This lane is complete. Taru now has the sidecar Addon package/install
descriptor boundary, redacted install-guide preview, Admin-only runtime
readiness diagnostics, declared task/event routing plans, and Addon Generated
Artifact / acquisition-intake handoff into existing Taru-owned AILO and DWI
semantics.

The lane intentionally did **not** ship Addon Manager discovery/install/update,
marketplace hosting, package signing trust roots, process/container
supervision, logs/rollback, Native Plugin ABI, Jellyfin Plugin Compatibility,
downloader protocol adapters, local AI/model runtime, Public Client surfaces,
direct library writes, hidden schedulers, or `taru-client-protocol` changes.
Those remain explicit follow-ons that require dedicated workstreams.
