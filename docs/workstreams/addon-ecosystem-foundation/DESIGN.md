# Addon Ecosystem Foundation

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

Nako has a strong Addon Sidecar direction, but the next ecosystem features
should not land on shallow seams. Notification bridges, watch-state sync, MCP
media steward agents, Arr-stack integration, DLNA/UPnP/WebDAV compatibility,
and remote tunnel integrations all depend on the same load-bearing platform
behaviors:

- event delivery from Nako to Addon Sidecars;
- host-owned Addon Task lifecycle correctness;
- package and catalog facts that do not drift from official addon manifests;
- operator-friendly deployment that avoids one Compose service per capability;
- Generated Artifact and Acceptance Workflow boundaries for AI-like output.

This lane deepens those seams before broad official addon feature work.

## Relevant Authority

- ADRs:
  - `docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
  - `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
  - `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/ROADMAP.md`
  - `docs/GOALS.md`
  - `docs/guides/ADDON_AUTHOR_GUIDE.md`
- Related workstreams:
  - `docs/workstreams/addon-architecture-deepening/`
  - `docs/workstreams/addon-runtime-and-distribution/`
  - `docs/workstreams/addon-task-runtime-contract/`
  - `docs/workstreams/addon-source-catalog-marketplace/`
  - `docs/workstreams/network-access-boundary/`
  - `docs/workstreams/ai-assisted-library-ops/`

## Problem

The current Addon ecosystem has enough surface area to prove sidecar calls,
protected writes, manager plans, catalog discovery, and direct task dispatch,
but several seams are still too shallow for the next wave:

- Addon Event Subscriptions are manifest declarations, but Nako does not yet
  deliver declared domain events to Addon Sidecars.
- Addon Task run idempotency replays on `addon_id + idempotency_key` without a
  request fingerprint, unlike the hardened Addon Side Effect path.
- The built-in official addon source catalog can drift from the actual
  official addon manifest and release facts.
- Official addons need an Addon Suite distribution strategy so users do not
  hand-write many Docker Compose services.
- Future feature ideas need a stable order so core does not absorb network
  tunnel, AI, notification, or compatibility-protocol responsibilities.

## Target State

When this workstream closes:

- `CONTEXT.md` names Addon Package and Addon Suite as first-class deployment
  concepts.
- Addon Task run creation uses deterministic request fingerprints and rejects
  mismatched idempotency-key reuse.
- The built-in official addon catalog descriptor is synchronized with the
  official addon manifest/task/config facts or guarded by a drift test.
- Nako has a host-owned Addon Event Delivery Runtime built on the durable event
  outbox, manifest event subscriptions, grants, tokens, retry/backoff, and
  redacted diagnostics.
- The official addon repository has the first event-driven proof addon or
  suite-facing event echo path.
- Future addon feature tiers are recorded and split instead of hidden in this
  lane.

## In Scope

- Addon Package and Addon Suite terminology and ADR authority.
- Addon Task request fingerprint schema, repository, and HTTP/runtime behavior.
- Official addon catalog descriptor synchronization for the metadata scraper
  and Addon Suite planning.
- Addon Event Subscription delivery from Nako to Addon Sidecars.
- A minimal official event-driven addon proof path.
- Documentation of future feature tiers and non-goals.

## Out Of Scope

- Native Plugin ABI.
- Jellyfin Plugin Compatibility.
- Nako-managed Docker socket, systemd, Kubernetes, SSH, host-agent, log
  collection, rollback execution, or process supervision.
- Built-in NAT traversal, relay infrastructure, TURN server operation, or
  direct Tailscale/Headscale control inside Nako core.
- Direct AI mutation of Canonical Metadata or User Playback State.
- Full notification provider matrix, full watch-state sync, full Arr-stack
  integration, full MCP media steward, full DLNA/UPnP/WebDAV compatibility, or
  package signing trust roots in the first slices.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Sidecar remains the right extension trust model. | High | ADR 0003, ADR 0020, completed Addon workstreams. | Reopen ADR 0020 before feature work. |
| Deployment should be coarse-grained while permissions stay fine-grained. | High | Operator Compose concerns and ADR 0034. | Addon Manager would need per-addon service sprawl mitigation later. |
| Event delivery unlocks more future addon value than another scraper feature. | High | Notification, watch sync, Arr, and MCP all need events. | Reorder breadth after event delivery proof. |
| Addon Task idempotency needs fingerprint parity with Addon Side Effects. | High | Existing Side Effect fingerprint work and task run replay code. | Long tasks can accidentally replay a different request. |
| Network tunnel behavior should stay outside Nako core first. | High | `CONTEXT.md` and network-access-boundary decisions. | A later ADR must explicitly grant core network tunnel authority. |

## Architecture Direction

The core principle is fine-grained permissions with coarse-grained deployment.

Nako owns:

- Addon manifests, grants, tokens, health, routing plans, and diagnostics;
- durable event outbox storage and Addon Event Delivery;
- Addon Task lifecycle, progress, cancellation, retry, result, and audit;
- Generated Artifact and Acceptance Workflow state;
- protected Addon Side Effects, Managed Artwork, and Library File Write
  application.

Addon Sidecars own:

- provider-specific external fetches;
- third-party credentials resolved through Secret References;
- protocol translation such as DLNA/UPnP/WebDAV compatibility;
- notification, watch-state sync, MCP, and Arr-stack adapter behavior;
- network tunnel provider interaction when accepted by a future lane.

Addon Packages and Addon Suites own deployment shape. They must not weaken the
manifest, grant, or protocol compatibility boundary. A single official suite
process can expose several Addons, but Nako still registers, grants, audits,
and routes each Addon independently.

## Future Feature Tiers

Tier 0 foundation:

- Addon Task request fingerprinting;
- official catalog/descriptor synchronization;
- Addon Event Subscription delivery;
- official event echo proof.

Tier 1 official automation:

- notification bridge;
- watch-state sync skeleton;
- generated-artifact based MCP media steward read-only proof;
- suite deployment snippets and profiles.

Tier 2 compatibility and acquisition:

- WebDAV compatibility surface as a client-facing virtual library adapter;
- DLNA/UPnP sidecar with LAN discovery isolation;
- Arr-stack integrator after acquisition/subscription concepts are stable.

Tier 3 remote and AI depth:

- Network Tunnel Provider sidecar planning around Remote Access Endpoints;
- MCP media steward write flows through Generated Artifacts and Acceptance
  Workflow;
- provider runtime deepening for field-level evidence and budgets.

## Closeout Condition

This lane can close when:

- AEF-010 through AEF-060 are complete or split with explicit follow-on docs;
- final gates are recorded in `EVIDENCE_AND_GATES.md`;
- `WORKSTREAM.json` and `HANDOFF.md` reflect the final state;
- no event/task/catalog correctness work remains hidden in future feature
  buckets.
