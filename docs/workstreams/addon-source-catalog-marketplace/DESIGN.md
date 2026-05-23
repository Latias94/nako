# Addon Source Catalog And Marketplace

Status: Active
Last updated: 2026-05-23

## Why This Lane Exists

The Addon Manager lifecycle lane is complete. Nako can now expose a
manager-owned registry/plan slot with explicit operator-confirmed lifecycle
intent, but it still lacks a clear source catalog and marketplace discovery
boundary for how addon sources are listed, resolved, and browsed before an
operator chooses a plan.

## Problem

Operators still need a discoverable place to find addon sources and resolve
installable addon descriptors. The manager plan slot can confirm intent, but it
does not yet define how addon sources become catalog entries or marketplace
metadata.

## Target State

When this lane closes, Nako should be able to:

- list one or more configured addon sources through a redaction-safe catalog
  surface;
- resolve installable addon descriptors or package metadata from those sources;
- expose browseable marketplace metadata without taking over package signing or
  process supervision;
- keep provider breadth, trust roots, and execution ownership as separate
  follow-ons;
- preserve the Addon Protocol and manager-plan contract while discovery grows.

## Scope

- Addon source catalog modeling.
- Marketplace-style discovery and listing metadata.
- Source resolution for installable addon descriptors.
- Minimal Admin/API surfaces needed to browse or select source entries.
- Documentation and validation for the first discovery slice.

## Non-Goals

- Package signing trust roots.
- Direct container or process supervision.
- Broad provider breadth beyond the first source set.
- Native Plugin ABI or in-process addon execution.
- Changes to the operator-confirmed manager-plan intent contract.

## Architecture Direction

Treat the catalog as a discovery and resolution control plane, not as a
distribution authority. The catalog should own:

- source listing and metadata lookup;
- descriptor resolution and candidate selection;
- redaction-safe browse and install entrypoints;
- stable handoff into the existing manager-plan surface.

The catalog should not smuggle in signing policy, sidecar lifecycle ownership,
or hidden provider execution. Marketplace behavior is the discovery surface,
not a new runtime authority.

## Reference Patterns

- Jellyfin uses multiple official and third-party plugin repositories, with a
  catalog that can browse installed repositories and install from repository
  manifests. That suggests Nako should support more than one source and keep
  repository metadata separate from per-addon entries.
- Home Assistant lets users add a repository URL to the store, and each
  repository contains one or more add-ons with a root repository manifest. That
  suggests Nako should treat source identity as first-class and keep source-level
  metadata distinct from addon manifests.
- Visual Studio Code extensions use a manifest with `publisher`, `name`,
  `version`, and `engines.vscode`, plus a pre-release channel. That suggests
  Nako should keep addon version separate from host compatibility and channel
  policy.
- Obsidian plugins use `version` and `minAppVersion`, with `versions.json`
  handling host-version-specific selection. That suggests Nako should model
  host compatibility explicitly rather than inferring it from addon version
  alone.

Nako's implication: source catalog entries should carry repo-level metadata,
per-addon descriptor metadata, and explicit compatibility/channel policy while
remaining separate from package signing or process ownership.

## Adjacent Follow-On

If Nako later needs a real `Addon Task` runtime contract, that should be a
separate lane from the source catalog. The catalog may surface which addons
declare tasks, but it should not turn task declaration names like
`bulk-metadata-scrape` into a runtime promise until a host-owned task execution
model exists with progress, results, cancellation, and retry semantics.

## Related Docs

- `docs/workstreams/addon-manager-lifecycle-automation/`
- `docs/workstreams/official-addon-e2e-alpha2/`
- `docs/workstreams/addon-runtime-and-distribution/`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
