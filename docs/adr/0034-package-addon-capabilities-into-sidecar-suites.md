# 0034: Package Addon Capabilities Into Sidecar Suites

## Status

Accepted.

## Context

Nako has chosen HTTP **Addon Sidecars** instead of in-process plugins. That
keeps extension code outside the server trust boundary, lets addons be written
in any HTTP-capable language, and lets Nako keep authority over protected
writes, tasks, event delivery, generated artifacts, and library-file writes.

The next concern is operator experience. If every small capability required a
separate Docker Compose service, a useful self-hosted Nako deployment would
become noisy quickly:

- one sidecar for metadata;
- one sidecar for notifications;
- one sidecar for watch-state sync;
- one sidecar for an MCP media steward;
- one sidecar for Arr-stack integration;
- one sidecar for DLNA or WebDAV compatibility;
- one sidecar for remote access tunnel integration.

That is the wrong deployment experience. The **Addon** abstraction should stay
fine-grained for permission, manifest, task, event, and audit purposes, but the
deployment unit should be allowed to be coarse-grained.

Comparable ecosystems separate extension identity from distribution shape.
Stremio addons are discovered through manifests and can be hosted by any HTTP
process. Home Assistant integrations and VS Code extensions are distributed as
packages that can contain multiple capabilities. Docker Compose deployments
usually prefer a small number of operational services, not one service per
minor feature.

## Decision

Nako will distinguish these concepts:

1. **Addon** is the Nako-visible capability and authority unit. It declares
   resources, tasks, event subscriptions, entry points, configuration schema,
   secret references, required scopes, and protocol compatibility.
2. **Addon Sidecar** is an independently running process or service that
   implements the Addon Protocol for one or more Addons.
3. **Addon Package** is a distribution artifact, such as a binary, container
   image, archive, or package manager entry, that contains one or more Addon
   Sidecars or Addon manifests.
4. **Addon Suite** is an Addon Package intentionally grouping related Addons
   for operator convenience while preserving per-Addon manifests and grants.

Official Nako addons should default to an **Addon Suite** distribution shape
when the capabilities share a trust level, runtime dependency profile, and
operator lifecycle. A future `nako-official-addons` suite may expose metadata,
notification, watch-state sync, MCP, and Arr-stack integration Addons from one
container or binary while keeping each Addon's manifest, grants, tasks, and
events separate.

Nako must not require one Docker Compose service per Addon. Addon Install Guide
and Addon Manager planning surfaces should be able to generate:

- one service for a whole official Addon Suite;
- profile-driven snippets for subsets such as metadata, automation, remote, or
  compatibility;
- per-Addon registration and grant instructions after the suite is running.

Nako may still recommend a separate Addon Sidecar or Addon Package when the
capability has a different trust or lifecycle boundary. Examples include:

- Network Tunnel Provider integration that needs host networking, privileged
  network configuration, external relay credentials, or stricter isolation;
- DLNA/UPnP compatibility that needs noisy LAN discovery and multicast access;
- browser-automation workers with large dependencies or distinct sandboxing;
- local AI/model runtimes with GPU, memory, or licensing constraints.

The Addon Protocol remains the runtime contract. Addon Package and Addon Suite
metadata belongs in source catalog, marketplace, install-guide, or manager-plan
surfaces. It must not replace the manifest-level permissions and protocol
compatibility checks.

## Consequences

- Operators can run a small Docker Compose file while Nako still grants and
  audits individual Addons.
- The official addon repository can grow into a suite without weakening the
  Addon Sidecar security model.
- Addon Manager can plan installation at package or suite granularity and
  still perform registration, health, token, grant, and task/event routing at
  Addon granularity.
- Package signing and source catalog design must eventually describe both the
  package/suite artifact and the contained Addon manifests.
- Nako core still does not install, start, stop, update, remove, log, or
  supervise sidecar processes until a later lifecycle automation decision
  explicitly accepts that authority.
- The "fine-grained permissions, coarse-grained deployment" principle becomes
  the default for official addon ecosystem work.

## Alternatives Considered

- One container per Addon: rejected because it makes a normal self-hosted
  deployment too verbose and pushes Compose complexity onto operators.
- One giant trusted plugin process with coarse permissions: rejected because it
  collapses permission, audit, task, and event ownership into a broad trust
  grant.
- Move future addon features into Nako core to simplify Compose: rejected
  because network tunnels, third-party integrations, notification bridges,
  MCP/AI, DLNA/UPnP, and downloader integrations have different dependency,
  credential, failure, and isolation profiles from the core media server.
- Let Addon Manager require Docker socket control early: rejected for now
  because install-guide and manager-plan outputs can improve operator
  experience without giving Nako host process supervision authority.

## Related Workstreams

- `docs/workstreams/addon-ecosystem-foundation/`
- `docs/workstreams/addon-architecture-deepening/`
- `docs/workstreams/addon-runtime-and-distribution/`
- `docs/workstreams/addon-manager-lifecycle-automation/`
- `docs/workstreams/addon-source-catalog-marketplace/`
- `docs/workstreams/addon-task-runtime-contract/`
- `docs/workstreams/network-access-boundary/`
